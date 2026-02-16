use crate::cmds::CommandDelegate;

use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};
use pam::{Client, Conversation};
use std::ffi::{CStr, CString};
use std::str::FromStr;

pub struct PAMTestCommand;

impl PAMTestCommand {
    const PAM_MODULE_NAME: &'static str = "apple-watch";
}

#[async_trait(?Send)]
impl CommandDelegate for PAMTestCommand {
    fn name(&self) -> &'static str {
        "pam_test"
    }

    fn definition(&self) -> Command {
        Command::new(self.name())
            .about("Test the Apple Watch PAM module")
            .arg(
                Arg::new("user")
                    .required(true)
                    .help("Specifies the username to supply to the Apple Watch PAM module"),
            )
            .arg(
                Arg::new("service-name")
                    .default_value(Self::PAM_MODULE_NAME)
                    .help(
                        "Specifies the name of the PAM service policy configuration in /etc/pam.d",
                    ),
            )
    }

    async fn execute(&self, args: &ArgMatches) -> i32 {
        let user: &String = args.get_one("user").expect("required argument");
        let service: &String = args.get_one("service-name").expect("has default");

        println!("Connecting to Apple Watch PAM module [{service}]");
        let mut client = match Client::with_conversation(
            service,
            MiscConv {
                mod_name: service.clone(),
                user: user.clone(),
            },
        ) {
            Ok(client) => client,
            Err(err) => {
                eprintln!("Failed to connect to Apple Watch PAM module: {err}");
                return 1;
            }
        };

        println!("Testing PAM module authentication with user '{user}'");
        match client.authenticate() {
            Ok(()) => {
                println!("Authentication was successful!");
                0
            }
            Err(err) => {
                eprintln!("Authentication was unsuccessful, PAM return code: {err}");
                1
            }
        }
    }
}

pub struct MiscConv {
    pub mod_name: String,
    pub user: String,
}

impl Conversation for MiscConv {
    fn prompt_echo(&mut self, _: &CStr) -> Result<CString, ()> {
        Ok(CString::from_str(self.user.as_str()).unwrap())
    }

    fn prompt_blind(&mut self, _: &CStr) -> Result<CString, ()> {
        todo!()
    }

    fn info(&mut self, msg: &CStr) {
        println!("[{}] INFO: {}", self.mod_name, msg.to_str().unwrap());
    }

    fn error(&mut self, msg: &CStr) {
        eprintln!("[{}] ERROR: {}", self.mod_name, msg.to_str().unwrap());
    }
}
