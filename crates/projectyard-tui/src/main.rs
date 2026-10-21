//! Interactive terminal inventory for ProjectYard.

use std::{
    env,
    io::{self, stdout},
    path::PathBuf,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use projectyard_index::{IndexedProject, ProjectIndex, ScanSummary};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Wrap},
};

const DEFAULT_DATABASE: &str = ".projectyard/index.sqlite";

fn main() -> io::Result<()> {
    let database = match parse_database_path(env::args().skip(1).collect()) {
        Ok(path) => path,
        Err(message) => {
            eprintln!("ProjectYard: {message}");
            eprintln!("Usage: projectyard-tui [--database FILE]");
            std::process::exit(2);
        }
    };
    let mut app = App::load(database);
    run_terminal(&mut app)
}

fn parse_database_path(args: Vec<String>) -> Result<PathBuf, String> {
    let mut database = PathBuf::from(DEFAULT_DATABASE);
    let mut position = 0;
    while position < args.len() {
        match args[position].as_str() {
            "--database" => {
                position += 1;
                database = PathBuf::from(
                    args.get(position)
                        .ok_or("--database requires a file path")?,
                );
            }
            "--help" | "-h" => return Err("help requested".to_owned()),
            value => return Err(format!("unknown argument `{value}`")),
        }
        position += 1;
    }
    Ok(database)
}

fn run_terminal(app: &mut App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut output = stdout();
    execute!(output, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(backend)?;
    let outcome = run_app(&mut terminal, app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    outcome
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| draw(frame, app))?;
        if !event::poll(Duration::from_millis(250))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(key.code);
                if app.should_quit {
                    return Ok(());
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputMode {
    Navigate,
    Search,
}

#[derive(Debug)]
struct App {
    database: PathBuf,
    projects: Vec<IndexedProject>,
    scan: Option<ScanSummary>,
    table_state: TableState,
    query: String,
    input_mode: InputMode,
    status: String,
    should_quit: bool,
}

impl App {
    fn load(database: PathBuf) -> Self {
        let mut app = Self {
            database,
            projects: Vec::new(),
            scan: None,
            table_state: TableState::default(),
            query: String::new(),
            input_mode: InputMode::Navigate,
            status: String::new(),
            should_quit: false,
        };
        app.reload();
        app
    }

    fn reload(&mut self) {
        match ProjectIndex::open(&self.database) {
            Ok(index) => match (index.projects(), index.latest_scan()) {
                (Ok(projects), Ok(scan)) => {
                    self.projects = projects;
                    self.scan = scan;
                    self.status = if self.projects.is_empty() {
                        "No indexed projects yet. Run `projectyard index PATH` first.".to_owned()
                    } else {
                        format!(
                            "Loaded {} projects from {}",
                            self.projects.len(),
                            self.database.display()
                        )
                    };
                }
                (Err(error), _) | (_, Err(error)) => {
                    self.status = format!("Could not read index: {error}");
                }
            },
            Err(error) => self.status = format!("Could not open index: {error}"),
        }
        self.ensure_selection();
    }

    fn filtered_indexes(&self) -> Vec<usize> {
        let needle = self.query.trim().to_lowercase();
        if needle.is_empty() {
            return (0..self.projects.len()).collect();
        }
        self.projects
            .iter()
            .enumerate()
            .filter(|(_, project)| searchable_project(project).contains(&needle))
            .map(|(index, _)| index)
            .collect()
    }

    fn selected_project(&self) -> Option<&IndexedProject> {
        self.table_state
            .selected()
            .and_then(|visible_index| self.filtered_indexes().get(visible_index).copied())
            .and_then(|project_index| self.projects.get(project_index))
    }

    fn ensure_selection(&mut self) {
        let length = self.filtered_indexes().len();
        match (length, self.table_state.selected()) {
            (0, _) => self.table_state.select(None),
            (_, None) => self.table_state.select(Some(0)),
            (length, Some(selected)) if selected >= length => {
                self.table_state.select(Some(length - 1));
            }
            _ => {}
        }
    }

    fn select_next(&mut self) {
        let length = self.filtered_indexes().len();
        if length == 0 {
            return;
        }
        let next = self
            .table_state
            .selected()
            .map_or(0, |selected| (selected + 1) % length);
        self.table_state.select(Some(next));
    }

    fn select_previous(&mut self) {
        let length = self.filtered_indexes().len();
        if length == 0 {
            return;
        }
        let previous = self.table_state.selected().map_or(0, |selected| {
            if selected == 0 {
                length - 1
            } else {
                selected - 1
            }
        });
        self.table_state.select(Some(previous));
    }

    fn handle_key(&mut self, code: KeyCode) {
        match self.input_mode {
            InputMode::Navigate => match code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Char('/') => self.input_mode = InputMode::Search,
                KeyCode::Char('j') | KeyCode::Down => self.select_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
                KeyCode::Char('r') => self.reload(),
                _ => {}
            },
            InputMode::Search => match code {
                KeyCode::Esc | KeyCode::Enter => self.input_mode = InputMode::Navigate,
                KeyCode::Backspace => {
                    self.query.pop();
                    self.ensure_selection();
                }
                KeyCode::Char(character) => {
                    self.query.push(character);
                    self.ensure_selection();
                }
                _ => {}
            },
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(2),
        ])
        .split(area);
    draw_header(vertical[0], frame, app);
    draw_search(vertical[1], frame, app);
    draw_workspace(vertical[2], frame, app);
    draw_footer(vertical[3], frame, app);
}

fn draw_header(area: Rect, frame: &mut Frame, app: &App) {
    let project_count = app.projects.len();
    let subtitle = app.scan.map_or_else(
        || "No completed scan".to_owned(),
        |summary| {
            format!(
                "{project_count} projects · {} folders checked",
                summary.directories_visited
            )
        },
    );
    let paragraph = Paragraph::new(vec![
        Line::from(Span::styled(
            "ProjectYard",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(subtitle).style(Style::default().fg(Color::DarkGray)),
    ])
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(paragraph, area);
}

fn draw_search(area: Rect, frame: &mut Frame, app: &App) {
    let active = app.input_mode == InputMode::Search;
    let value = if active {
        format!("{}▏", app.query)
    } else {
        app.query.clone()
    };
    let title = if active {
        " Search — typing "
    } else {
        " Search — press / "
    };
    let search = Paragraph::new(if value.is_empty() {
        "all projects"
    } else {
        &value
    })
    .style(Style::default().fg(Color::White))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(if active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            }),
    );
    frame.render_widget(search, area);
}

fn draw_workspace(area: Rect, frame: &mut Frame, app: &mut App) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);
    draw_projects(columns[0], frame, app);
    draw_details(columns[1], frame, app);
}

fn draw_projects(area: Rect, frame: &mut Frame, app: &mut App) {
    let indexes = app.filtered_indexes();
    if indexes.is_empty() {
        let message = if app.projects.is_empty() {
            "Nothing has been indexed yet.\n\nRun:\nprojectyard index ~/Code"
        } else {
            "No projects match this search."
        };
        frame.render_widget(
            Paragraph::new(message)
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title(" Projects ")),
            area,
        );
        return;
    }
    let rows = indexes
        .iter()
        .filter_map(|index| app.projects.get(*index))
        .map(|project| {
            let name = project
                .root
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("/");
            let kinds = project
                .kinds
                .iter()
                .map(|kind| kind.label())
                .collect::<Vec<_>>()
                .join(", ");
            Row::new(vec![
                name.to_owned(),
                truncate(&kinds, 24),
                truncate(&project.root.display().to_string(), 38),
            ])
        });
    let table = Table::new(
        rows,
        [
            Constraint::Length(22),
            Constraint::Length(25),
            Constraint::Min(20),
        ],
    )
    .header(
        Row::new(["Project", "Technology", "Location"]).style(Style::default().fg(Color::DarkGray)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Projects ({}) ", indexes.len())),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(25, 62, 76))
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("› ");
    frame.render_stateful_widget(table, area, &mut app.table_state);
}

fn draw_details(area: Rect, frame: &mut Frame, app: &App) {
    let (title, body) = match app.selected_project() {
        Some(project) => {
            let mut body = vec![
                Line::from(Span::styled("Location", Style::default().fg(Color::Cyan))),
                Line::from(project.root.display().to_string()),
                Line::from(""),
                Line::from(Span::styled(
                    "Detected technologies",
                    Style::default().fg(Color::Cyan),
                )),
            ];
            body.push(Line::from(
                project
                    .kinds
                    .iter()
                    .map(|kind| kind.label())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
            body.extend([
                Line::from(""),
                Line::from(Span::styled(
                    "Found markers",
                    Style::default().fg(Color::Cyan),
                )),
            ]);
            body.push(Line::from(project.markers.join(", ")));
            (" Evidence ", body)
        }
        None => (
            " Evidence ",
            vec![Line::from(
                "Select a project to see why ProjectYard found it.",
            )],
        ),
    };
    frame.render_widget(
        Paragraph::new(body)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
}

fn draw_footer(area: Rect, frame: &mut Frame, app: &App) {
    let controls = match app.input_mode {
        InputMode::Navigate => "j/k or ↑/↓ select · / search · r reload · q quit",
        InputMode::Search => "type to filter · enter keep search · esc stop searching",
    };
    let line = Line::from(vec![
        Span::styled(controls, Style::default().fg(Color::DarkGray)),
        Span::raw("    "),
        Span::styled(&app.status, Style::default().fg(Color::Gray)),
    ]);
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::TOP)),
        area,
    );
}

fn searchable_project(project: &IndexedProject) -> String {
    let kinds = project
        .kinds
        .iter()
        .map(|kind| kind.label())
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "{} {kinds} {}",
        project.root.display(),
        project.markers.join(" ")
    )
    .to_lowercase()
}

fn truncate(value: &str, maximum: usize) -> String {
    if value.chars().count() <= maximum {
        return value.to_owned();
    }
    let mut shortened: String = value.chars().take(maximum.saturating_sub(1)).collect();
    shortened.push('…');
    shortened
}

#[cfg(test)]
mod tests {
    use super::*;
    use projectyard_core::ProjectKind;

    fn project(path: &str, kind: ProjectKind) -> IndexedProject {
        IndexedProject {
            root: PathBuf::from(path),
            kinds: vec![kind],
            markers: vec!["marker".to_owned()],
            modified_unix_seconds: None,
            indexed_unix_seconds: 0,
        }
    }

    #[test]
    fn search_matches_project_path_and_technology() {
        let mut app = App {
            database: PathBuf::from("test.sqlite"),
            projects: vec![project("/work/yard", ProjectKind::Cargo)],
            scan: None,
            table_state: TableState::default(),
            query: "rust".to_owned(),
            input_mode: InputMode::Navigate,
            status: String::new(),
            should_quit: false,
        };
        app.ensure_selection();
        assert_eq!(app.filtered_indexes(), vec![0]);
        app.query = "missing".to_owned();
        app.ensure_selection();
        assert_eq!(app.table_state.selected(), None);
    }

    #[test]
    fn truncate_keeps_short_values_and_marks_long_ones() {
        assert_eq!(truncate("yard", 8), "yard");
        assert_eq!(truncate("projectyard", 8), "project…");
    }
}
