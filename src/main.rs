use largo::cli::Cli;
use largo_core::Result;

use clap::Parser;

fn main() -> Result<()> {
    Cli::parse().execute()
}
