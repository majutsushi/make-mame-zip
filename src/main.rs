mod dat;
mod romdb;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use indicatif::ProgressIterator;
use itertools::{Either, Itertools};
use lazy_static::lazy_static;
use structopt::StructOpt;
use zip::{ZipArchive, ZipWriter};

use crate::{dat::Status, romdb::RomDb};

#[derive(Debug, StructOpt)]
#[structopt(about = "make a MAME game ZIP from a DAT file and romset")]
enum MakeMameZip {
    #[structopt(name = "create-db")]
    CreateDb {
        #[structopt(parse(from_os_str), required = true)]
        romset_dirs: Vec<std::path::PathBuf>,
    },
    #[structopt(name = "make-zip")]
    MakeZip {
        #[structopt(parse(from_os_str))]
        dat_file: std::path::PathBuf,
        game_name: String,
    },
}

lazy_static! {
    static ref DB_PATH: PathBuf = dirs::data_local_dir().unwrap().join("mame-roms.db");
}

fn main() {
    if let Err(e) = match MakeMameZip::from_args() {
        MakeMameZip::CreateDb { romset_dirs } => create_db(romset_dirs),
        MakeMameZip::MakeZip {
            dat_file,
            game_name,
        } => make_zip(dat_file, game_name),
    } {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn create_db(romset_dirs: Vec<PathBuf>) -> Result<()> {
    let db = RomDb::create(&DB_PATH)?;

    for romset_dir in &romset_dirs {
        add_romset_dir(&db, romset_dir)?;
    }

    Ok(())
}

fn add_romset_dir(db: &RomDb, romset_dir: &Path) -> Result<()> {
    let romset_dir = fs::canonicalize(romset_dir)?;
    if !romset_dir.is_dir() {
        return Err(anyhow!("Not a directory: {}", romset_dir.to_string_lossy()));
    }

    println!("Reading directory '{}' ...", romset_dir.to_string_lossy());

    let read_err = || anyhow!("Error reading directory {}", romset_dir.to_string_lossy());
    let num_files = romset_dir.read_dir().with_context(read_err)?.count();
    'outer: for entry in romset_dir
        .read_dir()
        .with_context(read_err)?
        .progress_count(num_files as u64)
    {
        let entry = entry?;
        let reader = File::open(entry.path())
            .with_context(|| anyhow!("Error reading file '{}'", entry.path().to_string_lossy()))?;
        match ZipArchive::new(BufReader::new(reader)) {
            Ok(mut zip) => {
                for i in 0..zip.len() {
                    match zip.by_index(i) {
                        Ok(zipfile) => {
                            let file_name = zipfile.name();
                            let crc32 = zipfile.crc32();
                            db.add_rom(file_name, crc32, &entry.path())?;
                        }
                        Err(e) => {
                            eprintln!(
                                "Error reading ZIP file '{}': {}",
                                entry.path().to_string_lossy(),
                                e
                            );
                            continue 'outer;
                        }
                    };
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

fn make_zip(dat_file: PathBuf, game_name: String) -> Result<()> {
    let reader = File::open(&dat_file)
        .with_context(|| format!("Error opening file {}", dat_file.to_string_lossy()))?;
    let dat: dat::Mame = dat::parse(BufReader::new(reader))?;

    let game = match dat.games.iter().find(|&game| game.name == game_name) {
        Some(game) => game,
        None => return Err(anyhow!("Game not found in DAT file: {}", game_name)),
    };
    let bad_roms = game
        .roms
        .iter()
        .filter(|rom| rom.status != Status::Good)
        .map(|rom| rom.name.clone())
        .collect::<Vec<_>>();
    if !bad_roms.is_empty() {
        return Err(anyhow!("No good dump for ROMs: {}", bad_roms.join(", ")));
    }

    let db = RomDb::open(&DB_PATH)?;

    let (rom_infos, not_found): (HashMap<_, _>, Vec<_>) = game
        .roms
        .iter()
        // Unwrapping the CRC is safe since it will always be present for good dumps
        .map(|dat_rom| {
            db.find_rom(dat_rom.crc.unwrap())
                .map(|rom_info| (dat_rom.name.to_owned(), rom_info))
        })
        .partition_map(|r| match r {
            Ok(rom_tuple) => Either::Left(rom_tuple),
            Err(e) => Either::Right(e),
        });
    if !not_found.is_empty() {
        return Err(anyhow!(
            "Error looking up ROMs:\n{}",
            not_found.iter().join("\n")
        ));
    }

    let file_out = File::create(format!("{}.zip", game_name))?;
    let mut zip_out = ZipWriter::new(BufWriter::new(file_out));
    for (rom_name, rom_info) in rom_infos {
        let reader = File::open(&rom_info.path)
            .with_context(|| anyhow!("Error reading file {}", rom_info.path.to_string_lossy()))?;
        let mut zip_in = ZipArchive::new(BufReader::new(reader))?;

        // We need to find the correct ROM by CRC as the name may be different
        let mut filename = None;
        for i in 0..zip_in.len() {
            match zip_in.by_index(i) {
                Ok(zipfile) => {
                    if zipfile.crc32() == rom_info.crc32 {
                        filename = Some(zipfile.name().to_owned());
                        break;
                    }
                }
                Err(e) => {
                    return Err(anyhow!(
                        "Error reading ZIP file '{}': {}",
                        rom_info.path.to_string_lossy(),
                        e
                    ));
                }
            };
        }

        match filename {
            Some(filename) => {
                let file = zip_in.by_name(&filename)?;
                zip_out.raw_copy_file_rename(file, &rom_name)?;
            }
            None => {
                return Err(anyhow!(
                    "ROM with CRC {} not found in ZIP file '{}'",
                    rom_info.crc32,
                    rom_info.path.to_string_lossy()
                ))
            }
        }
    }

    Ok(())
}
