mod pins;

use anyhow::Result;
use clap::Command;
use clap::command;
#[macro_use]
extern crate lazy_static;

static PINS_COMMAND_NAME: &str = "pins";

#[tokio::main]
async fn main() -> Result<()> {
    let matches = command!().subcommand(
        Command::new(PINS_COMMAND_NAME).about("Gets all pins from subgraph")
    ).get_matches();

    if let Some(_) = matches.subcommand_matches(PINS_COMMAND_NAME) {
        let pins = pins::pins().await?;
        dbg!(&pins);
    }

    Ok(())
}