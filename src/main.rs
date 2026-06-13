use anyhow::{Result, Context};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use sysinfo::{System, Pid, Signal};
use std::{io, time::{Duration, Instant}};
use std::collections::HashMap;

struct App {
    state: ListState,
    items: Vec<OrphanInfo>,
    system: System,
    last_refresh: Instant,
    port_map: HashMap<u32, Vec<u16>>,
}

#[derive(Clone)]
struct OrphanInfo {
    pid: u32,
    name: String,
    memory_mb: f64,
    ports: Vec<u16>,
    path: String,
}

impl App {
    fn new() -> App {
        let mut system = System::new_all();
        system.refresh_all();
        App {
            state: ListState::default(),
            items: Vec::new(),
            system,
            last_refresh: Instant::now(),
            port_map: HashMap::new(),
        }
    }

    fn refresh_data(&mut self) {
        self.system.refresh_all();
        self.refresh_port_map();
        
        // Standard high-resource background processes that often leak
        let suspicious_names = vec![
            "node", "python", "ruby", "java", "chrome", "chromium", "brave", 
            "esbuild", "vite", "next-server", "webpack", "docker", "rustc"
        ];

        let mut orphans = Vec::new();
        for (pid, process) in self.system.processes() {
            let ppid = process.parent().map(|p| p.as_u32()).unwrap_or(0);
            let name = process.name().to_string_lossy().to_lowercase();
            
            // On Linux, orphans are reparented to PID 1
            if ppid == 1 {
                let is_suspicious = suspicious_names.iter().any(|&s| name.contains(s));
                if is_suspicious {
                    orphans.push(OrphanInfo {
                        pid: pid.as_u32(),
                        name: process.name().to_string_lossy().into_owned(),
                        memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
                        ports: self.port_map.get(&pid.as_u32()).cloned().unwrap_or_default(),
                        path: process.exe().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
                    });
                }
            }
        }
        
        orphans.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap_or(std::cmp::Ordering::Equal));
        self.items = orphans;
        
        if self.state.selected().is_none() && !self.items.is_empty() {
            self.state.select(Some(0));
        } else if self.items.is_empty() {
            self.state.select(None);
        }
    }

    fn refresh_port_map(&mut self) {
        self.port_map.clear();
        if let Ok(all_procs) = procfs::process::all_processes() {
            for p_res in all_procs {
                if let Ok(p) = p_res {
                    if let Ok(fds) = p.fd() {
                        for fd_res in fds {
                            if let Ok(fd) = fd_res {
                                if let procfs::process::FDTarget::Socket(inode) = fd.target {
                                    if let Ok(tcp) = procfs::net::tcp() {
                                        for entry in tcp {
                                            if entry.inode == inode {
                                                self.port_map.entry(p.pid as u32).or_default().push(entry.local_address.port());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn kill_selected(&mut self) {
        if let Some(index) = self.state.selected() {
            if let Some(orphan) = self.items.get(index) {
                if let Some(process) = self.system.process(Pid::from_u32(orphan.pid)) {
                    let _ = process.kill_with(Signal::Term);
                }
                self.refresh_data();
            }
        }
    }

    fn next(&mut self) {
        if self.items.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i >= self.items.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.items.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i == 0 { self.items.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn main() -> Result<()> {
    enable_raw_mode().context("Terminal raw mode entry failed")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).context("Terminal setup failed")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Terminal initialization failed")?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

    disable_raw_mode().context("Terminal raw mode exit failed")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).context("Terminal cleanup failed")?;
    terminal.show_cursor().context("Cursor restoration failed")?;

    if let Err(err) = res {
        eprintln!("Fatal error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    app.refresh_data();
    loop {
        terminal.draw(|f| ui(f, &mut app)).map_err(|e| anyhow::anyhow!("Render failed: {}", e))?;

        if event::poll(Duration::from_millis(500)).context("Polling failed")? {
            if let Event::Key(key) = event::read().context("Event capture failed")? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char('r') => app.refresh_data(),
                        KeyCode::Enter | KeyCode::Char('x') => app.kill_selected(),
                        _ => {}
                    }
                }
            }
        }
        
        if app.last_refresh.elapsed() > Duration::from_secs(5) {
            app.refresh_data();
            app.last_refresh = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = Paragraph::new(" Orphan Process Manager ")
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Gray)));
    f.render_widget(header, chunks[0]);

    let list_items: Vec<ListItem> = app.items
        .iter()
        .map(|i| {
            let ports = if i.ports.is_empty() {
                String::new()
            } else {
                format!(" [Ports: {:?}]", i.ports)
            };
            ListItem::new(format!(
                " PID {:<7} | {:<15} | {:>8.2} MB{}",
                i.pid, i.name, i.memory_mb, ports
            ))
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().title(" Background Resource Usage ").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[1], &mut app.state);

    let info_text = if let Some(index) = app.state.selected() {
        if let Some(item) = app.items.get(index) {
            format!(" {} | [X] Terminate | [R] Refresh | [Q] Exit ", item.path)
        } else {
            " [R] Refresh | [Q] Exit ".to_string()
        }
    } else {
        " [R] Refresh | [Q] Exit ".to_string()
    };

    let footer = Paragraph::new(info_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}
