use anyhow::Result;
use html_nomicon::cli::*;
use html_nomicon::processing::*;

fn main() -> Result<()> {

    let input = Input::get_with_timeout()?;

    let _documents = Documents::new(&input.path)?;

    Ok(())

}
