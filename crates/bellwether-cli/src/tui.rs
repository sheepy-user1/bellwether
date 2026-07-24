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
use bellwether_core::{catalog, installer, AppDef};

// Barnyard palette. Rgb keeps it consistent regardless of the user's
// terminal color scheme, unlike the named ANSI colors.
const BARNWOOD: Color = Color::Rgb(150, 105, 60);
const HAY: Color = Color::Rgb(216, 178, 98);
const PASTURE_GREEN: Color = Color::Rgb(90, 130, 60);
const BARN_RED: Color = Color::Rgb(170, 60, 50);
const DUSK: Color = Color::Rgb(120, 110, 100);

struct Row {
    idx: usize,
    area: Rect,
}

/// Which action the last `space`/click selection is destined for. Lets us
/// reuse one selection set for install/repair/remove instead of three.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ArmedAction {
    None,
    ConfirmRemove,
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
    let apps: Vec<&'static AppDef> = catalog();
    let mut selected: HashSet<usize> = HashSet::new();
    let mut cursor: usize = 0;
    let mut log: Vec<String> = vec![format!("welcome to the yard — {}", sys.distro_summary())];
    let mut rows: Vec<Row> = Vec::new();
    let mut working = false;
    let mut armed = ArmedAction::None;

    log.push("scanning pens for what's already out to pasture...".to_string());
    terminal.draw(|f| {
        rows = draw(f, &apps, &[], &selected, cursor, &log, &sys, working, armed);
    })?;
    let mut installed: Vec<bool> = apps
        .iter()
        .map(|a| installer::is_installed(a, &sys))
        .collect();
    let n_installed = installed.iter().filter(|b| **b).count();
    log.push(format!(
        "headcount done: {n_installed} of {} already on this rig",
        apps.len()
    ));
    log.push("space/click: pick · i: buy (install) · r: call the vet (repair) · x: send to pasture (remove) · a: round up the herd (select-all) · 1: Standard · 2: Advanced · 3: Server · q: leave the yard".to_string());

    loop {
        terminal.draw(|f| {
            rows = draw(
                f, &apps, &installed, &selected, cursor, &log, &sys, working, armed,
            );
        })?;

        if !event::poll(std::time::Duration::from_millis(150))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                // Any key other than 'x' cancels a pending remove confirmation.
                let is_x = matches!(key.code, KeyCode::Char('x'));
                if armed == ArmedAction::ConfirmRemove && !is_x {
                    armed = ArmedAction::None;
                    log.push("remove cancelled.".to_string());
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Down | KeyCode::Char('j') => {
                        if cursor + 1 < apps.len() {
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
                        if selected.len() == apps.len() {
                            selected.clear();
                        } else {
                            selected = (0..apps.len()).collect();
                        }
                    }
                    KeyCode::Char('1') => apply_profile("standard", &apps, &mut selected, &mut log),
                    KeyCode::Char('2') => apply_profile("advanced", &apps, &mut selected, &mut log),
                    KeyCode::Char('3') => apply_profile("server", &apps, &mut selected, &mut log),
                    KeyCode::Char('i') | KeyCode::Enter => {
                        if selected.is_empty() {
                            log.push("nothing picked — space to pick some stock first".into());
                        } else {
                            working = true;
                            terminal.draw(|f| {
                                rows = draw(
                                    f, &apps, &installed, &selected, cursor, &log, &sys, working,
                                    armed,
                                );
                            })?;
                            install_selected(&apps, &selected, &sys, &mut log);
                            for &idx in &selected {
                                installed[idx] = installer::is_installed(apps[idx], &sys);
                            }
                            working = false;
                        }
                    }
                    KeyCode::Char('r') => {
                        if selected.is_empty() {
                            log.push("nothing picked — space to pick some stock first".into());
                        } else {
                            working = true;
                            terminal.draw(|f| {
                                rows = draw(
                                    f, &apps, &installed, &selected, cursor, &log, &sys, working,
                                    armed,
                                );
                            })?;
                            repair_selected(&apps, &selected, &sys, &mut log);
                            for &idx in &selected {
                                installed[idx] = installer::is_installed(apps[idx], &sys);
                            }
                            working = false;
                        }
                    }
                    KeyCode::Char('x') => {
                        if selected.is_empty() {
                            log.push("nothing picked — space to pick some stock first".into());
                        } else if armed == ArmedAction::ConfirmRemove {
                            armed = ArmedAction::None;
                            working = true;
                            terminal.draw(|f| {
                                rows = draw(
                                    f, &apps, &installed, &selected, cursor, &log, &sys, working,
                                    armed,
                                );
                            })?;
                            remove_selected(&apps, &selected, &installed, &sys, &mut log);
                            for &idx in &selected {
                                installed[idx] = installer::is_installed(apps[idx], &sys);
                            }
                            working = false;
                        } else {
                            armed = ArmedAction::ConfirmRemove;
                            log.push(
                                "press x again to confirm sending the picked stock to pasture (any other key cancels)"
                                    .to_string(),
                            );
                        }
                    }
                    _ => {}
                }
            }
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

/// Replaces the current selection with a named profile's apps (matched by
/// position in `apps`, since `catalog()` is the same order every call).
fn apply_profile(
    profile_id: &str,
    apps: &[&'static AppDef],
    selected: &mut HashSet<usize>,
    log: &mut Vec<String>,
) {
    match bellwether_core::find_profile(profile_id) {
        Some(p) => {
            selected.clear();
            for (i, app) in apps.iter().enumerate() {
                if p.app_ids.contains(&app.id) {
                    selected.insert(i);
                }
            }
            log.push(format!(
                "picked the '{}' profile ({} apps) — press i to buy the lot",
                p.name,
                p.app_ids.len()
            ));
        }
        None => log.push(format!("no such profile: {profile_id}")),
    }
}

fn install_selected(
    apps: &[&'static AppDef],
    selected: &HashSet<usize>,
    sys: &SystemInfo,
    log: &mut Vec<String>,
) {
    let mut ids: Vec<usize> = selected.iter().copied().collect();
    ids.sort_unstable();
    for idx in ids {
        let app = apps[idx];
        log.push(format!("buying {}...", app.name));
        match installer::install_app(app, sys) {
            Ok(outcome) => {
                if let Some(m) = outcome.method_used {
                    log.push(format!("  sold! via {}", m.label()));
                }
                for note in outcome.post_install_notes {
                    log.push(format!("  - {note}"));
                }
            }
            Err(e) => log.push(format!("  FAILED: {e}")),
        }
    }
    log.push("done at the counter.".to_string());
}

fn repair_selected(
    apps: &[&'static AppDef],
    selected: &HashSet<usize>,
    sys: &SystemInfo,
    log: &mut Vec<String>,
) {
    let mut ids: Vec<usize> = selected.iter().copied().collect();
    ids.sort_unstable();
    for idx in ids {
        let app = apps[idx];
        log.push(format!("the vet is looking at {}...", app.name));
        match installer::repair_app(app, sys) {
            Ok(outcome) => {
                if let Some(m) = outcome.method_used {
                    log.push(format!("  patched up via {}", m.label()));
                }
                for note in outcome.post_install_notes {
                    log.push(format!("  - {note}"));
                }
            }
            Err(e) => log.push(format!("  FAILED: {e}")),
        }
    }
    log.push("all patched up.".to_string());
}

fn remove_selected(
    apps: &[&'static AppDef],
    selected: &HashSet<usize>,
    installed: &[bool],
    sys: &SystemInfo,
    log: &mut Vec<String>,
) {
    let mut ids: Vec<usize> = selected.iter().copied().collect();
    ids.sort_unstable();
    for idx in ids {
        let app = apps[idx];
        if !installed[idx] {
            log.push(format!("{} isn't installed, nothing to send off", app.name));
            continue;
        }
        log.push(format!("sending {} to pasture...", app.name));
        match installer::uninstall_app(app, sys) {
            Ok(()) => log.push("  gone.".to_string()),
            Err(e) => log.push(format!("  FAILED: {e}")),
        }
    }
    log.push("done at the gate.".to_string());
}

#[allow(clippy::too_many_arguments)]
fn draw(
    f: &mut Frame<CrosstermBackend<io::Stdout>>,
    apps: &[&'static AppDef],
    installed: &[bool],
    selected: &HashSet<usize>,
    cursor: usize,
    log: &[String],
    sys: &SystemInfo,
    working: bool,
    armed: ArmedAction,
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
            "🐑 DROVER'S YARD",
            Style::default().fg(HAY).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  —  a Bellwether livestock & software exchange  —  ",
            Style::default().fg(DUSK),
        ),
        Span::raw(format!(
            "{}  —  space: pick · i: buy · r: vet · x: pasture · a: round-up · 1/2/3: profiles · q: leave",
            sys.distro_summary()
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BARNWOOD)),
    );
    f.render_widget(header, chunks[0]);

    let list_area = chunks[1];
    let mut rows = Vec::with_capacity(apps.len());
    let items: Vec<ListItem> = apps
        .iter()
        .enumerate()
        .map(|(i, app)| {
            let checked = selected.contains(&i);
            let marker = if checked { "[x]" } else { "[ ]" };
            let is_installed = installed.get(i).copied().unwrap_or(false);
            let (tag, tag_color) = if is_installed {
                ("SOLD    ", PASTURE_GREEN)
            } else {
                ("FOR SALE", DUSK)
            };
            let base_style = if i == cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let line = Line::from(vec![
                Span::styled(format!(" {marker} "), base_style),
                Span::styled(format!("{tag} "), base_style.fg(tag_color)),
                Span::styled(
                    format!("{:<16} ", app.id),
                    base_style.add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} — {}", app.category.label(), app.description),
                    base_style.fg(DUSK),
                ),
            ]);
            rows.push(Row {
                idx: i,
                area: Rect {
                    x: list_area.x + 1,
                    y: list_area.y + 1 + i as u16,
                    width: list_area.width.saturating_sub(2),
                    height: 1,
                },
            });
            ListItem::new(line)
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .title("The Pens")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BARNWOOD)),
    );
    f.render_widget(list, list_area);

    let log_text: Vec<Line> = log
        .iter()
        .rev()
        .take(5)
        .rev()
        .map(|l| Line::from(l.as_str()))
        .collect();
    let (title, border_color) = if armed == ArmedAction::ConfirmRemove {
        ("Confirm — press x again", BARN_RED)
    } else if working {
        ("Working...", HAY)
    } else {
        ("The Ledger", BARNWOOD)
    };
    let log_widget = Paragraph::new(log_text).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(log_widget, chunks[2]);

    if working {
        let popup = Rect {
            x: size.width / 2 - 10,
            y: size.height / 2 - 1,
            width: 20,
            height: 3,
        };
        f.render_widget(Clear, popup);
        let msg = Paragraph::new("working the yard...").block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(HAY)),
        );
        f.render_widget(msg, popup);
    }

    rows
}
