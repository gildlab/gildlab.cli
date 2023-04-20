use anyhow::Result;
use clap::command;
use clap::Command;
mod pins;

pub async fn main() -> Result<()> {
    tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;

    let matches = command!()
        .subcommand(Command::new(pins::NAME).about(pins::ABOUT))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches(pins::NAME) {
        pins::pins(matches).await?;
    }

    Ok(())
}
