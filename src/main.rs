use anyhow::Result;
use html_nomicon::cli::*;
//use html_nomicon::nomming::*;

fn main() -> Result<()> {

    let input = Input::get_with_timeout()?;

    let (_path, json) = (input.path, input.json);

    println!("{json:#?}");

    Ok(())

}
