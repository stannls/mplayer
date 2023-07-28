#![feature(async_closure)]
mod api;
mod ui;
use std::io;
use ui::interface;
use ui::{input::create_input_channel, interface::setup_terminal};



#[tokio::main]
async fn main() -> Result<(), io::Error> {
    // setup input handler
    let rx = create_input_channel();

    // setup terminal
    let mut terminal = setup_terminal()?;

    // render the main interface
    interface::render_interface(&mut terminal, rx).await;

    // restore terminal
    interface::restore_terminal(&mut terminal)?;

    Ok(())
}
