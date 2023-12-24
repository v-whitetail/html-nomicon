use anyhow::Result;
use html_nomicon::cli::*;
use html_nomicon::nomming::*;

fn main() -> Result<()> {

    let input = Input::get_with_timeout()?;

    let documents = Documents::new(&input.path)?;

    let _templates = Dispatch::new(&input.json, &documents)
        .log("dispatch.log".into())
        .read_all()?;

    Ok(())

}
