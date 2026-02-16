mod conv;
#[path = "../lib.rs"]
mod lib;

use crate::lib::watch::{AppleWatch, AppleWatchStatus};

use crate::conv::ClientConv;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use pam::{export_pam_module, PamHandle, PamModule, PamReturnCode};
use std::collections::HashMap;
use std::ffi::{c_uint, CStr};
use std::time::Duration;

struct AppleWatchPAM;
export_pam_module!(AppleWatchPAM);

impl PamModule for AppleWatchPAM {
    fn authenticate(handle: &PamHandle, args: Vec<&CStr>, _: c_uint) -> PamReturnCode {
        let Ok(async_runtime) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return PamReturnCode::Service_Err;
        };

        let args: Vec<_> = args.iter().map(|s| s.to_string_lossy()).collect();

        let args: HashMap<&str, &str> = args
            .iter()
            .map(|s| {
                let mut parts = s.splitn(2, '=');
                (parts.next().unwrap(), parts.next().unwrap_or(""))
            })
            .collect();

        let conv = match ClientConv::try_from(handle) {
            Ok(conv) => conv,
            Err(err) => {
                eprintln!("Unable to get pam_conv");
                return err.0;
            }
        };

        let Some(encoded_irk) = args.get("irk") else {
            eprintln!("IRK is not configured");
            return PamReturnCode::No_Module_Data;
        };

        println!("Decoding Identity Resolution Key for Apple Watch");
        let mut raw_irk: [u8; 16] = [0; 16];
        match STANDARD.decode_slice(encoded_irk, &mut raw_irk[..]) {
            Err(err) => {
                eprintln!("Failed to decode IRK: {err}");
                return PamReturnCode::Authinfo_Unavail;
            }
            Ok(decoded_length) if decoded_length != 16 => {
                eprintln!("Corrupt IRK, it must be 16 bytes long");
                return PamReturnCode::Bad_Item;
            }
            Ok(_) => raw_irk.reverse(),
        }

        async_runtime
            .block_on(async { AppleWatchPAM::unlock_with_apple_watch(&conv, raw_irk).await })
    }
}

impl AppleWatchPAM {
    const UNLOCK_THRESHOLD: i16 = -80;

    async fn unlock_with_apple_watch(conv: &ClientConv<'_>, irk: [u8; 16]) -> PamReturnCode {
        let Ok(session) = bluer::Session::new().await else {
            return PamReturnCode::Service_Err;
        };

        let Ok(adapter) = session.default_adapter().await else {
            return PamReturnCode::Service_Err;
        };

        conv.info(c"Searching for Apple Watch");
        let mut watch = AppleWatch::new(irk);

        match watch
            .find_watch(&adapter, 3, Duration::from_millis(500))
            .await
        {
            Err(err) => {
                eprintln!("Failed to find Apple Watch: {err}");
                conv.error(c"Apple Watch not available");
                return PamReturnCode::Ignore;
            }
            Ok(tries) => println!("Found Apple Watch after {tries} tries"),
        }

        match watch.get_watch_status().await {
            Err(err) => {
                eprintln!("Failed to get Apple Watch status: {err}");
                conv.error(c"Apple Watch not available");
                PamReturnCode::Ignore
            }
            Ok(status) => match status {
                AppleWatchStatus { rssi, .. } if rssi < Self::UNLOCK_THRESHOLD => {
                    eprintln!(
                        "Apple Watch RSSI: {}, Target Threshold: {}",
                        rssi,
                        Self::UNLOCK_THRESHOLD
                    );
                    conv.error(c"Apple Watch is too far away");
                    PamReturnCode::Ignore
                }
                AppleWatchStatus { locked, .. } if locked => {
                    conv.error(c"Apple Watch is locked");
                    PamReturnCode::Ignore
                }
                AppleWatchStatus {
                    device_auto_unlock_enabled,
                    ..
                } if !device_auto_unlock_enabled => {
                    conv.error(c"Apple Watch is not configured to auto-unlock devices");
                    PamReturnCode::Ignore
                }
                _ => {
                    conv.info(c"Unlocking with Apple Watch");
                    PamReturnCode::Success
                }
            },
        }
    }
}
