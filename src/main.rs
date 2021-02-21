use std::fs;
use std::io::BufReader;

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    dat_file: std::path::PathBuf,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Status {
    BadDump,
    Good,
    NoDump,
}
impl Default for Status {
    fn default() -> Self {
        Status::Good
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct Rom {
    name: String,
    crc: Option<String>,
    sha1: Option<String>,
    #[serde(deserialize_with = "de_dispose", default = "default_dispose")]
    dispose: bool,
    #[serde(default)]
    status: Status,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Game {
    name: String,
    description: String,
    #[serde(rename = "rom", default)]
    roms: Vec<Rom>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Mame {
    #[serde(rename = "game", default)]
    games: Vec<Game>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    println!("{:#?}", opt);

    let reader = fs::File::open(&opt.dat_file)
        .with_context(|| format!("Error opening file {}", &opt.dat_file.to_string_lossy()))?;
    let dat: Mame = quick_xml::de::from_reader(BufReader::new(reader))?;
    println!("{:#?}", dat);

    Ok(())
}

fn default_dispose() -> bool {
    false
}

fn de_dispose<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let val = String::deserialize(deserializer)?;
    match val.as_ref() {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Ok(default_dispose()),
    }
}
