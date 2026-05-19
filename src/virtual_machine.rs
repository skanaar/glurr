use std::collections::HashMap;

use crate::model;
use model::{create_natives, Mode, Nat, Token};
use model::Token::*;

mod evaluate_native;

pub struct DictEntry {
    symbol: usize,
    jump: usize,
}

pub struct VirtualMachine {
    natives: HashMap<&'static str, Nat>,
    index: usize,
    tokens: Vec<Token>,
    stack: Vec<Token>,
    ctrl: Vec<Token>,
    loops: Vec<Token>,
    syms: Vec<String>,
    strs: Vec<String>,
    dict: Vec<DictEntry>,
    vars: Vec<Token>,
    arrays: Vec<Vec<f64>>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            natives: create_natives(),
            index: 0,
            tokens: Vec::new(),
            stack: Vec::new(),
            ctrl: Vec::new(),
            loops: Vec::new(),
            syms: Vec::new(),
            strs: Vec::new(),
            dict: Vec::new(),
            vars: Vec::new(),
            arrays: Vec::new(),
        }
    }

    pub fn interpret(&mut self, source: String) {
        let raw_tokens: Vec<&str> = source
            .split(char::is_whitespace)
            .filter(|s| !s.is_empty())
            .collect();
        self.index = 0;
        let mut source_index = 0;
        while source_index < raw_tokens.len() {
            let token = if self.index < self.tokens.len() {
                self.tokens[self.index]
            } else {
                let token = self.parse(raw_tokens[source_index]);
                source_index += 1;
                self.tokens.push(token);
                token
            };
            self.index = self.evaluate(token);
        }
    }

    pub fn parse(&mut self, raw_token: &str) -> Token {
        if let Some(Control(Mode::Comment)) = self.ctrl.last() {
            if let Some(Nat::CloseParen) = self.natives.get(raw_token) {
                self.ctrl.pop();
            }
            return Empty
        }
        if let Some(Control(Mode::Def)) = self.ctrl.last() {
            self.ctrl.pop();
            if self.syms.iter().any(|e| *e == raw_token) {
                panic!("Symbol {} already defined", raw_token);
            }
            self.syms.push(raw_token.to_string());
            return Symbol(self.syms.len() - 1);
        }
        if let Some(Control(Mode::Var)) = self.ctrl.last() {
            self.ctrl.pop();
            if self.syms.iter().any(|e| *e == raw_token) {
                panic!("Symbol {} already defined", raw_token);
            }
            self.vars.push(Number(0.));
            self.syms.push(raw_token.to_string());
            self.tokens.push(Empty);
            self.tokens.push(Symbol(self.syms.len() - 1));
            self.tokens.push(Native(Nat::OpenBrace));
            self.tokens.push(Var(self.vars.len() - 1));
            self.tokens.push(Native(Nat::CloseBrace));
            self.tokens.push(Native(Nat::Semicolon));
            return Empty;
        }
        // native
        if let Some(native) = self.natives.get(raw_token) {
            return Native(*native)
        }
        // number
        if let Some(number) = raw_token.parse::<f64>().ok() {
            return Number(number);
        }
        // string
        if raw_token.starts_with("\"") && raw_token.ends_with("\"") {
            self.strs.push(raw_token.to_string());
            return Str(self.strs.len());
        }
        // word in dict
        if let Some(symb_i) = self.syms.iter().position(|w| w == raw_token) {
            if let Some(entry) = self.dict.iter().find(|e| e.symbol == symb_i) {
                return Jump(entry.jump)
            }
        }
        panic!("unknown word '{}'", raw_token)
    }

    pub fn evaluate(&mut self, token: Token) -> usize {
        if let Some(Control(Mode::Def)) = self.ctrl.last() {
            return self.index + 1;
        }
        if let Some(Control(Mode::Quote)) = self.ctrl.last() {
            self.ctrl.pop();
            self.stack.push(token);
            return self.index + 1;
        }
        if let Some(Control(Mode::Compile)) = self.ctrl.last() {
            if token == Native(Nat::OpenBrace) {
                self.ctrl.push(Control(Mode::Compile));
            }
            else if token == Native(Nat::CloseBrace) {
                self.ctrl.pop();
            }
            return self.index + 1;
        }
        match token {
            Native(nat) => return self.evaluate_native(nat),
            Number(x) => self.stack.push(Number(x)),
            Var(x) => self.stack.push(Var(x)),
            Array(x) => self.stack.push(Array(x)),
            Control(_) => panic!("cannot evaluate a control token"),
            Jump(index) => {
                self.ctrl.push(Jump(self.index + 1));
                return index;
            }
            Bool(x) => self.stack.push(Bool(x)),
            Empty => {},
            Symbol(index) => self.stack.push(Symbol(index)),
            Str(_) => panic!("strings not supported"),
        }
        return self.index + 1;
    }
}
