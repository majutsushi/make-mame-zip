use std::io::{BufReader, Read};

use anyhow::Result;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, PartialEq)]
pub struct Mame {
    #[serde(rename = "game", default)]
    pub games: Vec<Game>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Game {
    pub name: String,
    pub description: String,
    #[serde(rename = "rom", default)]
    pub roms: Vec<Rom>,
    #[serde(rename = "disk", default)]
    pub disks: Vec<Disk>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Rom {
    pub name: String,
    #[serde(deserialize_with = "de_crc", default)]
    pub crc: Option<u32>,
    pub sha1: Option<String>,
    #[serde(deserialize_with = "de_dispose", default = "default_dispose")]
    pub dispose: bool,
    #[serde(default)]
    pub status: Status,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Disk {
    pub name: String,
    pub sha1: String,
    pub md5: String,
    pub region: String,
    pub index: u8,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    BadDump,
    #[default]
    Good,
    NoDump,
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
