use std::collections::HashSet;
use std::io;

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::{Frame, Terminal};

use bellwether_core::SystemInfo;
use bellwether_core::{installer, CATALOG};

struct Row {
    idx: usize,
    area: Rect,
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let sys = SystemInfo::detect();
    let mut selected: HashSet<usize> = HashSet::new();
    let mut cursor: usize = 0;
    let mut log: Vec<String> = vec![
        format!("detected: {}", sys.distro_summary()),
        "space/click = toggle, i = install selected, q = quit".to_string(),
    ];
    let mut rows: Vec<Row> = Vec::new();
    let mut installing = false;

    loop {
        terminal.draw(|f| {
            rows = draw(f, &selected, cursor, &log, &sys, installing);
        })?;

        if !event::poll(std::time::Duration::from_millis(150))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => {
                    if cursor + 1 < CATALOG.len() {
                        cursor += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    cursor = cursor.saturating_sub(1);
                }
                KeyCode::Char(' ') => {
                    toggle(&mut selected, cursor);
                }
                KeyCode::Char('a') => {
                    if selected.len() == CATALOG.len() {
                        selected.clear();
                    } else {
                        selected = (0..CATALOG.len()).collect();
                    }
                }
                KeyCode::Char('i') | KeyCode::Enter => {
                    if selected.is_empty() {
                        log.push("nothing selected — press space to select apps first".into());
                    } else {
                        installing = true;
                        terminal.draw(|f| {
                            rows = draw(f, &selected, cursor, &log, &sys, installing);
                        })?;
                        install_selected(&selected, &sys, &mut log);
                        installing = false;
                    }
                }
                _ => {}
            },
            Event::Mouse(m) => {
                if let MouseEventKind::Down(MouseButton::Left) = m.kind {
                    for row in &rows {
                        if row.area.y == m.row
                            && m.column >= row.area.x
                            && m.column < row.area.x + row.area.width
                        {
                            cursor = row.idx;
                            toggle(&mut selected, row.idx);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn toggle(selected: &mut HashSet<usize>, idx: usize) {
    if !selected.insert(idx) {
        selected.remove(&idx);
    }
}

fn install_selected(selected: &HashSet<usize>, sys: &SystemInfo, log: &mut Vec<String>) {
    let mut ids: Vec<usize> = selected.iter().copied().collect();
    ids.sort_unstable();
    for idx in ids {
        let app = &CATALOG[idx];
        log.push(format!("installing {}...", app.name));
        match installer::install_app(app, sys) {
            Ok(outcome) => {
                if let Some(m) = outcome.method_used {
                    log.push(format!("  ok, via {}", m.label()));
                }
                for note in outcome.post_install_notes {
                    log.push(format!("  - {note}"));
                }
            }
            Err(e) => log.push(format!("  FAILED: {e}")),
        }
    }
    log.push("done.".to_string());
}

fn draw(
    f: &mut Frame<CrosstermBackend<io::Stdout>>,
    selected: &HashSet<usize>,
    cursor: usize,
    log: &[String],
    sys: &SystemInfo,
    installing: bool,
) -> Vec<Row> {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(7),
        ])
        .split(size);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "Bellwether",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "  —  {}  —  space: toggle · i: install · a: select-all · q: quit",
            sys.distro_summary()
        )),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    let list_area = chunks[1];
    let mut rows = Vec::with_capacity(CATALOG.len());
    let items: Vec<ListItem> = CATALOG
        .iter()
        .enumerate()
        .map(|(i, app)| {
            let checked = selected.contains(&i);
            let marker = if checked { "[x]" } else { "[ ]" };
            let style = if i == cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let line = format!(
                " {marker} {:<16} {} — {}",
                app.id,
                app.category.label(),
                app.description
            );
            rows.push(Row {
                idx: i,
                area: Rect {
                    x: list_area.x + 1,
                    y: list_area.y + 1 + i as u16,
                    width: list_area.width.saturating_sub(2),
                    height: 1,
                },
            });
            ListItem::new(line).style(style)
        })
        .collect();
    let list = List::new(items).block(Block::default().title("Apps").borders(Borders::ALL));
    f.render_widget(list, list_area);

    let log_text: Vec<Line> = log
        .iter()
        .rev()
        .take(5)
        .rev()
        .map(|l| Line::from(l.as_str()))
        .collect();
    let title = if installing { "Working..." } else { "Log" };
    let log_widget =
        Paragraph::new(log_text).block(Block::default().title(title).borders(Borders::ALL));
    f.render_widget(log_widget, chunks[2]);

    if installing {
        let popup = Rect {
            x: size.width / 2 - 10,
            y: size.height / 2 - 1,
            width: 20,
            height: 3,
        };
        f.render_widget(Clear, popup);
        let msg = Paragraph::new("installing...").block(Block::default().borders(Borders::ALL));
        f.render_widget(msg, popup);
    }

    rows
}
