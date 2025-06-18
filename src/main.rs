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

    fn interpret(&mut self, source: Vec<&str>) {
        while self.index < source.len() {
            self.index = self.evaluate(self.get_token(source, self.index));
        }
    }

    fn get_token(&mut self, source: Vec<&str>, index: usize) -> Token {
        if let Some(token) = self.tokens.get(index) {
            return token.clone();
        } else if let Some(raw_token) = source.get(index) {
            let token = self.parse(*raw_token);
            self.tokens.push(token.clone());
            return token;
        } else {
            return Token::End;
        }
    }

    fn evaluate(&self, token: Token) -> usize {
        if self.in_mode(DEF) {
            if self.syms.iter().any(|e| e == token) {
                panic!("Symbol already defined");
            }
            self.syms.push(token.to_string());
            self.tokens.push(Token::Symbol(self.syms.len()));
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
            tok => { self.stack.push(tok) }

        }
        return self.index + 1;
    }

    fn parse(&self, raw_token: &str) -> Token {
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
            return Token::String(raw_token.to_string());
        }
        return Token::Empty
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    let mut vm = VirtualMachine::new();

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

    // interpret
    for token in contents.split(char::is_whitespace) {
        if token == "" { vm.index += 1; continue; }
        else if token == ";" {
            // ...
        }
        println!("{}", token);
    }

    // execute
    for token in &vm.tokens {
        match token {
            Token::Number(x) => println!("Number: {}", x),
            Token::Jump(x) => println!("Jump {}", x),
            Token::Symbol(x) => println!("Symbol {}", x),
            Token::Word(x) => println!("Word {}", x),
            Token::Bool(x) => println!("Bool {}", x),
            Token::String(x) => println!("String {}", x),
            Token::Control(x) => println!("Control {}", x),
            Token::Native(_) => println!("Native"),
            Token::Empty => println!("Empty"),
            Token::End => println!("End"),
        }
    }
}
