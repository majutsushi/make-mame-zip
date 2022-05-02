use std::{
    ffi::OsStr,
    fs,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};

pub struct RomDb {
    conn: Connection,
}

#[derive(Debug)]
pub struct RomInfo {
    pub name: String,
    pub zipname: String,
    pub crc32: u32,
    pub path: PathBuf,
}

impl RomDb {
    pub fn create(path: &Path) -> Result<Self> {
        if path.exists() {
            fs::remove_file(path)?;
        }

        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE roms (
                    name      TEXT NOT NULL,
                    zipname   TEXT NOT NULL,
                    crc32     INTEGER NOT NULL,
                    path      BLOB NOT NULL
                )",
            [],
        )?;

        Ok(RomDb { conn })
    }

    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        Ok(RomDb { conn })
    }

    pub fn add_rom(&self, zipname: &str, crc32: u32, path: &Path) -> Result<()> {
        let name = match zipname.rsplit_once('/') {
            Some((_, name)) => name,
            None => zipname,
        };
        self.conn.execute(
            "INSERT INTO roms (name, zipname, crc32, path) VALUES (?1, ?2, ?3, ?4)",
            params![name, zipname, crc32, path.as_os_str().as_bytes()],
        )?;

        Ok(())
    }

    pub fn find_rom(&self, name: &str, crc32: u32) -> Result<RomInfo> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, zipname, crc32, path FROM roms WHERE name = ? AND crc32 = ?")?;
        let rominfo_iter = stmt.query_map(params![name, crc32], |row| {
            Ok(RomInfo {
                name: row.get(0)?,
                zipname: row.get(1)?,
                crc32: row.get(2)?,
                path: PathBuf::from(OsStr::from_bytes(row.get_ref_unwrap(3).as_bytes()?)),
            })
        })?;

        // Always return the first result, if there are multiple they should be identical
        return match rominfo_iter.take(1).next() {
            Some(rominfo) => rominfo.map_err(|err| err.into()),
            None => Err(anyhow!("No ROM found matching name {}", name)),
        };
    }
}
