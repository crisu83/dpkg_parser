//! # Debian package parser
//!
//! A library for parsing files that describe Debian packages.

pub mod ast;
pub mod parser;

use std::{error, fmt, fs, io};

/// Runs the application.
pub fn run(config: Config) -> Result<(), Box<dyn error::Error>> {
    let contents = read_file(&config.file_path[..])?;

    let parsed = parser::parse(contents.trim())?;

    println!("parsed: {:#?}", parsed);

    Ok(())
}

/// Describes an error that may occur when reading a file.
#[derive(Debug, Clone)]
pub enum ReadFileError {
    PathDoesNotExist,
    PermissionDenied,
}

impl ReadFileError {
    fn from_io_error(err: io::Error) -> ReadFileError {
        match err.kind() {
            io::ErrorKind::NotFound => ReadFileError::PathDoesNotExist,
            io::ErrorKind::PermissionDenied => ReadFileError::PermissionDenied,
            _ => panic!("could not map error type"),
        }
    }
}

impl error::Error for ReadFileError {}

impl fmt::Display for ReadFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadFileError::PathDoesNotExist => write!(f, "path does not exist"),
            ReadFileError::PermissionDenied => write!(f, "permission denied"),
        }
    }
}

/// Reads the contents of a file to a string.
fn read_file(file_path: &str) -> Result<String, ReadFileError> {
    fs::read_to_string(file_path).map_err(ReadFileError::from_io_error)
}

/// Describes the application's configuration.
pub struct Config {
    pub file_path: String,
}

/// Describes an error that may occur when building the configuration.
#[derive(Debug, Clone)]
pub enum BuildConfigError {
    NoFilePath,
}

impl error::Error for BuildConfigError {}

impl fmt::Display for BuildConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildConfigError::NoFilePath => write!(f, "no file path provided"),
        }
    }
}

/// Describes the application configuration.
impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, BuildConfigError> {
        args.next(); // skip the first argument

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err(BuildConfigError::NoFilePath),
        };

        Ok(Config { file_path })
    }
}
