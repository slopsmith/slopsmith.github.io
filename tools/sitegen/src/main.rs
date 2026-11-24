mod compat;
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
    /// Build ps5slop's compatibility pages and compat.json from report issues.
    Compat {
        /// Issues as returned by GitHub's REST API.
        #[arg(long)]
        issues: PathBuf,
        #[arg(long, default_value = "content/ps5slop/compat")]
        out: PathBuf,
        #[arg(long, default_value = "static/ps5slop/compat.json")]
        json: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Docs { ps5slop, out } => docs::run(&ps5slop.join("docs/user"), &out),
        Command::Compat { issues, out, json } => compat::run(&issues, &out, &json),
    }
}
