use anyhow::Result;
use reedline_repl_rs::Repl;
use taotie::{get_callbacks, ReplCommand, ReplContext};

const HISTORY: usize = 1024;

fn main() -> Result<()> {
    let ctx = ReplContext::new();
    let callbacks = get_callbacks();

    let history_file = dirs::home_dir().unwrap().join(".taotie_history");

    let mut repl = Repl::new(ctx)
        .with_history(history_file, HISTORY)
        .with_banner("Welcome to Taotie, your dataset exploration REPL!")
        .with_derived::<ReplCommand>(callbacks);

    repl.run()?;
    Ok(())
}
