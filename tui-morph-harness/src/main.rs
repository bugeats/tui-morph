use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph, Wrap};

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
        duration: Duration::from_millis(500),
        ..MorphConfig::default()
    };
    let mut terminal = MorphBackend::wrap(backend, config)?;

    let scenes: &[fn(&mut Frame)] = &[
        scene_inbox,
        scene_detail,
        scene_article,
        scene_article_modal,
        scene_dashboard,
        scene_about,
    ];
    let total = scenes.len() + 1;
    let rangers_idx = scenes.len();

    let mut current = 0;
    let mut ranger_count: usize = 0;
    let mut last_tick = Instant::now();
    let tick_interval = Duration::from_millis(900);

    terminal.draw(|f| scenes[current](f))?;

    loop {
        let timeout = if current == rangers_idx {
            Duration::from_millis(50)
        } else {
            Duration::from_secs(60)
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,

                    KeyCode::Right | KeyCode::Char(' ') | KeyCode::Enter => {
                        current = (current + 1) % total;
                    }

                    KeyCode::Left => {
                        current = (current + total - 1) % total;
                    }

                    _ => continue,
                }

                if current == rangers_idx {
                    ranger_count = 0;
                    last_tick = Instant::now();
                    terminal.draw(|f| scene_rangers(f, ranger_count))?;
                } else {
                    terminal.draw(|f| scenes[current](f))?;
                }
            }
        } else if current == rangers_idx {
            let pause = if ranger_count >= RANGERS.len() {
                tick_interval * 3
            } else {
                tick_interval
            };

            if last_tick.elapsed() >= pause {
                if ranger_count >= RANGERS.len() {
                    ranger_count = 0;
                } else {
                    ranger_count += 1;
                }

                last_tick = Instant::now();
                terminal.draw(|f| scene_rangers(f, ranger_count))?;
            }
        }
    }

    Ok(())
}

fn header(f: &mut Frame, area: Rect, label: &str) {
    let text = format!("tui-morph  [</>  cycle]  [q quit]  |  {label}");

    f.render_widget(
        Paragraph::new(text).style(Style::new().fg(Color::DarkGray)),
        area,
    );
}

fn split_header(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);
    (chunks[0], chunks[1])
}

fn centered_rect(width_pct: u16, height_pct: u16, area: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - height_pct) / 2),
        Constraint::Percentage(height_pct),
        Constraint::Percentage((100 - height_pct) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - width_pct) / 2),
        Constraint::Percentage(width_pct),
        Constraint::Percentage((100 - width_pct) / 2),
    ])
    .split(v[1])[1]
}

fn scene_inbox(f: &mut Frame) {
    let (hdr, body) = split_header(f.area());
    header(f, hdr, "1/7 Inbox");

    let cols = Layout::horizontal([Constraint::Percentage(35), Constraint::Min(0)]).split(body);

    let messages: &[(&str, &str, bool)] = &[
        ("* Deploy v2.4.1", "ops-bot", true),
        ("  Review: morph solver", "chadwick", false),
        ("  Oklch edge cases", "color-team", false),
        ("  Weekly standup notes", "team", false),
        ("  Flake input update", "nix-bot", false),
        ("  Performance regression", "ci", false),
        ("  License audit results", "legal-bot", false),
    ];

    let items: Vec<ListItem> = messages
        .iter()
        .map(|(subject, from, selected)| {
            let style = if *selected {
                Style::new()
                    .fg(Color::Rgb(255, 255, 255))
                    .bg(Color::Rgb(40, 60, 120))
            } else {
                Style::new().fg(Color::Rgb(180, 180, 200))
            };

            ListItem::new(Line::from(vec![
                Span::styled(*subject, style),
                Span::styled(format!("  {from}"), Style::new().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    f.render_widget(
        List::new(items).block(
            Block::bordered().title(" Inbox ").style(
                Style::new()
                    .fg(Color::Rgb(100, 140, 255))
                    .bg(Color::Rgb(15, 15, 30)),
            ),
        ),
        cols[0],
    );

    let preview = "Deploy v2.4.1 completed successfully.\n\
                   \n\
                   Changes:\n\
                     - Morph backend: transition timing fix\n\
                     - Solver: reduced allocation in hot path\n\
                     - Harness: added list and modal scenes\n\
                   \n\
                   All checks passed. Merged to main.";

    f.render_widget(
        Paragraph::new(preview)
            .block(Block::bordered().title(" Preview "))
            .style(
                Style::new()
                    .fg(Color::Rgb(200, 200, 220))
                    .bg(Color::Rgb(15, 15, 30)),
            )
            .wrap(Wrap { trim: false }),
        cols[1],
    );
}

fn scene_detail(f: &mut Frame) {
    let (hdr, body) = split_header(f.area());
    header(f, hdr, "2/7 Detail");

    let cols = Layout::horizontal([Constraint::Length(4), Constraint::Min(0)]).split(body);

    let indicators: Vec<ListItem> =
        std::iter::once(ListItem::new("*").style(Style::new().fg(Color::Rgb(100, 140, 255))))
            .chain((0..6).map(|_| ListItem::new(" ").style(Style::new().fg(Color::DarkGray))))
            .collect();

    f.render_widget(
        List::new(indicators)
            .block(Block::bordered().style(Style::new().bg(Color::Rgb(15, 15, 30)))),
        cols[0],
    );

    let detail = "From: ops-bot <ops@deploy.internal>\n\
                  Date: 2026-02-26 14:32 UTC\n\
                  Subject: Deploy v2.4.1\n\
                  \n\
                  -----------------------------------------------\n\
                  \n\
                  Deploy v2.4.1 completed successfully.\n\
                  \n\
                  ## Changes\n\
                  \n\
                    Morph backend: transition timing fix\n\
                    The tick loop now properly handles sub-ms\n\
                    frame intervals on high-refresh displays.\n\
                  \n\
                    Solver: reduced allocation in hot path\n\
                    Cost matrix is now stack-allocated for diffs\n\
                    under 256 displaced cells. 40% reduction in\n\
                    allocation pressure during transitions.\n\
                  \n\
                    Harness: added list and modal scenes\n\
                    New demo scenes exercise List, scrolling\n\
                    Paragraph, and modal overlay patterns.\n\
                  \n\
                  ## Status\n\
                  \n\
                  All 29 tests passed. Clippy clean. No warnings.\n\
                  Merged to main via fast-forward.";

    f.render_widget(
        Paragraph::new(detail)
            .block(Block::bordered().title(" Deploy v2.4.1 "))
            .style(
                Style::new()
                    .fg(Color::Rgb(200, 210, 230))
                    .bg(Color::Rgb(15, 15, 30)),
            )
            .wrap(Wrap { trim: false }),
        cols[1],
    );
}

fn scene_article(f: &mut Frame) {
    let (hdr, body) = split_header(f.area());
    header(f, hdr, "3/7 Article");

    let text = "Frame-Level Morphing for Terminal UIs\n\
                =====================================\n\
                \n\
                Terminal user interfaces have traditionally been\n\
                immediate-mode: each frame is a complete redraw,\n\
                and transitions between states are instantaneous.\n\
                The user sees a hard cut from one layout to the\n\
                next.\n\
                \n\
                tui-morph introduces a morphing layer that sits\n\
                between the application and the terminal backend.\n\
                It intercepts complete frames, diffs them at the\n\
                cell level, and produces smooth interpolated\n\
                transitions.\n\
                \n\
                The key insight is that terminal cells carry rich\n\
                information: position, glyph, foreground color,\n\
                background color, and text attributes. By operating\n\
                in Oklch color space, we get perceptually linear\n\
                interpolation.\n\
                \n\
                Cell Classification\n\
                -------------------\n\
                \n\
                The solver partitions cells into four categories:\n\
                \n\
                  Stable    - same position, same content.\n\
                  Mutating  - same position, different content.\n\
                  Displaced - content moved between frames.\n\
                  Orphan    - exists in only one frame.\n\
                \n\
                The displaced-cell assignment uses the Hungarian\n\
                algorithm to find the minimum-cost matching. The\n\
                cost function balances spatial distance, glyph\n\
                similarity, and color distance.";

    f.render_widget(
        Paragraph::new(text)
            .block(Block::bordered().title(" Article: Frame-Level Morphing "))
            .style(
                Style::new()
                    .fg(Color::Rgb(230, 200, 150))
                    .bg(Color::Rgb(30, 25, 10)),
            )
            .wrap(Wrap { trim: false })
            .scroll((2, 0)),
        body,
    );
}

fn scene_article_modal(f: &mut Frame) {
    let (hdr, body) = split_header(f.area());
    header(f, hdr, "4/7 Modal");

    let bg_text = "Frame-Level Morphing for Terminal UIs\n\
                   =====================================\n\
                   \n\
                   Terminal user interfaces have traditionally been\n\
                   immediate-mode: each frame is a complete redraw,\n\
                   and transitions between states are instantaneous.";

    f.render_widget(
        Paragraph::new(bg_text)
            .block(Block::bordered().title(" Article "))
            .style(
                Style::new()
                    .fg(Color::Rgb(80, 70, 50))
                    .bg(Color::Rgb(15, 12, 5)),
            )
            .wrap(Wrap { trim: false }),
        body,
    );

    let modal_area = centered_rect(50, 40, body);
    f.render_widget(Clear, modal_area);

    let modal_text = "Are you sure you want to continue?\n\
                      \n\
                      This will apply the morphing configuration\n\
                      to all subsequent frame transitions.\n\
                      \n\
                      [Enter] Confirm    [Esc] Cancel";

    f.render_widget(
        Paragraph::new(modal_text)
            .block(
                Block::bordered()
                    .title(" Confirm ")
                    .border_style(Style::new().fg(Color::Rgb(255, 200, 80))),
            )
            .style(
                Style::new()
                    .fg(Color::Rgb(240, 230, 200))
                    .bg(Color::Rgb(40, 35, 20)),
            )
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center),
        modal_area,
    );
}

fn scene_dashboard(f: &mut Frame) {
    let (hdr, body) = split_header(f.area());
    header(f, hdr, "5/7 Dashboard");

    let grid_rows =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(body);

    let quarters = [Constraint::Percentage(25); 4];
    let top = Layout::horizontal(quarters).split(grid_rows[0]);
    let bot = Layout::horizontal(quarters).split(grid_rows[1]);

    let panes: &[(&str, &str, Color, Color)] = &[
        (
            " CPU ",
            "87%",
            Color::Rgb(255, 120, 80),
            Color::Rgb(40, 15, 10),
        ),
        (
            " Memory ",
            "4.2 / 16 GB",
            Color::Rgb(80, 200, 255),
            Color::Rgb(10, 25, 40),
        ),
        (
            " Network ",
            "12 Mbps in\n 3 Mbps out",
            Color::Rgb(180, 255, 100),
            Color::Rgb(20, 35, 10),
        ),
        (
            " Disk ",
            "218 / 500 GB",
            Color::Rgb(255, 200, 80),
            Color::Rgb(40, 30, 10),
        ),
        (
            " Tasks ",
            "29 passed\n 0 failed",
            Color::Rgb(100, 255, 180),
            Color::Rgb(10, 35, 25),
        ),
        (
            " Uptime ",
            "14d 7h 32m",
            Color::Rgb(200, 160, 255),
            Color::Rgb(25, 15, 40),
        ),
        (
            " Build ",
            "release\nv0.1.0",
            Color::Rgb(255, 140, 200),
            Color::Rgb(40, 15, 30),
        ),
        (
            " Alerts ",
            "0 critical\n2 warnings",
            Color::Rgb(255, 255, 130),
            Color::Rgb(35, 35, 10),
        ),
    ];

    let areas = [
        top[0], top[1], top[2], top[3], bot[0], bot[1], bot[2], bot[3],
    ];

    for (area, (title, content, fg, bg)) in areas.iter().zip(panes.iter()) {
        f.render_widget(
            Paragraph::new(*content)
                .block(Block::bordered().title(*title))
                .style(Style::new().fg(*fg).bg(*bg)),
            *area,
        );
    }
}

fn scene_about(f: &mut Frame) {
    let (hdr, body) = split_header(f.area());
    header(f, hdr, "6/7 About");

    f.render_widget(
        Block::new().style(Style::new().bg(Color::Rgb(20, 10, 30))),
        body,
    );

    let modal_area = centered_rect(60, 50, body);
    f.render_widget(Clear, modal_area);

    let about = "tui-morph v0.1\n\
                 \n\
                 Frame-level morphing layer for ratatui.\n\
                 \n\
                 Interpolates between discrete terminal\n\
                 frames using Oklch color space, producing\n\
                 smooth visual transitions with zero\n\
                 application-level awareness.\n\
                 \n\
                 Features:\n\
                   - Perceptual color interpolation\n\
                   - Hungarian cell assignment\n\
                   - Configurable easing curves\n\
                   - Weight presets: LIQUID, CRISP, FADE\n\
                 \n\
                 Press any key to continue";

    f.render_widget(
        Paragraph::new(about)
            .block(
                Block::bordered()
                    .title(" About ")
                    .border_style(Style::new().fg(Color::Rgb(180, 100, 255))),
            )
            .style(
                Style::new()
                    .fg(Color::Rgb(220, 200, 240))
                    .bg(Color::Rgb(30, 15, 45)),
            )
            .alignment(Alignment::Center),
        modal_area,
    );
}

const RANGERS: &[(&str, &str, &str, Color)] = &[
    ("Jason", "Red Ranger", "Tyrannosaurus", Color::Rgb(220, 40, 40)),
    ("Zack", "Black Ranger", "Mastodon", Color::Rgb(200, 200, 210)),
    ("Billy", "Blue Ranger", "Triceratops", Color::Rgb(60, 120, 255)),
    ("Trini", "Yellow Ranger", "Sabertooth Tiger", Color::Rgb(255, 220, 40)),
    ("Kimberly", "Pink Ranger", "Pterodactyl", Color::Rgb(255, 100, 180)),
    ("Tommy", "Green Ranger", "Dragonzord", Color::Rgb(40, 200, 80)),
];

fn scene_rangers(f: &mut Frame, count: usize) {
    let (hdr, body) = split_header(f.area());
    header(f, hdr, "7/7 Morphin");

    let items: Vec<ListItem> = RANGERS[..count]
        .iter()
        .map(|(name, role, zord, color)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {name:<12}"), Style::new().fg(*color)),
                Span::styled(format!("{role:<20}"), Style::new().fg(*color)),
                Span::styled(format!("  {zord}"), Style::new().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let title = if count >= RANGERS.len() {
        " MEGAZORD ASSEMBLED "
    } else {
        " IT'S MORPHIN' TIME "
    };

    f.render_widget(
        List::new(items).block(
            Block::bordered().title(title).style(
                Style::new()
                    .fg(Color::Rgb(200, 180, 255))
                    .bg(Color::Rgb(15, 10, 30)),
            ),
        ),
        body,
    );
}
