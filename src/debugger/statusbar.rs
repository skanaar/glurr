use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Paragraph};

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
