use std::collections::HashMap;
use std::time::Duration;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use bluer::{Adapter, AdapterEvent, Address, DiscoveryFilter, DiscoveryTransport};
use ecb::cipher::{BlockEncryptMut, KeyInit};
use ecb::cipher::block_padding::{NoPadding};
use futures::{StreamExt};
use futures::stream::{BoxStream};
use thiserror::Error;
use tokio::time::timeout;
use crate::WatchStatusError::{AppleContinuityMessageError, ManufacturerDataUnavailable, RSSIUnavailable};

const WATCH_IRK: &'static str = "";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Decoding Identity Resolution Key for Apple Watch");
    let mut raw_irk: [u8; 16] = [0; 16];
    STANDARD.decode_slice(WATCH_IRK, &mut raw_irk[..]).expect("decode success");
    raw_irk.reverse();

    println!("Connecting to BT system");

    let session = bluer::Session::new().await.expect("open BT session");
    let adapter = session.default_adapter().await.expect("obtain default adapter");

    println!("Turning on default adapter");
    adapter.set_powered(true).await.expect("adapter turns on");

    println!("Configuring adapter for BTLE scanning");
    adapter.set_discovery_filter(DiscoveryFilter {
        transport: DiscoveryTransport::Le,
        ..Default::default()
    }).await.expect("adapter configured for BTLE scanning");

    let search_result = timeout(Duration::from_secs(1), find_apple_watch(&adapter, &raw_irk)).await;
    match search_result {
        Err(_) => println!("Timed out trying to find apple watch"),
        Ok(Err(err)) => println!("Encountered error searching for apple watch: {:?}", err),
        Ok(Ok(addr)) => {
            println!("Found Apple Watch at: {:?}", addr);
            let status = WatchStatus::extract_from(&adapter, addr).await.expect("get watch status");
            println!("Got Apple Watch status: {:?}", status)
        }
    }
}

async fn find_apple_watch(adapter: &Adapter, ikr: &[u8; 16]) -> bluer::Result<Address> {
    println!("Searching for Apple Watch");
    let mut device_events = adapter.discover_devices().await?;

    loop {
        tokio::select! {
            Some(device_event) = device_events.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) if is_target_apple_watch(addr,ikr) =>
                        return Ok(addr),
                    _ => continue
                }
            }
        }
    }
}

#[derive(Debug)]
struct WatchStatus {
    rssi: i16,
    locked: bool,
    device_auto_unlock_enabled: bool
}

#[derive(Error, Debug)]
enum WatchStatusError {
    #[error("Bluetooth system returned an error: {0}")]
    BluetoothError(#[from] bluer::Error),

    #[error("No RSSI is unavailable for the Apple Watch")]
    RSSIUnavailable,

    #[error("Manufacture data is unavailable for the Apple Watch")]
    ManufacturerDataUnavailable,

    #[error("Apple Continuity message invalid: {0}")]
    AppleContinuityMessageError(&'static str)
}

impl WatchStatus {
    const MANUFACTURER_CODE_APPLE: u16 = 0x004c;
    const NEARBY_INFO_MESSAGE: u8 = 0x10;
    const NEARBY_INFO_DATA_FLAG_WATCH_LOCKED: u8 = 0x20;
    const NEARBY_INFO_DATA_FLAG_AUTO_UNLOCK_ENABLED: u8 = 0x80;

    async fn extract_from(adapter: &Adapter, addr: Address) -> Result<Self, WatchStatusError> {
        let device = adapter.device(addr)?;

        let rssi = match device.rssi().await? {
            None => return Err(RSSIUnavailable),
            Some(rssi) => rssi
        };

        let manufacturer_data = match device.manufacturer_data().await? {
            None => return Err(ManufacturerDataUnavailable),
            Some(data) => match data.get(&Self::MANUFACTURER_CODE_APPLE) {
                None => return Err(ManufacturerDataUnavailable),
                Some(apple_data) => apple_data.clone()
            },
        };

        let message_header = manufacturer_data.get(..2).ok_or(AppleContinuityMessageError("header unavailable"))?;
        if message_header[0] != Self::NEARBY_INFO_MESSAGE {
            return Err(AppleContinuityMessageError("expected Nearby Info message"))
        } else if message_header[1] < 0x5 {
            return Err(AppleContinuityMessageError("Nearby Info message is less than 5 bytes long"))
        }

        let data_flags = manufacturer_data.get(3).ok_or(AppleContinuityMessageError("Nearby Info message data flags unavailable"))?;
        Ok(WatchStatus {
            rssi,
            locked: (data_flags & Self::NEARBY_INFO_DATA_FLAG_WATCH_LOCKED) != 0x0,
            device_auto_unlock_enabled: (data_flags & Self::NEARBY_INFO_DATA_FLAG_AUTO_UNLOCK_ENABLED) != 0,
        })
    }
}

async fn handle_event_stream<'a>(adapter: &Adapter, ikr: &[u8; 16], mut stream: BoxStream<'a, AdapterEvent>) {
    println!("Waiting for device events");
    loop {
        tokio::select! {
            Some(device_event) = stream.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        query_device(adapter, addr).await.expect("query device");
                    },
                    AdapterEvent::DeviceRemoved(addr) => {
                        println!("Device removed: {:?}", addr)
                    },
                    AdapterEvent::PropertyChanged(prop) => {
                        println!("Property change: {:?}", prop)
                    },
                }
                println!()
            }
        }
    }
}

async fn query_device(adapter: &Adapter, addr: Address) -> bluer::Result<()> {
    let device = adapter.device(addr)?;
    println!("    Address type:       {}", device.address_type().await?);
    println!("    Address:            {:?}", addr);
    println!("    Name:               {:?}", device.name().await?);
    println!("    Icon:               {:?}", device.icon().await?);
    println!("    Class:              {:?}", device.class().await?);
    println!("    UUIDs:              {:?}", device.uuids().await?.unwrap_or_default());
    println!("    Paired:             {:?}", device.is_paired().await?);
    println!("    Connected:          {:?}", device.is_connected().await?);
    println!("    Trusted:            {:?}", device.is_trusted().await?);
    println!("    Modalias:           {:?}", device.modalias().await?);
    println!("    RSSI:               {:?}", device.rssi().await?);
    println!("    TX power:           {:?}", device.tx_power().await?);
    let manufacturer_data = device.manufacturer_data().await?;
    println!("    Manufacturer data:  {:x?}", manufacturer_data);
    println!("    Service data:       {:?}", device.service_data().await?);

    if let Some(data) = manufacturer_data {
        match data.get(&0x4c) {
            None => {
                println!("WARN: No status information available for Apple Watch");
            },
            Some(message) => {
                assert_eq!(message.get(0), Some(&0x10), "nearby info message");
                assert_eq!(message.get(1), Some(&0x05), "correct length for nearby info message");
                let status_flags = message.get(3).expect("status flags");
                println!(
                    "watch locked: {:?} || device auto unlock enabled: {:?}",
                    (status_flags & 0x20) != 0x0,
                    (status_flags & 0x80) != 0x0
                );
            }
        }
    }

    Ok(())
}

type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;

fn is_target_apple_watch(addr: Address, ikr: &[u8; 16]) -> bool {
    if (addr.0[0] >> 6) != 0x01 {
        return false
    }

    let (top, bottom) = (&addr.0[..3], &addr.0[3..]);
    println!("{:?} {:?}", top, bottom);

    let mut buf: [u8; 16] = [0; 16];
    buf[13..16].copy_from_slice(top);

    let hashed_address = Aes128EcbEnc::new(ikr.into())
        .encrypt_padded_mut::<NoPadding>(&mut buf, 16)
        .expect("address hash");

    println!(
        "{:?} == {:?} ? {:?}",
        bottom, &hashed_address[13..16],
        bottom == &hashed_address[13..16]
    );

    bottom == &hashed_address[13..16]
}