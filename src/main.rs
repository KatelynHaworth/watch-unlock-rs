mod watch;

use crate::watch::AppleWatch;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::time::Duration;

const WATCH_IRK: &'static str = "";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Decoding Identity Resolution Key for Apple Watch");
    let mut raw_irk: [u8; 16] = [0; 16];
    STANDARD
        .decode_slice(WATCH_IRK, &mut raw_irk[..])
        .expect("decode success");
    raw_irk.reverse();

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
        Err(err) => println!("Failed to find Apple Watch: {:?}", err),
        Ok(tries) => println!("Found Apple Watch after {} tries", tries),
    };

    match watch.get_watch_status().await {
        Err(err) => println!("Failed to get Apple Watch status: {:?}", err),
        Ok(status) => println!("Got Apple Watch status: {:?}", status),
    };
}
