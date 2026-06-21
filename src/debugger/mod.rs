use std::cmp::max;
use ratatui::text::{Line, Text};
use ratatui::{Frame, Terminal};
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{KeyCode::*, KeyEvent, KeyModifiers};

mod event;
mod tui;
mod ui;

use event::{Event, Events};
use ratatui::widgets::Paragraph;
use tui::Tui;

use crate::debugger::ui::{statusbar, layout, panel, source_view};
use crate::virtual_machine::VirtualMachine;

pub struct Debugger {
    pub should_quit: bool,
    pub pointer: i32,
    pub breakpoints: Vec<i32>,
    pub vm: VirtualMachine
}

impl Debugger {
    pub fn new(vm: VirtualMachine) -> Self {
        Self { should_quit: false, pointer: 0, breakpoints: Vec::new(), vm: vm }
    }
    pub fn run(&mut self) -> std::io::Result<()> {
        let backend = CrosstermBackend::new(std::io::stderr());
        let mut terminal = Terminal::new(backend)?;
        let mut events = Events::new();
        Tui::enter()?;
        terminal.hide_cursor()?;
        terminal.clear()?;
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key_event) = events.next()? {
                self.input(key_event);
            }
        }
        Tui::teardown()?;
        terminal.show_cursor()?;
        Ok(())
    }
    fn step(&mut self) { self.should_quit = !self.vm.debug_step(); }
    fn quit(&mut self) { self.should_quit = true }
    fn left(&mut self) { self.pointer = max(0, self.pointer - 1) }
    fn right(&mut self) { self.pointer = max(0, self.pointer + 1) }
    fn toggle_breakpoint(&mut self) {
        let p = self.pointer;
        if let Some(i) = self.breakpoints.iter().position(|e| e.clone() == p) {
            self.breakpoints.remove(i);
        } else {
            self.breakpoints.push(self.pointer)
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let tokens: Vec<String> =
            self.vm.tokens.iter().map(|e|self.vm.serialize_token(e)).collect();
        let layout = layout(frame.area());
        frame.render_widget(source_view(self, &tokens), layout.source);
        let stack_items: Vec<Line> =
            self.vm.stack.iter().map(|e|Line::from(e.to_string())).collect();
        frame.render_widget(
            Paragraph::new(Text::from(stack_items))
                .block(panel().title(" Stack ")),
            layout.stack,
        );
        let ctrl_items: Vec<Line> =
            self.vm.ctrl.iter().map(|e|Line::from(e.to_string())).collect();
        frame.render_widget(
            Paragraph::new(Text::from(ctrl_items))
                .block(panel().title(" Control ")),
            layout.ctrl,
        );
        frame.render_widget(statusbar(), layout.statusbar);
    }

    pub fn input(&mut self, e: KeyEvent) {
        let ctrl = KeyModifiers::CONTROL;
        match e.code {
            Esc | Char('q') => self.quit(),
            Char('b') => self.toggle_breakpoint(),
            Char('s') => self.step(),
            Char('c') | Char('C') if e.modifiers == ctrl => self.quit(),
            Right => self.right(),
            Left => self.left(),
            _ => {}
        };
    }
}
