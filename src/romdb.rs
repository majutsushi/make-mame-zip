use std::{fs, os::unix::prelude::OsStrExt, path::Path};

use anyhow::Result;
use rusqlite::{params, Connection};

pub struct RomDb {
    conn: Connection,
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
}
