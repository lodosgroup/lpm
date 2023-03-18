use std::io::{self, Write};

const LOGGER_NAME: &str = "lpm";

pub enum OutputMode {
    SUCCESS,
    INFO,
    ERROR,
    WARNING,
    DEBUG,
}

impl OutputMode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::SUCCESS => "SUCCESS",
            Self::INFO => "INFO",
            Self::ERROR => "ERROR",
            Self::WARNING => "WARNING",
            Self::DEBUG => "DEBUG",
        }
    }

    /// Useful only for `WARNING` and `ERROR` modes
    pub fn colored_log_format(&self) -> &str {
        match self {
            Self::SUCCESS => "\x1b[0;32m",
            Self::INFO => "\x1b[0;39m",
            Self::ERROR => "\x1b[0;31m",
            Self::WARNING => "\x1b[0;33m",
            Self::DEBUG => "\x1b[0;39m",
        }
    }

    pub fn colored_and_bold_prefix_format(&self) -> &str {
        match self {
            Self::SUCCESS => "\x1b[1;32m",
            Self::INFO => "\x1b[1;34m",
            Self::ERROR => "\x1b[1;31m",
            Self::WARNING => "\x1b[1;33m",
            Self::DEBUG => "\x1b[1;95m",
        }
    }

    /// Returns default ansi format
    pub fn default_format(&self) -> &str {
        "\x1b[0;39m"
    }
}

pub fn build_log(mode: OutputMode, log: String) -> String {
    let log_prefix = format!(
        "{}[{}{}{}]:",
        LOGGER_NAME,
        mode.colored_and_bold_prefix_format(),
        mode.as_str(),
        mode.default_format(),
    );

    format!(
        "{} {}{}{}\n",
        log_prefix,
        mode.colored_log_format(),
        log,
        mode.default_format(),
    )
}

pub fn log_to_stderr(log: &[u8]) {
    if io::stderr().write_all(log).is_err() {
        log_to_stderr(
            build_log(OutputMode::ERROR, String::from("writing to stderr failed")).as_bytes(),
        );
        std::process::exit(101);
    }
}

pub fn log_to_stdout(log: &[u8]) {
    if io::stdout().write_all(log).is_err() {
        log_to_stderr(
            build_log(OutputMode::ERROR, String::from("writing to stdout failed")).as_bytes(),
        );
        std::process::exit(101);
    }
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug {
    ($log: expr, $($args: tt)+) => {
        term::logger::log_to_stdout(term::logger::build_log(term::logger::OutputMode::DEBUG, format!($log, $($args)+)).as_bytes());

    };
    ($log: expr) => {
        term::logger::log_to_stdout(term::logger::build_log(term::logger::OutputMode::DEBUG, format!($log)).as_bytes());
    }
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
    ($log: expr, $($args: tt)+) => {};
    ($log: expr) => {};
}

#[macro_export]
macro_rules! success {
    ($log: expr, $($args: tt)+) => {
        term::logger::log_to_stdout(term::logger::build_log(term::logger::OutputMode::SUCCESS, format!($log, $($args)+)).as_bytes());

    };
    ($log: expr) => {
        term::logger::log_to_stdout(term::logger::build_log(term::logger::OutputMode::SUCCESS, format!($log)).as_bytes());
    }
}

#[macro_export]
macro_rules! info {
    ($log: expr, $($args: tt)+) => {
        term::logger::log_to_stdout(term::logger::build_log(term::logger::OutputMode::INFO, format!($log, $($args)+)).as_bytes());

    };
    ($log: expr) => {
        term::logger::log_to_stdout(term::logger::build_log(term::logger::OutputMode::INFO, format!($log)).as_bytes());
    }
}

#[macro_export]
macro_rules! error {
    ($log: expr, $($args: tt)+) => {
        term::logger::log_to_stderr(term::logger::build_log(term::logger::OutputMode::ERROR, format!($log, $($args)+)).as_bytes());

    };
    ($log: expr) => {
        term::logger::log_to_stderr(term::logger::build_log(term::logger::OutputMode::ERROR, format!($log)).as_bytes());
    }
}

#[macro_export]
macro_rules! warning {
    ($log: expr, $($args: tt)+) => {
        term::logger::log_to_stdout(term::logger::build_log(term::logger::OutputMode::WARNING, format!($log, $($args)+)).as_bytes());

    };
    ($log: expr) => {
        term::logger::log_to_stdout(term::logger::build_log(term::logger::OutputMode::WARNING, format!($log)).as_bytes());
    }
}
