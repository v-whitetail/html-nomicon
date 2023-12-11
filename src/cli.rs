use clap::Parser;
use anyhow::Result;
use serde_json::{
    Map,
    Value,
    from_str,
};
use std::{
    fs::read_to_string,
    io::{
        Read,
        Stdin,
        stdin,
    },
    path::PathBuf,
    time::Duration,
    sync::mpsc::{
        channel,
        RecvTimeoutError,
    },
    thread::spawn,
};




pub const TIMEOUT: Duration = Duration::from_secs(16);





#[derive(Debug,Parser)]
pub struct Cli {
    /// root of the project directory (./ indicates the current dir)
    pub path: PathBuf,
    /// optional data to pass as an argument indead of passing via stdin
    pub data: Option<PathBuf>,
}

impl Cli{
    pub fn args() -> Self {
        Cli::parse()
    }
}





pub struct Input {
    pub path: PathBuf,
    pub json: Map<String, Value>
}





pub enum Data {
    File(PathBuf),
    IO(Stdin),
}

impl Data {
    pub fn new(data: Option<PathBuf>) -> Self {
        if let Some(file) = data {
            Self::File(file)
        }
        else {
            Self::IO(stdin())
        }
    }

    pub fn get() -> Result<Input> {
        let args = Cli::args();
        let path = args.path;
        let json = match Self::new(args.data) {
            Self::File(file) => Self::file(file),
            Self::IO(stdin) => Self::io(stdin),
        }?;
        Ok(Input{path, json})
    }

    pub fn get_with_timeout() -> Result<Input> {
        let (data, timeout) = channel();
        spawn(
            move || {
                data.send( Self::get() );
            });
        timeout.recv_timeout(TIMEOUT)?
    }

    fn file (file: PathBuf) -> Result<Map<String, Value>> {
        Ok(from_str(&read_to_string(file)?)?)
    }

    fn io (mut stdin: Stdin) -> Result<Map<String, Value>> {
        let mut buffer = String::new();
        stdin.read_to_string(&mut buffer)?;
        Ok(from_str(&buffer)?)
    }

}
