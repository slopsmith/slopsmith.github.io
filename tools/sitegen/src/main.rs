mod docs;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Generates content for the slopsmith site before `zola build`")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Copy ps5slop's user docs into the site.
    Docs {
        /// A checkout of slopsmith/ps5slop.
        #[arg(long, env = "PS5SLOP_DIR")]
        ps5slop: PathBuf,
        #[arg(long, default_value = "content/ps5slop/docs")]
        out: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Docs { ps5slop, out } => docs::run(&ps5slop.join("docs/user"), &out),
    }
}
