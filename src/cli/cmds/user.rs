use crate::cmds::CommandDelegate;
use crate::lib::conf::Config;

use async_trait::async_trait;
use clap::{Arg, ArgMatches, Command};

pub struct UserCommand;

#[async_trait(?Send)]
impl CommandDelegate for UserCommand {
    fn name(&self) -> &'static str {
        "add_user"
    }

    fn definition(&self) -> Command {
        Command::new(self.name())
            .about("Adds a new user to the Apple Watch PAM module configuration")
            .long_about(concat!(
                "Updates the Apple Watch PAM module config, /etc/security/apple_watch.conf,\n",
                "to either add a new user mapping, or update an existing mapping if one exist.\n",
                "\n",
                "This command requires root permission (i.e. sudo) to modify the configuration file."
            ))
            .arg(
                Arg::new("user")
                    .required(true)
                    .help("Specifies the user to create, or update, a configuration mapping for"),
            )
            .arg(Arg::new("irk").required(true).help(
                "Specifies the Base64 encoded Identity Resolution Key for the user's Apple Watch",
            ))
    }

    async fn execute(&self, args: &ArgMatches) -> i32 {
        let user: &String = args.get_one("user").expect("required argument");
        let irk: &String = args.get_one("irk").expect("required argument");

        println!("Loading configuration for Apple Watch PAM module");
        let mut config = match Config::load() {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Failed to load configuration: {err}");
                return 1;
            }
        };

        println!("Adding user '{user}' to PAM module configuration");
        if config.update_user(user, irk) {
            println!("WARN: User already existed, updating existing entry");
        }

        println!("Saving configuration");
        if let Err(err) = config.save() {
            eprintln!("Failed to save configuration: {err}");
            return 1;
        }

        println!("Configuration saved successfully");
        0
    }
}
