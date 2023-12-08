#![feature(async_closure)]
mod api;
mod ui;
use std::io;
use ui::interface;
use ui::{input::create_input_channel, interface::setup_terminal};
use rodio::{Sink, OutputStream};



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // test if the audio sink works
    test_sink()?;

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

/**
 * Code for testing if the audio sink can be owned by the program, because checking in the thread
 * owning the sink is pain.
 */
fn test_sink() -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    drop(sink);
    Ok(())
}
