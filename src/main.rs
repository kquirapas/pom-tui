use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use chrono::{self, Utc};
// use unicode_width::UnicodeWidthStr;

enum Modes {
    Input, Running }

struct App {
    time: i64,
    elapsed: i64,
    mode: Modes,
}

impl Default for App {
    fn default() -> Self {
        App {
            time: 0,
            elapsed: 0,
            mode: Modes::Input,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // setup backend
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app state
    let app = App::default();
    let result = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;

    if let Err(err) = result {
        println!("{:?}", err);
    };

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut start = Utc::now().time();
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // non-blocking event read
        if poll(Duration::from_millis(100))? {
            if let Event::Key(key) = read()? {
                match app.mode {
                    Modes::Input => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Up => app.time += 1,
                        KeyCode::Down => if app.time != 0 {
                            app.time -= 1;
                        },
                        KeyCode::Enter => {
                            app.mode = Modes::Running;
                            start = Utc::now().time();
                        },
                        _ => {}
                    },
                    Modes::Running => if key.code == KeyCode::Esc {
                        app.elapsed = 0;
                        app.mode = Modes::Input;
                    }
                }
            }
        }

        if let Modes::Running = app.mode {
            app.elapsed = (Utc::now().time() - start).num_seconds();
            if (app.elapsed >= app.time) {
                app.elapsed = app.time;
            }
        }
    }
}

fn log<B: Backend>(f: &mut Frame<B>, msg: String, area: Rect) {
    let mut message_text = Text::from(Span::raw(msg));
    message_text.patch_style(Style::default().fg(Color::Red));

    let message_widget = Paragraph::new(message_text)
        .block(Block::default().title("Error Message"));

    f.render_widget(message_widget, area);
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(3)
            ].as_ref()
        )
        .split(f.size());


    let time_left = app.time - app.elapsed;

    let (msg, style) = match app.mode {
        Modes::Input => (
            Span::raw(time_left.to_string()),
            Style::default()
        ),
        Modes::Running => {
            let color = if time_left == 0 { Color::Red } else { Color::Green };

            (
                Span::styled(time_left.to_string(), Style::default().fg(color)),
                Style::default().add_modifier(Modifier::RAPID_BLINK)
            )
        }
    };

    let instruction_widget = Paragraph::new(Text::from(Span::raw(match app.mode {
        Modes::Input => "[ q ] to quit, [ ^ ] inc time, [ v ] dec time, [ enter ] to start time",
        Modes::Running => "[ esc ] to change time"
    })));
    f.render_widget(instruction_widget, chunks[0]);

    let mut time_text = Text::from(msg);
    time_text.patch_style(style);

    let time_widget = Paragraph::new(time_text)
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(time_widget, chunks[1]);
}
