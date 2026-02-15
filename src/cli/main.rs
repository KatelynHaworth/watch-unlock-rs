#[path = "../watch.rs"]
mod watch;

use crate::watch::AppleWatch;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::{Arg, ArgMatches, Command};
use std::env;
use std::process::exit;
use std::time::Duration;

fn cli_commands() -> [Command; 1] {
    [Command::new("query_status")
        .about("Queries the current status of an Apple Watch")
        .arg(
            Arg::new("irk")
                .required(true)
                .help("Identity Resolution Key, in base64, of the Apple Watch to query"),
        )]
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut cmd = Command::new(env!("CARGO_CRATE_NAME")).subcommands(cli_commands());
    let matches = cmd.get_matches_mut();

    #[allow(clippy::single_match_else)]
    let status_code = match matches.subcommand() {
        Some(("query_status", args)) => query_status(args).await,
        _ => {
            let _ = cmd.print_help();
            1
        }
    };

    exit(status_code);
}

async fn query_status(args: &ArgMatches) -> i32 {
    let encoded_irl: &String = args.get_one("irk").expect("required argument");

    println!("Decoding Identity Resolution Key for Apple Watch");
    let mut raw_irk: [u8; 16] = [0; 16];
    match STANDARD.decode_slice(encoded_irl, &mut raw_irk[..]) {
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

    println!("Creating Bluetooth session");
    let session = match bluer::Session::new().await {
        Ok(session) => session,
        Err(err) => {
            println!("Failed to create Bluetooth session: {err}");
            return 1;
        }
    };

    println!("Selecting default Bluetooth adapter");
    let adapter = match session.default_adapter().await {
        Ok(adapter) => adapter,
        Err(err) => {
            println!("Failed to obtain access to default Bluetooth adapter: {err}");
            return 1;
        }
    };

    println!("Searching for Apple Watch");
    match watch
        .find_watch(&adapter, 3, Duration::from_millis(500))
        .await
    {
        Err(err) => {
            println!("Failed to find Apple Watch: {err}");
            return 1;
        }
        Ok(tries) => println!("Found Apple Watch after {tries} tries"),
    }

    let status = match watch.get_watch_status().await {
        Ok(status) => status,
        Err(err) => {
            println!("Failed to get Apple Watch status: {err}");
            return 1;
        }
    };

    println!("Apple Watch Status");
    println!(
        "\tAddress.......................: {}",
        watch.get_watch_address()
    );
    println!("\tRSSI..........................: {}", status.rssi);
    println!("\tUnlocked......................: {}", status.locked);
    println!(
        "\tAuto-unlock devices enabled...: {}",
        status.device_auto_unlock_enabled
    );

    0
}
