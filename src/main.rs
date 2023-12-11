#![allow(unused, dead_code)]

use anyhow::Result;
use html_nomicon::cli::*;

fn main() -> Result<()> {

    let input = Input::get_with_timeout()?;

    let (path, json) = (input.path, input.json);

    println!("{json:#?}");

    Ok(())

}
