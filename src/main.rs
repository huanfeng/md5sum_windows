use std::fs::File;
use std::io::{self, Read, BufRead, BufReader};
use std::path::PathBuf;
use clap::{Parser, ArgAction};
use md5::{Md5, Digest};
use anyhow::{Result, Context};

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

fn compute_md5<R: Read>(mut reader: R) -> Result<String> {
    let mut hasher = Md5::new();
    let mut buffer = [0; 1024];
    
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    
    Ok(format!("{:x}", hasher.finalize()))
}

fn process_file(path: &PathBuf, binary: bool) -> Result<()> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let md5sum = compute_md5(file)?;
    let mode = if binary { "*" } else { " " };
    println!("{}{}{}", md5sum, mode, path.display());
    Ok(())
}

fn verify_checksums(path: &PathBuf) -> Result<()> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() != 2 {
            eprintln!("{}: improperly formatted checksum line", path.display());
            continue;
        }

        let (expected_sum, filename) = (parts[0], parts[1]);
        let file_path = PathBuf::from(filename);
        
        if let Ok(file) = File::open(&file_path) {
            let computed_sum = compute_md5(file)?;
            if computed_sum == expected_sum {
                println!("{}: OK", filename);
            } else {
                println!("{}: FAILED", filename);
            }
        } else {
            eprintln!("{}: No such file or directory", filename);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.check {
        for path in &cli.files {
            if let Err(err) = verify_checksums(path) {
                eprintln!("{}: {}", path.display(), err);
            }
        }
    } else {
        if cli.files.is_empty() {
            // Read from stdin when no files are specified
            let stdin = io::stdin();
            let md5sum = compute_md5(stdin.lock())?;
            println!("{}", md5sum);
        } else {
            for path in &cli.files {
                if let Err(err) = process_file(path, cli.binary) {
                    eprintln!("{}", err);
                }
            }
        }
    }

    Ok(())
}
