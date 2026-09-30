use crate::{post_all, Account};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout},
    widgets::{Block, Paragraph, Wrap},
};

const HELP: &str = "Ctrl+S: post  Alt+<n>: toggle account  Esc: quit";

pub fn run(accounts: &[Account]) -> std::io::Result<()> {
    let mut on = vec![true; accounts.len()];
    let mut text = String::new();
    let mut status = HELP.to_string();
    let mut term = ratatui::init();
    let res = loop {
        let drawn = term.draw(|f| {
            let [top, mid, bot] = Layout::vertical([
                Constraint::Length(3),
                Constraint::Min(3),
                Constraint::Length(3),
            ])
            .areas(f.area());
            let targets: String = accounts
                .iter()
                .zip(&on)
                .enumerate()
                .map(|(i, (a, o))| format!("[{}] {}:{}  ", if *o { "x" } else { " " }, i + 1, a.name))
                .collect();
            f.render_widget(Paragraph::new(targets).block(Block::bordered().title("Accounts")), top);
            // ponytail: cursor is always at the end. Swap in tui-textarea if mid-text editing is needed
            f.render_widget(
                Paragraph::new(format!("{text}▏"))
                    .wrap(Wrap { trim: false })
                    .block(Block::bordered().title(format!("Text ({} chars)", text.chars().count()))),
                mid,
            );
            f.render_widget(Paragraph::new(status.as_str()).block(Block::bordered()), bot);
        });
        if let Err(e) = drawn {
            break Err(e);
        }
        let k = match event::read() {
            Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => k,
            Ok(_) => continue,
            Err(e) => break Err(e),
        };
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        match k.code {
            KeyCode::Esc => break Ok(()),
            KeyCode::Char('c') if ctrl => break Ok(()),
            KeyCode::Char('s') if ctrl => {
                let targets: Vec<&Account> =
                    accounts.iter().zip(&on).filter(|(_, o)| **o).map(|(a, _)| a).collect();
                if text.trim().is_empty() || targets.is_empty() {
                    status = "Text or accounts are empty".into();
                    continue;
                }
                let results = post_all(&targets, text.trim_end());
                if results.iter().all(|(_, r)| r.is_ok()) {
                    text.clear();
                }
                status = results
                    .into_iter()
                    .map(|(n, r)| match r {
                        Ok(_) => format!("✓ {n}  "),
                        Err(e) => format!("✗ {n}: {e}  "),
                    })
                    .collect();
            }
            KeyCode::Char(c) if k.modifiers.contains(KeyModifiers::ALT) => {
                if let Some(o) = c.to_digit(10).and_then(|d| on.get_mut((d as usize).wrapping_sub(1))) {
                    *o = !*o;
                }
            }
            KeyCode::Char(c) => text.push(c),
            KeyCode::Enter => text.push('\n'),
            KeyCode::Backspace => {
                text.pop();
            }
            _ => {}
        }
    };
    ratatui::restore();
    res
}
