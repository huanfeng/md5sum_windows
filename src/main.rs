use std::io;
use std::path::PathBuf;
use clap::{Parser, ArgAction};
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Read in binary mode
    #[arg(short = 'b', long, action = ArgAction::SetTrue)]
    binary: bool,

    /// Read checksums from the FILEs and check them
    #[arg(short = 'c', long, action = ArgAction::SetTrue)]
    check: bool,

    /// Create a BSD-style checksum
    #[arg(long, action = ArgAction::SetTrue)]
    tag: bool,

    /// Read in text mode (default)
    #[arg(short = 't', long, action = ArgAction::SetTrue)]
    text: bool,

    /// Files to process
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.check {
        for path in &cli.files {
            if let Err(err) = md5sum::verify_checksums(path) {
                eprintln!("{}: {}", path.display(), err);
            }
        }
    } else {
        if cli.files.is_empty() {
            // Read from stdin when no files are specified
            let stdin = io::stdin();
            let md5sum = md5sum::compute_md5(stdin.lock())?;
            println!("{}", md5sum);
        } else {
            for path in &cli.files {
                if let Err(err) = md5sum::process_file(path, cli.binary) {
                    eprintln!("{}", err);
                }
            }
        }
    }
    Ok(())
}
