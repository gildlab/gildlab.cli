use anyhow::Result;
use clap::command;
use clap::Command;
mod pins;

pub async fn main() -> Result<()> {
    let matches = command!()
        .subcommand(Command::new(pins::NAME).about(pins::ABOUT))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches(pins::NAME) {
        pins::pins(matches).await?;
    }

    Ok(())
}
