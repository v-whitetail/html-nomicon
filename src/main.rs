use anyhow::Result;
use html_nomicon::cli::*;
use html_nomicon::processing::*;

fn main() -> Result<()> {

    let input = Input::get_with_timeout()?;

    let documents = Documents::new(&input.path)?;

    documents.process(&input.json)?;

    Ok(())

}
