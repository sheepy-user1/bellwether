use std::collections::{HashMap, HashSet};
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
use bellwether_core::{catalog, find_profile, installer, AppDef};

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

/// The four pens. Home/Advanced/Server mirror the profiles in
/// bellwether-core; All is everything in the catalog, so nothing (like
/// the snap-purge utility, or your own apps) ever becomes unreachable.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Home,
    Advanced,
    Server,
    All,
}

impl Tab {
    fn all() -> [Tab; 4] {
        [Tab::Home, Tab::Advanced, Tab::Server, Tab::All]
    }

    fn label(&self) -> &'static str {
        match self {
            Tab::Home => "Home",
            Tab::Advanced => "Advanced",
            Tab::Server => "Server",
            Tab::All => "All",
        }
    }

    fn profile_id(&self) -> Option<&'static str> {
        match self {
            Tab::Home => Some("home"),
            Tab::Advanced => Some("advanced"),
            Tab::Server => Some("server"),
            Tab::All => None,
        }
    }

    fn next(&self) -> Tab {
        match self {
            Tab::Home => Tab::Advanced,
            Tab::Advanced => Tab::Server,
            Tab::Server => Tab::All,
            Tab::All => Tab::Home,
        }
    }
}

/// Apps to show for a given tab, preserving the catalog's own ordering.
fn apps_for_tab(tab: Tab, full: &[&'static AppDef]) -> Vec<&'static AppDef> {
    match tab.profile_id() {
        Some(pid) => match find_profile(pid) {
            Some(p) => full
                .iter()
                .copied()
                .filter(|a| p.app_ids.contains(&a.id))
                .collect(),
            None => Vec::new(),
        },
        None => full.to_vec(),
    }
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

#[allow(clippy::too_many_arguments)]
fn goto_tab(
    new_tab: Tab,
    full_apps: &[&'static AppDef],
    tab: &mut Tab,
    apps: &mut Vec<&'static AppDef>,
    selected: &mut HashSet<usize>,
    cursor: &mut usize,
    log: &mut Vec<String>,
) {
    *tab = new_tab;
    *apps = apps_for_tab(new_tab, full_apps);
    selected.clear();
    *cursor = 0;
    log.push(format!("moved to the {} pen", new_tab.label()));
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let sys = SystemInfo::detect();
    let full_apps: Vec<&'static AppDef> = catalog();
    let mut tab = Tab::Home;
    let mut apps: Vec<&'static AppDef> = apps_for_tab(tab, &full_apps);
    let mut selected: HashSet<usize> = HashSet::new();
    let mut cursor: usize = 0;
    let mut log: Vec<String> = vec![format!("welcome to the yard — {}", sys.distro_summary())];
    let mut rows: Vec<Row> = Vec::new();
    let mut working = false;
    let mut armed = ArmedAction::None;

    log.push("taking headcount across every pen...".to_string());
    terminal.draw(|f| {
        rows = draw(
            f,
            tab,
            &apps,
            &HashMap::new(),
            &selected,
            cursor,
            &log,
            &sys,
            working,
            armed,
        );
    })?;
    let mut installed: HashMap<&'static str, bool> = full_apps
        .iter()
        .map(|a| (a.id, installer::is_installed(a, &sys)))
        .collect();
    let n_installed = installed.values().filter(|b| **b).count();
    log.push(format!(
        "headcount done: {n_installed} of {} already in the barn",
        full_apps.len()
    ));
    log.push(
        "space/click: pick · i: bring in · r: call the vet · x: send to pasture · a: round up the herd · Tab/1-4: switch pens · q: leave the yard"
            .to_string(),
    );

    loop {
        terminal.draw(|f| {
            rows = draw(
                f, tab, &apps, &installed, &selected, cursor, &log, &sys, working, armed,
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
                    log.push("removal cancelled.".to_string());
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
                    KeyCode::Tab => {
                        goto_tab(
                            tab.next(),
                            &full_apps,
                            &mut tab,
                            &mut apps,
                            &mut selected,
                            &mut cursor,
                            &mut log,
                        );
                    }
                    KeyCode::Char('1') => goto_tab(
                        Tab::Home,
                        &full_apps,
                        &mut tab,
                        &mut apps,
                        &mut selected,
                        &mut cursor,
                        &mut log,
                    ),
                    KeyCode::Char('2') => goto_tab(
                        Tab::Advanced,
                        &full_apps,
                        &mut tab,
                        &mut apps,
                        &mut selected,
                        &mut cursor,
                        &mut log,
                    ),
                    KeyCode::Char('3') => goto_tab(
                        Tab::Server,
                        &full_apps,
                        &mut tab,
                        &mut apps,
                        &mut selected,
                        &mut cursor,
                        &mut log,
                    ),
                    KeyCode::Char('4') => goto_tab(
                        Tab::All,
                        &full_apps,
                        &mut tab,
                        &mut apps,
                        &mut selected,
                        &mut cursor,
                        &mut log,
                    ),
                    KeyCode::Char('i') | KeyCode::Enter => {
                        if selected.is_empty() {
                            log.push("nothing picked — space to pick some stock first".into());
                        } else {
                            working = true;
                            terminal.draw(|f| {
                                rows = draw(
                                    f, tab, &apps, &installed, &selected, cursor, &log, &sys,
                                    working, armed,
                                );
                            })?;
                            bring_in_selected(&apps, &selected, &sys, &mut log);
                            for &idx in &selected {
                                let app = apps[idx];
                                installed.insert(app.id, installer::is_installed(app, &sys));
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
                                    f, tab, &apps, &installed, &selected, cursor, &log, &sys,
                                    working, armed,
                                );
                            })?;
                            repair_selected(&apps, &selected, &sys, &mut log);
                            for &idx in &selected {
                                let app = apps[idx];
                                installed.insert(app.id, installer::is_installed(app, &sys));
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
                                    f, tab, &apps, &installed, &selected, cursor, &log, &sys,
                                    working, armed,
                                );
                            })?;
                            remove_selected(&apps, &selected, &installed, &sys, &mut log);
                            for &idx in &selected {
                                let app = apps[idx];
                                installed.insert(app.id, installer::is_installed(app, &sys));
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

fn bring_in_selected(
    apps: &[&'static AppDef],
    selected: &HashSet<usize>,
    sys: &SystemInfo,
    log: &mut Vec<String>,
) {
    let mut ids: Vec<usize> = selected.iter().copied().collect();
    ids.sort_unstable();
    for idx in ids {
        let app = apps[idx];
        log.push(format!("bringing {} in from the field...", app.name));
        match installer::install_app(app, sys) {
            Ok(outcome) => {
                if let Some(m) = outcome.method_used {
                    log.push(format!("  in the barn now, via {}", m.label()));
                }
                for note in outcome.post_install_notes {
                    log.push(format!("  - {note}"));
                }
            }
            Err(e) => log.push(format!("  FAILED: {e}")),
        }
    }
    log.push("all brought in.".to_string());
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
    installed: &HashMap<&'static str, bool>,
    sys: &SystemInfo,
    log: &mut Vec<String>,
) {
    let mut ids: Vec<usize> = selected.iter().copied().collect();
    ids.sort_unstable();
    for idx in ids {
        let app = apps[idx];
        if !installed.get(app.id).copied().unwrap_or(false) {
            log.push(format!(
                "{} isn't in the barn, nothing to send off",
                app.name
            ));
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
    tab: Tab,
    apps: &[&'static AppDef],
    installed: &HashMap<&'static str, bool>,
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
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(7),
        ])
        .split(size);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "🐑 DROVER'S YARD",
            Style::default().fg(HAY).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  —  herding your Linux flock into shape  —  ",
            Style::default().fg(DUSK),
        ),
        Span::raw(format!(
            "{}  —  space: pick · i: bring in · r: vet · x: pasture",
            sys.distro_summary()
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BARNWOOD)),
    );
    f.render_widget(header, chunks[0]);

    let mut tab_spans = Vec::new();
    for t in Tab::all() {
        let active = t == tab;
        let style = if active {
            Style::default()
                .fg(Color::Black)
                .bg(HAY)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DUSK)
        };
        tab_spans.push(Span::styled(format!(" {} ", t.label()), style));
        tab_spans.push(Span::raw(" "));
    }
    tab_spans.push(Span::styled(
        "  (Tab or 1-4 to switch)",
        Style::default().fg(DUSK),
    ));
    let tab_bar = Paragraph::new(Line::from(tab_spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BARNWOOD)),
    );
    f.render_widget(tab_bar, chunks[1]);

    let list_area = chunks[2];
    let mut rows = Vec::with_capacity(apps.len());
    let items: Vec<ListItem> = apps
        .iter()
        .enumerate()
        .map(|(i, app)| {
            let checked = selected.contains(&i);
            let marker = if checked { "[x]" } else { "[ ]" };
            let is_installed = installed.get(app.id).copied().unwrap_or(false);
            let (tag, tag_color) = if is_installed {
                ("IN THE BARN  ", PASTURE_GREEN)
            } else {
                ("OUT TO PASTURE", DUSK)
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
            .title(format!("The {} Pen", tab.label()))
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
    f.render_widget(log_widget, chunks[3]);

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
