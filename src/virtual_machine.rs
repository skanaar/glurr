use std::collections::HashMap;

use crate::model;
use model::{create_natives, Mode, Nat, Token};
use model::Token::*;

mod evaluate_native;
mod pop;

pub struct DictEntry {
    symbol: usize,
    jump: usize,
}

pub struct Included {
    pub source_index: usize,
    pub tokens: Vec<String>,
}

pub struct VirtualMachine {
    pub flag_trace: bool,
    natives: HashMap<&'static str, Nat>,
    includeables: HashMap<String, String>,
    include_stack: Vec<Included>,
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
            flag_trace: false,
            natives: create_natives(),
            includeables: HashMap::new(),
            include_stack: Vec::new(),
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

    pub fn include(&mut self, name: String, content: String) {
        self.includeables.insert(name.clone(), content);
    }

    fn current_source(&self) -> &Vec<String> {
        return &self.include_stack.last().unwrap().tokens;
    }

    fn src_pointer(&self) -> usize {
        return self.include_stack.last().unwrap().source_index;
    }

    fn move_src_pointer(&mut self) {
        let last = self.include_stack.len() - 1;
        let i = self.src_pointer();
        self.include_stack[last].source_index = i+1;
        if self.src_pointer() >= self.current_source().len() {
            self.include_stack.pop();
        }
    }

    fn source_stack_push(&mut self, tokens: Vec<String>) {
        if tokens.len() > 0 {
            self.include_stack.push(Included { source_index: 0, tokens });
        }
    }

    pub fn interpret(&mut self, source: String) {
        let raw_tokens: Vec<String> = source
            .split(char::is_whitespace)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        self.index = 0;
        self.include_stack.push(Included { source_index: 0, tokens: raw_tokens });
        while self.include_stack.len() > 0 && self.src_pointer() < self.current_source().len() {
            while self.index < self.tokens.len() {
                let token = self.tokens[self.index];
                self.index = self.evaluate(token);
            }
            let src_i = self.src_pointer();
            let raw = self.current_source()[src_i].clone();
            let token = self.parse(&raw);
            self.move_src_pointer();
            self.tokens.push(token);
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
            self.strs.push(raw_token[1..raw_token.len()-1].to_string());
            return Str(self.strs.len() - 1);
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
            Str(index) => self.stack.push(Str(index)),
        }
        return self.index + 1;
    }

    pub fn serialize_token(&self, token: &Token) -> String {
        match token {
            Jump(jmp) => {
                if let Some(word) = self.dict.iter().find(|e| e.jump == *jmp) {
                    return format!("Jump({})", self.syms[word.symbol])
                } else {
                    return format!("Jump({})", jmp);
                }
            },
            _ => return format!("{}", token.to_string())
        }
    }
}
