use clap::Parser;
use anyhow::Result;
use serde_json::from_str;
use std::{
    fs::read_to_string,
    io::{ Read, Stdin, stdin, },
    path::PathBuf,
    time::Duration,
    thread::spawn,
    sync::mpsc::{ RecvTimeoutError, channel, },
};
use crate::buffer::Buffer;




pub const TIMEOUT: Duration = Duration::from_secs(16);





#[derive(Debug,Parser)]
pub struct Cli {
    /// root of the project directory (./ indicates the current dir)
    pub path: PathBuf,
    /// optional json string to pass as an argument instead of passing via stdin
    #[arg(short,long,default_value=None)]
    pub json: Option<String>,
    /// optional file path to pass as an argument instead of passing via stdin
    #[arg(short,long,default_value=None)]
    pub file: Option<PathBuf>,
}

impl Cli{
    pub fn args() -> Self {
        Cli::parse()
    }
}





pub struct Input {
    pub path: PathBuf,
    pub json: Buffer,
}

impl Input {

    pub fn get() -> Result<Self> {
        let args = Cli::args();
        let path = args.path.clone();
        let json = match Data::new(args) {
            Data::String(string) => Data::string(&string),
            Data::File(path) => Data::file(path),
            Data::IO(stdin) => Data::io(stdin),
        }?;
        Ok(Self{path, json})
    }

    pub fn get_with_timeout() -> Result<Self> {
        let (data, timeout) = channel();
        spawn( move || data.send( Self::get() ));
        timeout.recv_timeout(TIMEOUT)?
    }

}





enum Data {
    String(String),
    File(PathBuf),
    IO(Stdin),
}

impl Data {
    fn new(args: Cli) -> Self {
        if let Some(string) = args.json {
            Self::String(string)
        }
        else if let Some(file) = args.file {
            Self::File(file)
        }
        else {
            Self::IO(stdin())
        }
    }

    fn string (s: &str) -> Result<Buffer> {
        Ok(from_str(s)?)
    }

    fn file (file: PathBuf) -> Result<Buffer> {
        Ok(from_str(&read_to_string(file)?)?)
    }

    fn io (mut stdin: Stdin) -> Result<Buffer> {
        let mut buffer = String::new();
        stdin.read_to_string(&mut buffer)?;
        Ok(from_str(&buffer)?)
    }

}
