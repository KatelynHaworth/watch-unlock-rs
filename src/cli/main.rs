mod cmds;
#[path = "../lib.rs"]
mod lib;

use clap::Command;
use std::env;
use std::process::exit;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cmd_defs = cmds::commands();
    let commands: Vec<Command> = cmd_defs
        .iter()
        .map(|delegate| delegate.definition())
        .collect();

    let mut cmd = Command::new(env!("CARGO_CRATE_NAME")).subcommands(commands);
    let matches = cmd.get_matches_mut();
    let Some((cmd_name, args)) = matches.subcommand() else {
        let _ = cmd.print_help();
        exit(1)
    };

    let Some(delegate) = cmd_defs.iter().find(|delegate| delegate.name() == cmd_name) else {
        let _ = cmd.print_help();
        exit(1)
    };

    let status_code = delegate.execute(args).await;
    exit(status_code);
}
