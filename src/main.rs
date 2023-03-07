use std::{io, thread, time::Duration, sync::mpsc, time::Instant};
use tui::{backend::CrosstermBackend,
    Terminal,
    layout::{Layout, Direction, Constraint},
    style::{Style, Modifier},
    widgets::{Widget, Block, Borders, Paragraph}};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

enum Event<I> {
    Input(I),
    Tick,
}

enum State{
    Default,
    Input,
}

fn main() -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;

    // setup input handler
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let crossterm::event::Event::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    let mut current_state = State::Default;
    let mut search = String::from("");
    loop{
        terminal.draw(|f| {
            let size = f.size();
            
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                    ]
                    .as_ref(),
                )
                .split(size);

            let block = Block::default()
                .title("mplayer")
                .borders(Borders::ALL);
            f.render_widget(block, size);

            let searchbar_content = if matches!(current_state, State::Default) {
                "Enter Search Term [s]"
            } else{
                search.as_str()
            };
            let searchbar = Paragraph::new(searchbar_content)
                .style(Style::default().add_modifier(Modifier::ITALIC))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Search")
                );
            let below_search_layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(30),
                        Constraint::Min(2),
                    ]
                    .as_ref(),
                )
                .split(main_layout[1]);
            let side_menu = Paragraph::new("Albums")
                    .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Libary")
                    );

            let main_content = Paragraph::new("Lorem ipsum dolor sit amet.")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Welcome")
                    );
            f.render_widget(searchbar, main_layout[0]);
            f.render_widget(main_content, below_search_layout[1]);
            f.render_widget(side_menu, below_search_layout[0])
        })?;
        if matches!(current_state, State::Default){
                match rx.recv().unwrap() {
                    Event::Input(event) => match event.code {
                        KeyCode::Char('q') => {
                            break;
                        },
                        KeyCode::Char('s') => {
                            current_state = State::Input;
                        },
                    _ => {}
                    }
                    Event::Tick => {}
                }
        } else {
            match rx.recv().unwrap() {
                Event::Input(event) => match event.code {
                    KeyCode::Char(c) => search.push(c),
                    KeyCode::Backspace => {
                        search.pop();
                    },
                    KeyCode::Esc => {
                        current_state = State::Default;
                        search = String::from("");
                    },
                    _ => {}
            }
                _ => {}
            }
    }
    }
    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

