use ratatui::layout::{Constraint, Direction, Layout, Offset, Rect};
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph};

use crate::debugger::Debugger;
// ---------------------------
pub struct LayoutAreas {
    pub statusbar: Rect,
    pub source: Rect,
    pub stack: Rect,
    pub ctrl: Rect,
}
pub fn layout(area: Rect) -> LayoutAreas {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(80),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(layout[0]);
    LayoutAreas {
        statusbar: layout[1].offset(Offset::new(1,0)),
        source: panes[0],
        stack: panes[1],
        ctrl: panes[2],
    }
}
// ---------------------------
pub fn source_view<'a>(app: &Debugger, tokens: &'a [String]) -> Paragraph<'a> {
    let text_toks: Vec<Line> = tokens
        .iter()
        .enumerate()
        .map(|(i, e)| {
            Line::from(e.as_str())
                .style(token_style(
                    i+1 == app.vm.index,
                    i == app.pointer as usize,
                    app.breakpoints.iter().any(|e| e.clone() == i as i32)
                ))
        })
        .collect();
    Paragraph::new(text_toks).block(panel())
        .style(Style::default().fg(Color::White))
}
// ---------------------------
pub fn statusbar() -> Paragraph<'static> {
    let commands = vec![
        ("Quit", "q"),
        ("Move", "←/→"),
        ("Breakpoint", "b"),
        ("Scroll", "↑/↓"),
        ("Step", "s"),
        ("Run", "r"),
        ("Tab", "t"),
    ];
    let mut instr: Vec<Span> = Vec::new();
    for (name, key) in commands {
        instr.push(Span::from("["));
        instr.push(Span::styled(key, Style::new().fg(Color::Cyan)));
        instr.push(Span::from("] "));
        instr.push(Span::from(name));
        instr.push(Span::from("  "));
    }
    return Paragraph::new(Line::from_iter(instr))
        .style(Style::default().fg(Color::Yellow));
}
// ---------------------------
fn token_style(current: bool, highlight: bool, breakpoint: bool) -> Style {
    return Style::new()
        .add_modifier(if current { Modifier::UNDERLINED } else { Modifier::ITALIC })
        .bg(if breakpoint { Color::Red } else { Color::Reset })
        .fg(if highlight { Color::Cyan } else { Color::Reset });
}
// ---------------------------
pub fn panel() -> Block<'static> {
    return Block::bordered()
        .border_type(BorderType::Rounded)
        .yellow()
        .padding(Padding::horizontal(1));
}

// ╭ Control ╮ Vars   Fields
// │         ╰────────────────╮
// │                          │
// ╰──────────────────────────╯

// ╭─────────╮
// │ Control │  Vars   Fields
// │         ╰────────────────╮
// │                          │
// ╰──────────────────────────╯

// ╭─────────╮  Vars   Fields
// │ Control ╰────────────────╮
// │                          │
// ╰──────────────────────────╯

// ╭─────────╮
// │ Control ╰─ Vars ─ Fields ╮
// │                          │
// ╰──────────────────────────╯
