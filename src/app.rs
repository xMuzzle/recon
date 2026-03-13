use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};

use crate::process;
use crate::session::{self, Session};
use crate::warp;

pub struct App {
    pub sessions: Vec<Session>,
    pub selected: usize,
    pub effort_level: String,
    pub should_quit: bool,
    prev_sessions: HashMap<String, Session>,
}

impl App {
    pub fn new() -> Self {
        let effort_level = read_effort_level().unwrap_or_else(|| "medium".to_string());
        App {
            sessions: Vec::new(),
            selected: 0,
            effort_level,
            should_quit: false,
            prev_sessions: HashMap::new(),
        }
    }

    pub fn refresh(&mut self) {
        let procs = process::discover_claude_processes();
        let mut sessions = session::resolve_sessions(&procs, &self.prev_sessions);

        // Store for next incremental parse
        self.prev_sessions = sessions
            .iter()
            .map(|s| (s.session_id.clone(), s.clone()))
            .collect();

        // Merge Warp tab titles by tab_number (1-indexed)
        let tab_titles = warp::get_tab_titles();
        for session in sessions.iter_mut() {
            if let Some(n) = session.tab_number {
                if let Some(title) = tab_titles.get((n - 1) as usize) {
                    if !title.is_empty() {
                        session.tab_title = Some(title.clone());
                    }
                }
            }
        }

        self.sessions = sessions;

        // Clamp selection
        if self.selected >= self.sessions.len() && !self.sessions.is_empty() {
            self.selected = self.sessions.len() - 1;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.sessions.is_empty() {
                    self.selected = (self.selected + 1).min(self.sessions.len() - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Enter => {
                self.jump_to_session(self.selected);
            }
            KeyCode::Char('r') => {
                self.refresh();
            }
            KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                // Number keys jump to the session at that list position
                let idx = (c as u8 - b'1') as usize;
                self.jump_to_session(idx);
            }
            _ => {}
        }
    }

    fn jump_to_session(&self, idx: usize) {
        if let Some(session) = self.sessions.get(idx) {
            if let Some(tab) = session.tab_number {
                warp::switch_to_tab_number(tab);
            }
        }
    }

    pub fn to_json(&self) -> String {
        let sessions: Vec<serde_json::Value> = self
            .sessions
            .iter()
            .map(|s| {
                serde_json::json!({
                    "session_id": s.session_id,
                    "project_name": s.project_name,
                    "tab_title": s.tab_title,
                    "tab_number": s.tab_number,
                    "model": s.model,
                    "model_display": s.model_display(&self.effort_level),
                    "total_input_tokens": s.total_input_tokens,
                    "total_output_tokens": s.total_output_tokens,
                    "tokens_display": s.token_display(),
                    "token_ratio": s.token_ratio(),
                    "status": s.status.label(),
                    "pid": s.pid,
                    "tty": s.tty,
                    "last_activity": s.last_activity,
                })
            })
            .collect();

        serde_json::to_string_pretty(&serde_json::json!({
            "sessions": sessions,
            "effort_level": self.effort_level,
        }))
        .unwrap_or_else(|_| "{}".to_string())
    }
}

fn read_effort_level() -> Option<String> {
    let home = dirs::home_dir()?;
    let path = home.join(".claude").join("settings.json");
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get("effortLevel")?.as_str().map(|s| s.to_string())
}
