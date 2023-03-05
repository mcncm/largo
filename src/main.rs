use largo::cli::Cli;
use largo::Result;

use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let res = cli.execute();
    res
}
