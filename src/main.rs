use anyhow::Result;
use html_nomicon::cli::*;
use html_nomicon::nomming::*;

fn main() -> Result<()> {

    let input = Input::get_with_timeout()?;

    let documents = Documents::new(&input.path)?;

    let templates = Dispatch::new(&input.json, &documents)
        .with_log("dispatch.log".into())
        .read_all()?
        .parse_all();

    Ok(())

}
