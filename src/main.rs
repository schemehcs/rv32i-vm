mod inst;
mod vm;

use anyhow::Ok;
use clap::Parser;
use std::path::PathBuf;
use vm::Machine;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    /// program file
    #[arg(value_name = "FILE")]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let bin = std::fs::read(cli.path)?;
    let mut machine = Machine::new(2 << 20);
    machine.load_bin(&bin);
    machine.run()?;
    machine.debug();
    Ok(())
}
