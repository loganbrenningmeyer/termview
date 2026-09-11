use std::io::{self};
use termview::repl::TermviewApp;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = TermviewApp::default();
    let mut output = io::stdout().lock();

    app.run(&mut output)?;

    Ok(())
}