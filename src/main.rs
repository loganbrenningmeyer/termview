use std::io::{self};
use rustyline::{error::ReadlineError, DefaultEditor};

use termview::repl::{
    AppControl, 
    Command, 
    TermviewApp
};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut editor = DefaultEditor::new()?;
    
    let mut app = TermviewApp::default();

    let mut output = io::stdout().lock();

    loop {
        // -------------------------
        // Loop parsing command inputs from:
        // # termview> {command}
        // -------------------------
        match editor.readline("termview> ") {
            Ok(line) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                editor.add_history_entry(line)?;

                let command = match Command::parse(line) {
                    Ok(command) => command,
                    Err(error) => {
                        eprintln!("{error}");
                        continue;
                    }
                };

                if app.execute(command, &mut output)? == AppControl::Quit {
                    break;
                }
            }

            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(error) => return Err(error.into()),
        }
    }

    Ok(())
}