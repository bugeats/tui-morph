use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Paragraph};

use tui_morph::backend::{MorphBackend, MorphConfig};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;

    let result = run();

    disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}

fn run() -> io::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let config = MorphConfig {
        duration: Duration::from_millis(300),
        ..MorphConfig::default()
    };
    let mut terminal = MorphBackend::wrap(backend, config)?;

    let scenes: &[fn(&mut Frame)] = &[scene_a, scene_b, scene_c, scene_d];
    let mut current = 0;

    terminal.draw(|f| scenes[current](f))?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,

                KeyCode::Right | KeyCode::Char(' ') | KeyCode::Enter => {
                    current = (current + 1) % scenes.len();
                }

                KeyCode::Left => {
                    current = (current + scenes.len() - 1) % scenes.len();
                }

                _ => continue,
            }

            terminal.draw(|f| scenes[current](f))?;
        }
    }

    Ok(())
}

fn header(f: &mut Frame, area: Rect) {
    f.render_widget(
        Paragraph::new("tui-morph  [←/→ cycle scenes]  [q quit]")
            .style(Style::new().fg(Color::DarkGray)),
        area,
    );
}

fn split_main(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area)
}

fn scene_a(f: &mut Frame) {
    let chunks = split_main(f.area());
    header(f, chunks[0]);

    let cols = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Min(0),
    ])
    .split(chunks[1]);

    f.render_widget(
        Paragraph::new("Hello, world!")
            .block(Block::bordered().title(" A: Red Left "))
            .style(Style::new().fg(Color::Rgb(255, 80, 80)).bg(Color::Rgb(40, 0, 0))),
        cols[0],
    );
}

fn scene_b(f: &mut Frame) {
    let chunks = split_main(f.area());
    header(f, chunks[0]);

    let cols = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Percentage(40),
    ])
    .split(chunks[1]);

    f.render_widget(
        Paragraph::new("Hello, world!")
            .block(Block::bordered().title(" B: Blue Right "))
            .style(Style::new().fg(Color::Rgb(80, 120, 255)).bg(Color::Rgb(0, 0, 40))),
        cols[1],
    );
}

fn scene_c(f: &mut Frame) {
    let chunks = split_main(f.area());
    header(f, chunks[0]);

    let cols = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(chunks[1]);

    f.render_widget(
        Paragraph::new("Left panel")
            .block(Block::bordered().title(" C: Green "))
            .style(Style::new().fg(Color::Rgb(80, 255, 80)).bg(Color::Rgb(0, 40, 0))),
        cols[0],
    );

    f.render_widget(
        Paragraph::new("Right panel")
            .block(Block::bordered().title(" Details "))
            .style(Style::new().fg(Color::Rgb(255, 200, 80)).bg(Color::Rgb(40, 30, 0))),
        cols[1],
    );
}

fn scene_d(f: &mut Frame) {
    let chunks = split_main(f.area());
    header(f, chunks[0]);

    let h = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .split(chunks[1]);

    let v = Layout::vertical([
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .split(h[1]);

    f.render_widget(
        Paragraph::new("Centered content\nwith multiple lines")
            .block(Block::bordered().title(" D: Purple Center "))
            .style(Style::new().fg(Color::Rgb(200, 80, 255)).bg(Color::Rgb(30, 0, 40)))
            .alignment(Alignment::Center),
        v[1],
    );
}
