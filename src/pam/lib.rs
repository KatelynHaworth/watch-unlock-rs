#[path = "../watch.rs"]
mod watch;

use crate::watch::{AppleWatch, AppleWatchStatus};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use pam::constants::{PamFlag, PamResultCode, PAM_TEXT_INFO};
use pam::conv::Conv;
use pam::module::{PamHandle, PamHooks};
use pam::{pam_hooks, pam_try};
use std::collections::HashMap;
use std::ffi::CStr;
use std::time::Duration;

struct AppleWatchPAM;
pam_hooks!(AppleWatchPAM);

impl PamHooks for AppleWatchPAM {
    fn sm_authenticate(pamh: &mut PamHandle, args: Vec<&CStr>, _: PamFlag) -> PamResultCode {
        let async_runtime = pam_try!(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build(),
            PamResultCode::PAM_SERVICE_ERR
        );

        let args: Vec<_> = args.iter().map(|s| s.to_string_lossy()).collect();

        let args: HashMap<&str, &str> = args
            .iter()
            .map(|s| {
                let mut parts = s.splitn(2, '=');
                (parts.next().unwrap(), parts.next().unwrap_or(""))
            })
            .collect();

        let conv = match pamh.get_item::<Conv>() {
            Ok(Some(conv)) => conv,
            Ok(None) => {
                unreachable!("No conv available");
            }
            Err(err) => {
                eprintln!("Couldn't get pam_conv");
                return err;
            }
        };

        let Some(encoded_irk) = args.get("irk") else {
            eprintln!("IRK is not configured");
            return PamResultCode::PAM_NO_MODULE_DATA;
        };

        println!("Decoding Identity Resolution Key for Apple Watch");
        let mut raw_irk: [u8; 16] = [0; 16];
        match STANDARD.decode_slice(encoded_irk, &mut raw_irk[..]) {
            Err(err) => {
                eprintln!("Failed to decode IRK: {err}");
                return PamResultCode::PAM_AUTHINFO_UNAVAIL;
            }
            Ok(decoded_length) if decoded_length != 16 => {
                eprintln!("Corrupt IRK, it must be 16 bytes long");
                return PamResultCode::PAM_BAD_ITEM;
            }
            Ok(_) => raw_irk.reverse(),
        }

        async_runtime
            .block_on(async { AppleWatchPAM::unlock_with_apple_watch(&conv, raw_irk).await })
    }
}

impl AppleWatchPAM {
    const UNLOCK_THRESHOLD: i16 = -80;

    async fn unlock_with_apple_watch(conv: &Conv<'_>, irk: [u8; 16]) -> PamResultCode {
        let session = pam_try!(bluer::Session::new().await, PamResultCode::PAM_SERVICE_ERR);
        let adapter = pam_try!(
            session.default_adapter().await,
            PamResultCode::PAM_SERVICE_ERR
        );

        let _ = conv.send(PAM_TEXT_INFO, "Searching for Apple Watch");
        let mut watch = AppleWatch::new(irk);

        match watch
            .find_watch(&adapter, 3, Duration::from_millis(500))
            .await
        {
            Err(err) => {
                eprintln!("Failed to find Apple Watch: {err}");
                let _ = conv.send(PAM_TEXT_INFO, "Apple Watch not available");
                return PamResultCode::PAM_IGNORE;
            }
            Ok(tries) => println!("Found Apple Watch after {tries} tries"),
        }

        match watch.get_watch_status().await {
            Err(err) => {
                eprintln!("Failed to get Apple Watch status: {err}");
                let _ = conv.send(PAM_TEXT_INFO, "Apple Watch not available");
                PamResultCode::PAM_IGNORE
            }
            Ok(status) => match status {
                AppleWatchStatus { rssi, .. } if rssi < Self::UNLOCK_THRESHOLD => {
                    eprintln!(
                        "Apple Watch RSSI: {}, Target Threshold: {}",
                        rssi,
                        Self::UNLOCK_THRESHOLD
                    );
                    let _ = conv.send(PAM_TEXT_INFO, "Apple Watch is too far away");
                    PamResultCode::PAM_IGNORE
                }
                AppleWatchStatus { locked, .. } if locked => {
                    let _ = conv.send(PAM_TEXT_INFO, "Apple Watch is locked");
                    PamResultCode::PAM_IGNORE
                }
                AppleWatchStatus {
                    device_auto_unlock_enabled,
                    ..
                } if !device_auto_unlock_enabled => {
                    let _ = conv.send(
                        PAM_TEXT_INFO,
                        "Apple Watch is not configured to auto-unlock devices",
                    );
                    PamResultCode::PAM_IGNORE
                }
                _ => {
                    let _ = conv.send(PAM_TEXT_INFO, "Unlocking with Apple Watch");
                    PamResultCode::PAM_SUCCESS
                }
            },
        }
    }
}
