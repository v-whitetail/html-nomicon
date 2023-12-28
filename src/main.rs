use anyhow::Result;
use html_nomicon::cli::*;
use html_nomicon::processing::*;

fn main() -> Result<()> {

    let input = Input::get_with_timeout()?;

    let documents = Documents::new(&input.path)?;

    let raw_templates = RawTemplates::new(&input.json, &documents)?;

    let _parsed_templates = ParsedTemplates::new(&input.json, &raw_templates)?;

//    println!("{parsed_templates:#?}");

    Ok(())

}
