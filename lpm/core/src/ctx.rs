use crate::open_core_db_connection;

use cli_parser::CliParser;
use db::SQL_NO_CALLBACK_FN;
use ehandle::{lpm::LpmError, MainError};
use min_sqlite3_sys::prelude::{Database, Operations};
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

    pub fn new_from_cli_parser(cli_parser: &CliParser) -> Result<Self, LpmError<MainError>> {
        Ok(Self {
            core_db: open_core_db_connection()?,
            force_yes: cli_parser.force_yes,
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

impl Drop for Ctx {
    fn drop(&mut self) {
        #[allow(clippy::disallowed_methods)]
        self.core_db
            .execute(
                String::from("PRAGMA journal_mode = DELETE;"),
                SQL_NO_CALLBACK_FN,
            )
            .unwrap();
    }
}
