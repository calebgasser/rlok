use color_eyre::eyre::Result;
use rlok_lib::interpreter::Interpreter;
use std::env;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::ERROR)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    color_eyre::install()?;
    Interpreter::build().start(env::args().collect())?;
    Ok(())
}
