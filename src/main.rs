mod dat;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{Context, Result};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    dat_file: std::path::PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    if let Err(e) = run(opt.dat_file) {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run(dat_file: PathBuf) -> Result<()> {
    let reader = File::open(&dat_file)
        .with_context(|| format!("Error opening file {}", dat_file.to_string_lossy()))?;
    let dat: dat::Mame = dat::parse(BufReader::new(reader))?;
    println!("{:#?}", dat);

    Ok(())
}
