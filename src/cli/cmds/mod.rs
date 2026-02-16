mod pam_test;
mod query_status;

use crate::cmds::pam_test::PAMTestCommand;
use crate::cmds::query_status::QueryStatusCommand;

use async_trait::async_trait;
use clap::{ArgMatches, Command};

#[async_trait(?Send)]
pub trait CommandDelegate {
    fn name(&self) -> &'static str;

    fn definition(&self) -> Command;

    async fn execute(&self, args: &ArgMatches) -> i32;
}

pub fn commands() -> [Box<dyn CommandDelegate>; 2] {
    [Box::new(QueryStatusCommand), Box::new(PAMTestCommand)]
}
