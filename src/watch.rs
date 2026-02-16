use crate::lib::watch::AppleWatchError::{
    AppleContinuityMessageError, BluetoothError, ManufacturerDataUnavailable, RSSIUnavailable,
    RetriesExceeded,
};

use aes::cipher::block_padding::NoPadding;
use bluer::{Adapter, AdapterEvent, Address, Device, DiscoveryFilter, DiscoveryTransport};
use ecb::cipher::{BlockEncryptMut, KeyInit};
use futures::StreamExt;
use std::time::Duration;
use thiserror::Error;
use tokio::time::timeout;

type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;

pub struct AppleWatch {
    identity_resolution_key: [u8; 16],
    device: Option<Device>,
}

impl AppleWatch {
    /// Creates a new [`AppleWatch`] that can be used to search
    /// for, and obtain the status of, an Apple Watch that has
    /// a Bluetooth address matching the supplied Identity Resolution
    /// Key.
    pub fn new(irk: [u8; 16]) -> Self {
        Self {
            identity_resolution_key: irk,
            device: None,
        }
    }

    /// Searches for an Apple Watch using Bluetooth Low Energy that has
    /// an address that matches the configured Identity Resolution Key.
    ///
    /// This function will attempt multiple times to discover the device,
    /// when the watch is found it will return the number of tries it took
    /// to find it.
    pub async fn find_watch(
        &mut self,
        adapter: &Adapter,
        retries: u8,
        retry_timeout: Duration,
    ) -> Result<u8, AppleWatchError> {
        AppleWatchError::wrap_bluetooth_action("power-on adapter", || adapter.set_powered(true))
            .await?;

        AppleWatchError::wrap_bluetooth_action("configure BT-LE discovery filter", || {
            adapter.set_discovery_filter(DiscoveryFilter {
                transport: DiscoveryTransport::Le,
                ..Default::default()
            })
        })
        .await?;

        for i in 1..=retries {
            match timeout(retry_timeout, self.find_watch_internal(adapter)).await {
                Ok(Err(err)) => return Err(err),
                Ok(Ok(Some(device))) => {
                    self.device = Some(device);
                    return Ok(i);
                }

                // Discovery timed out or an RSSI wasn't available for
                // the watch, retry
                _ => (),
            }
        }

        Err(RetriesExceeded(retries))
    }

    /// Consumes device discovery events from the Bluetooth adapter
    /// till a device is discovered that has an address that matches,
    /// via [`AppleWatch::is_matching_watch_address`], the Apple Watch
    /// being searched for.
    async fn find_watch_internal(
        &self,
        adapter: &Adapter,
    ) -> Result<Option<Device>, AppleWatchError> {
        let mut device_events = AppleWatchError::wrap_bluetooth_action("discover devices", || {
            adapter.discover_devices()
        })
        .await?;

        loop {
            tokio::select! {
                Some(device_event) = device_events.next() => {
                    match device_event {
                        AdapterEvent::DeviceAdded(addr) if self.is_matching_watch_address(addr) => {
                            // Before returning a successful discovery, first make
                            // sure an RSSI value is available otherwise it won't
                            // be possible to use it for unlocking a user session.
                            let device = match adapter.device(addr) {
                                Err(err) => return Err(BluetoothError {
                                    action: "get Apple Watch device",
                                    source: err
                                }),
                                Ok(device) => device
                            };

                            let rssi = AppleWatchError::wrap_bluetooth_action("get device RSSI", || {
                                device.rssi()
                            }).await?;

                            return match rssi {
                                Some(_) => Ok(Some(device)),
                                None => Ok(None)
                            }
                        }
                        _ => ()
                    }
                }
            }
        }
    }

    /// Determines if the supplied Bluetooth address matches
    /// the desired Apple Watch by using the Identity Resolution Key
    /// to encrypt, using AES-128 ECB, the top 3 bytes of the address
    /// and comparing the result against the bottom 3 bytes of the address.
    ///
    /// If the encrypted top bytes match the bottom bytes then the Bluetooth
    /// address is that of the Apple Watch desired.
    fn is_matching_watch_address(&self, addr: Address) -> bool {
        if (addr.0[0] >> 6) != 0x01 {
            return false;
        }

        let (top, bottom) = (&addr.0[..3], &addr.0[3..]);

        let mut buf: [u8; 16] = [0; 16];
        buf[13..16].copy_from_slice(top);

        let hashed_address = Aes128EcbEnc::new(&self.identity_resolution_key.into())
            .encrypt_padded_mut::<NoPadding>(&mut buf, 16)
            .expect("address hash");

        bottom == &hashed_address[13..16]
    }

    /// Specifies the 16-bit unsigned integer Company Identifier
    /// assigned to Apple for use in Bluetooth protocols.
    ///
    /// Reference: <https://www.bluetooth.com/wp-content/uploads/Files/Specification/HTML/Assigned_Numbers/out/en/Assigned_Numbers.pdf>
    const MANUFACTURER_CODE_APPLE: u16 = 0x004c;

    /// Specifies the 8-bit unsigned integer Apple Continuity
    /// message type for Nearby Information messages.
    ///
    /// Reference: <https://github.com/furiousMAC/continuity/blob/master/messages/nearby_info.md>
    const NEARBY_INFO_MESSAGE: u8 = 0x10;

    /// Specifies the bit-flag for the Apple Watch Locked status
    /// within the data flags segment of a Nearby Information
    /// message.
    ///
    /// Reference: <https://github.com/furiousMAC/continuity/blob/master/messages/nearby_info.md>
    const NEARBY_INFO_DATA_FLAG_WATCH_LOCKED: u8 = 0x20;

    /// Specifies the bit-flag for the Apple Watch Auto Unlock,
    /// for other devices (e.g. Macbook or iPhone), within the
    /// data flags segment of a Nearby Information message.
    ///
    /// Reference: <https://github.com/furiousMAC/continuity/blob/master/messages/nearby_info.md>
    const NEARBY_INFO_DATA_FLAG_AUTO_UNLOCK_ENABLED: u8 = 0x80;

    /// Returns an [`AppleWatchStatus`] for this [`AppleWatch`] by extracting
    /// the information from the manufacturer data advertised by the Apple Watch
    /// over Bluetooth Low Energy.
    ///
    /// ### Panics
    /// This function expects that [`AppleWatch::find_watch`] has been called first
    /// to identify the target Bluetooth device from which to extract the information.
    pub async fn get_watch_status(&self) -> Result<AppleWatchStatus, AppleWatchError> {
        let device = self.device.clone().expect("device already found");

        let Some(rssi) =
            AppleWatchError::wrap_bluetooth_action("get device RSSI", || device.rssi()).await?
        else {
            // This _should_ be impossible given it is checked for in
            // find_watch_internal, but is better to be safe than sorry.
            // Graceful errors are better than panics.
            return Err(RSSIUnavailable);
        };

        let apple_data: Vec<u8> =
            match AppleWatchError::wrap_bluetooth_action("get manufacture data", || {
                device.manufacturer_data()
            })
            .await?
            {
                None => return Err(ManufacturerDataUnavailable("root")),
                Some(data) => match data.get(&Self::MANUFACTURER_CODE_APPLE) {
                    None => return Err(ManufacturerDataUnavailable("apple")),
                    Some(apple_data) => apple_data.clone(),
                },
            };

        let message_header = apple_data
            .get(..2)
            .ok_or(AppleContinuityMessageError("header unavailable"))?;

        if message_header[0] != Self::NEARBY_INFO_MESSAGE {
            return Err(AppleContinuityMessageError("expected Nearby Info message"));
        } else if message_header[1] < 0x5 {
            return Err(AppleContinuityMessageError(
                "Nearby Info message is less than 5 bytes long",
            ));
        }

        let message = apple_data.get(2..5).ok_or(AppleContinuityMessageError(
            "Nearby Info message data unavailable",
        ))?;

        let data_flags = message[1];
        Ok(AppleWatchStatus {
            rssi,
            locked: (data_flags & Self::NEARBY_INFO_DATA_FLAG_WATCH_LOCKED) != 0x0,
            device_auto_unlock_enabled: (data_flags
                & Self::NEARBY_INFO_DATA_FLAG_AUTO_UNLOCK_ENABLED)
                != 0,
        })
    }

    /// Returns the [`bluer::Address`] of the Apple Watch
    /// found by [`AppleWatch::find_watch`].
    ///
    /// ## Panics
    /// A panic will be thrown if [`AppleWatch::find_watch`] has not been called
    /// successfully before invoking this function.
    pub fn get_watch_address(&self) -> Address {
        self.device
            .as_ref()
            .expect("device already found")
            .address()
    }
}

#[derive(Debug)]
pub struct AppleWatchStatus {
    /// Specifies the received signal strength indicator of the
    /// Apple Watch, this value can be used to imply the distance
    /// of the Apple Watch from the receiving device.
    pub rssi: i16,

    /// Specifies if the Apple Watch is currently locked.
    pub locked: bool,

    /// Specifies if the Apple Watch is configured to allow
    /// for the unlocking of remote devices (e.g. Macbook, iPhone).
    pub device_auto_unlock_enabled: bool,
}

#[derive(Error, Debug)]
pub enum AppleWatchError {
    #[error("Bluetooth action '{action}' returned an error: {source}")]
    BluetoothError {
        action: &'static str,
        #[source]
        source: bluer::Error,
    },

    #[error("Apple Watch search retries ({0}) exceeded")]
    RetriesExceeded(u8),

    #[error("No RSSI is unavailable for the Apple Watch")]
    RSSIUnavailable,

    #[error("Manufacture data (source: {0}) is unavailable for the Apple Watch")]
    ManufacturerDataUnavailable(&'static str),

    #[error("Apple Continuity message invalid: {0}")]
    AppleContinuityMessageError(&'static str),
}

impl AppleWatchError {
    /// Helper function to wrap a [`bluer::Error`] into a [`AppleWatchError`]
    /// along with the action that was attempted that resulted in
    /// the error.
    async fn wrap_bluetooth_action<R, T>(action: &'static str, f: T) -> Result<R, AppleWatchError>
    where
        T: AsyncFn() -> bluer::Result<R>,
    {
        match f().await {
            Err(err) => Err(BluetoothError {
                action,
                source: err,
            }),
            Ok(result) => Ok(result),
        }
    }
}
