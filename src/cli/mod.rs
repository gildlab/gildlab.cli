use anyhow::Result;
use clap::{command, Arg, Command};

mod pins;

pub async fn main() -> Result<()> {
    // tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new())?;

    let matches = command!()
        .subcommand(
            Command::new(pins::NAME)
                .about(pins::ABOUT)
                .arg(Arg::new("manager").long("manager"))
                .arg(Arg::new("subgraph-url").long("subgraph-url")),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches(pins::NAME) {
        pins::pins(matches).await?;
    }

    Ok(())
}
