use crate::open_core_db_connection;

use ehandle::{lpm::LpmError, MainError};
use min_sqlite3_sys::prelude::Database;
use std::io::{self, Write};

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

        loop {
            let mut input = String::new();

            print!(
                "{} [Y/n]: ",
                logger::build_log(logger::OutputMode::QUESTION, q)
            );

            io::stdout().flush()?;

            io::stdin().read_line(&mut input)?;

            // Expect next char to be new line, so anything other than Y-y/N-n
            // will fail.
            if input.len() > 2 {
                continue;
            }

            if input.to_lowercase().starts_with("y\n") {
                return Ok(true);
            }

            if input.to_lowercase().starts_with("n\n") {
                return Ok(false);
            }
        }
    }
}
