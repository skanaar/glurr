use std::collections::HashMap;
use std::env;
use std::fs;

mod model;
use model::Native;
use model::create_natives;
use model::Token;

const DEF: f64 = 0.0;
const COMPILE: f64 = 1.0;

struct Word {
    word: String,
    jump: usize,
}

struct VirtualMachine {
    natives: HashMap<&'static str, Native>,
    index: usize,
    tokens: Vec<Token>,
    stack: Vec<Token>,
    ctrl: Vec<Token>,
    syms: Vec<String>,
    strs: Vec<String>,
    vars: Vec<String>,
    dict: Vec<Word>,
}

impl VirtualMachine {
    fn new() -> Self {
        Self {
            natives: create_natives(),
            index: 0,
            tokens: Vec::new(),
            stack: Vec::new(),
            ctrl: Vec::new(),
            syms: Vec::new(),
            strs: Vec::new(),
            vars: Vec::new(),
            dict: Vec::new(),
        }
    }
}

impl VirtualMachine {
    fn in_mode(&self, mode: f64) -> bool {
        return match self.ctrl.last() {
            Some(Token::Control(x)) => *x == mode,
            _ => false,
        }
    }

    fn parse(&mut self, raw_token: &str) -> Token {
        if self.in_mode(DEF) {
            if self.syms.iter().any(|e| *e == raw_token) {
                panic!("Symbol already defined");
            }
            self.syms.push(raw_token.to_string());
            self.tokens.push(Token::Symbol(self.syms.len()));
            return Token::Native(Native::Def);
        }
        if let Some(native) = self.natives.get(raw_token) {
            return Token::Native(*native)
        }
        if let Some(i) = self.dict.iter().position(|w| w.word == raw_token) {
            return Token::Word(i)
        }
        if let Some(number) = raw_token.parse::<f64>().ok() {
            return Token::Number(number);
        }
        if raw_token.starts_with("\"") && raw_token.ends_with("\"") {
            self.strs.push(raw_token.to_string());
            return Token::String(self.strs.len());
        }
        return Token::Empty
    }

    fn evaluate(&mut self, token: Token) -> usize {
        if self.in_mode(DEF) {
            return self.index + 1;
        }
        if self.in_mode(COMPILE) {
            if token == Token::Native(Native::CloseBrace) {
                self.ctrl.pop();
            }
            return self.index + 1;
        }
        match token {
            Token::Native(Native::Def) => self.ctrl.push(Token::Control(DEF)),
            Token::Native(Native::OpenBrace) => {
                self.ctrl.push(Token::Control(COMPILE));
                self.stack.push(Token::Jump(self.index + 1));
            }
            Token::Native(Native::CloseBrace) => {
                if let Some(Token::Jump(ret)) = self.ctrl.pop() {
                    return ret;
                }
            }
            Token::Native(Native::Dot) => {
                println!("{}", self.stack.pop().unwrap().to_string());
            }
            Token::Native(Native::Dots) => {
                for item in self.stack.clone() {
                    println!("{}", item.to_string());
                }
            }
            tok => { self.stack.push(tok) }

        }
        return self.index + 1;
    }

    fn interpret(&mut self, source: String) {
        let raw_tokens: Vec<&str> = source
            .split(char::is_whitespace)
            .map(|s| s)
            .collect();
        self.index = 0;
        while self.index < raw_tokens.len() {
            let token = if self.index < self.tokens.len() {
                self.tokens[self.index]
            } else {
                self.parse(raw_tokens[self.index])
            };
            self.index = self.evaluate(token);
        }
    }

    // Glurr source code
    // def foo { 8 } ;
    // ----
    // [stack] c[control stack] {dict} "glurrcode"
    // [] c[] {} ""
    // [] c[def] "native:def "
    // [0] c[] {0:foo:nil} "native:def word:0"
    // [0,3] c[compile] {0:foo:nil} "native:def word:0 native:{"
    // [0,3] c[compile] {0:foo:nil} "native:def word:0 native:{ num:8"
    // [0,3] c[] {0:foo:nil} "native:def word:0 native:{ num:8 native:}"
    // [] c[] {0:foo:3} "native:def word:0 native:{ num:8 native:} native:;"
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    let mut vm = VirtualMachine::new();
    vm.interpret(contents);

}
