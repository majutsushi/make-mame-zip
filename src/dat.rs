use std::io::{BufReader, Read};

use anyhow::Result;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Mame {
    #[serde(rename = "game", default)]
    games: Vec<Game>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Game {
    name: String,
    description: String,
    #[serde(rename = "rom", default)]
    roms: Vec<Rom>,
    #[serde(rename = "disk", default)]
    disks: Vec<Disk>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Rom {
    name: String,
    #[serde(deserialize_with = "de_crc", default)]
    crc: Option<u32>,
    sha1: Option<String>,
    #[serde(deserialize_with = "de_dispose", default = "default_dispose")]
    dispose: bool,
    #[serde(default)]
    status: Status,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Disk {
    name: String,
    sha1: String,
    md5: String,
    region: String,
    index: u8,
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

pub fn parse<T: Read>(reader: T) -> Result<Mame> {
    quick_xml::de::from_reader(BufReader::new(reader)).map_err(|e| e.into())
}

fn de_crc<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Option<String> = Option::deserialize(deserializer)?;
    if let Some(val) = val {
        return Ok(Some(
            u32::from_str_radix(&val, 16).map_err(serde::de::Error::custom)?,
        ));
    }

    Ok(None)
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
