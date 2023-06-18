use crate::open_core_db_connection;

use ehandle::{lpm::LpmError, MainError};
use min_sqlite3_sys::prelude::Database;
use std::io::{self, Read, Write};

pub struct Ctx {
    pub core_db: Database,
    pub force_yes: bool,
}

impl Ctx {
    pub fn new() -> Result<Self, LpmError<MainError>> {
        Ok(Self {
            core_db: open_core_db_connection()?,
            force_yes: false,
        })
    }

    pub fn ask_for_confirmation(&self, q: &str) -> Result<bool, LpmError<MainError>> {
        if self.force_yes {
            return Ok(true);
        }

        let mut input = [0u8; 2];
        loop {
            print!("{q} [Y/n]:");
            io::stdout().flush()?;

            io::stdin().read_exact(&mut input)?;

            // Expect next char to be new line, so anything other than Y-y/N-n
            // will fail.
            if input[1] != b'\n' {
                io::stdin().read_line(&mut Default::default())?; // Clear remaining bytes
                continue;
            }

            match input[0] {
                b'y' | b'Y' => return Ok(true),
                b'n' | b'N' => return Ok(false),
                _ => {
                    io::stdin().read_line(&mut Default::default())?; // Clear remaining bytes
                    continue;
                }
            }
        }
    }
}
