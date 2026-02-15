#[path = "../watch.rs"]
mod watch;

use crate::watch::AppleWatch;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::env;
use std::process::exit;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Invalid usage");
        eprintln!();

        eprintln!("Usage:");
        eprintln!("\t{} [identity_resolution_key]", args.first().unwrap());

        eprintln!("Parameters:");
        eprintln!(
            "\tidentity_resolution_key - The base64 IRK required to match the desired Apple Watch"
        );
        exit(1);
    }

    println!("Decoding Identity Resolution Key for Apple Watch");
    let mut raw_irk: [u8; 16] = [0; 16];
    match STANDARD.decode_slice(args.get(1).unwrap(), &mut raw_irk[..]) {
        Err(err) => {
            eprintln!("Failed to decode IRK: {err}");
            exit(1)
        }
        Ok(decoded_length) if decoded_length != 16 => {
            eprintln!("Corrupt IRK, it must be 16 bytes long");
            exit(1)
        }
        Ok(_) => raw_irk.reverse(),
    }

    let mut watch = AppleWatch::new(raw_irk);

    println!("Connecting to BT system");

    let session = bluer::Session::new().await.expect("open BT session");
    let adapter = session
        .default_adapter()
        .await
        .expect("obtain default adapter");

    println!("Searching for Apple Watch");
    match watch
        .find_watch(&adapter, 3, Duration::from_millis(500))
        .await
    {
        Err(err) => {
            println!("Failed to find Apple Watch: {err}");
            exit(1)
        }
        Ok(tries) => println!("Found Apple Watch after {tries} tries"),
    }

    match watch.get_watch_status().await {
        Err(err) => {
            println!("Failed to get Apple Watch status: {err}");
            exit(1)
        }
        Ok(status) => println!("Got Apple Watch status: {status:?}"),
    }
}
