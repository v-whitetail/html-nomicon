#![allow(unused, dead_code)]

use anyhow::Result;
use html_nomicon::cli::*;
use html_nomicon::nomming::*;

fn main() -> Result<()> {

    let input = Input::get_with_timeout()?;

    let (path, json) = (input.path, input.json);

    let buffer = Buffer::new(json.clone())?;

    let reports = buffer.listed_reports()?;

    println!("{reports:#?}");

    Ok(())

}
