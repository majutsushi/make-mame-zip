mod dat;
mod romdb;

use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use indicatif::ProgressIterator;
use structopt::StructOpt;
use zip::ZipArchive;

use crate::romdb::RomDb;

#[derive(Debug, StructOpt)]
#[structopt(about = "make a MAME game ZIP from a DAT file and romset")]
enum MakeMameZip {
    #[structopt(name = "create-db")]
    CreateDb {
        #[structopt(parse(from_os_str))]
        romset_dir: std::path::PathBuf,
    },
    #[structopt(name = "make-zip")]
    MakeZip {
        #[structopt(parse(from_os_str))]
        dat_file: std::path::PathBuf,
        game_name: String,
    },
}

fn main() {
    if let Err(e) = match MakeMameZip::from_args() {
        MakeMameZip::CreateDb { romset_dir } => create_db(romset_dir),
        MakeMameZip::MakeZip {
            dat_file,
            game_name,
        } => make_zip(dat_file, game_name),
    } {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn create_db(romset_dir: PathBuf) -> Result<()> {
    let romset_dir = fs::canonicalize(romset_dir)?;
    if !romset_dir.is_dir() {
        return Err(anyhow!("Not a directory: {}", romset_dir.to_string_lossy()));
    }

    println!("Reading directory '{}' ...", romset_dir.to_string_lossy());

    let db = RomDb::create(&dirs::data_local_dir().unwrap().join("mame-roms.db"))?;

    let read_err = || anyhow!("Error reading directory {}", romset_dir.to_string_lossy());
    let num_files = romset_dir.read_dir().with_context(read_err)?.count();
    for entry in romset_dir
        .read_dir()
        .with_context(read_err)?
        .progress_count(num_files as u64)
    {
        let entry = entry?;
        let reader = File::open(entry.path())
            .with_context(|| anyhow!("Error reading file {}", entry.path().to_string_lossy()))?;
        match ZipArchive::new(BufReader::new(reader)) {
            Ok(mut zip) => {
                for i in 0..zip.len() {
                    let zipfile = zip.by_index(i)?;
                    let file_name = zipfile.name();
                    let crc32 = zipfile.crc32();
                    db.add_rom(file_name, crc32, &entry.path())?;
                }
            }
            Err(_) => {
                eprintln!(
                    "Ignoring non-zip file: {}",
                    entry.file_name().to_string_lossy()
                )
            }
        };
    }

    Ok(())
}

fn make_zip(dat_file: PathBuf, _game_name: String) -> Result<()> {
    let reader = File::open(&dat_file)
        .with_context(|| format!("Error opening file {}", dat_file.to_string_lossy()))?;
    let dat: dat::Mame = dat::parse(BufReader::new(reader))?;
    println!("{:#?}", dat);

    Ok(())
}
