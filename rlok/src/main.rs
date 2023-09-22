use color_eyre::eyre::Result;
use rlok_lib::interpreter::Interpreter;
use std::env;

fn main() -> Result<()> {
    color_eyre::install()?;
    Interpreter::build().start(env::args().collect())?;
    Ok(())
}
