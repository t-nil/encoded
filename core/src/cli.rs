use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    pub encoded_path: PathBuf,
    pub raw_path: Option<PathBuf>,
}
