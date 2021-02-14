use std::fs;

use serde::{Deserialize, Deserializer};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    dat_file: String,
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

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);

    let dat_string =
        fs::read_to_string(&opt.dat_file).expect(&format!("File {} not found", &opt.dat_file));

    let dat: Mame = quick_xml::de::from_str(&dat_string).unwrap_or_else(|err| panic!("{}", err));
    println!("{:#?}", dat);
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
