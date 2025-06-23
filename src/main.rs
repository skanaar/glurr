use std::collections::HashMap;
use std::env;
use std::fs;

mod model;
use model::Nat;
use model::create_natives;
use model::Token;
use model::Token::*;

const DEF: f64 = 0.0;
const COMPILE: f64 = 1.0;

struct Word {
    word: String,
    jump: usize,
}

struct VirtualMachine {
    natives: HashMap<&'static str, Nat>,
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
            Some(Control(x)) => *x == mode,
            _ => false,
        }
    }

    fn pop_num(&mut self) -> f64 {
        if let Some(Token::Number(value)) = self.stack.pop() {
            return value;
        }
        panic!("stack is empty");
    }

    fn parse(&mut self, raw_token: &str) -> Token {
        if self.in_mode(DEF) {
            if self.syms.iter().any(|e| *e == raw_token) {
                panic!("Symbol already defined");
            }
            self.syms.push(raw_token.to_string());
            self.tokens.push(Symbol(self.syms.len()));
            return Native(Nat::Def);
        }
        if let Some(native) = self.natives.get(raw_token) {
            return Native(*native)
        }
        if let Some(i) = self.dict.iter().position(|w| w.word == raw_token) {
            return Word(i)
        }
        if let Some(number) = raw_token.parse::<f64>().ok() {
            return Number(number);
        }
        if raw_token.starts_with("\"") && raw_token.ends_with("\"") {
            self.strs.push(raw_token.to_string());
            return Str(self.strs.len());
        }
        return Empty
    }

    fn evaluate(&mut self, token: Token) -> usize {
        if self.in_mode(DEF) {
            return self.index + 1;
        }
        if self.in_mode(COMPILE) {
            if token == Native(Nat::CloseBrace) {
                self.ctrl.pop();
            }
            return self.index + 1;
        }
        match token {
            Native(nat) => {
                return self.evaluate_native(nat);
            }
            tok => { self.stack.push(tok) }

        }
        return self.index + 1;
    }

    fn evaluate_native(&mut self, native: Nat) -> usize {
        use model::Nat::*;
        match native {
            Plus => {
                let sum = self.pop_num() + self.pop_num();
                self.stack.push(Token::Number(sum));
            }
            Minus => {
                let rhs = self.pop_num();
                let lhs = self.pop_num();
                self.stack.push(Token::Number(lhs - rhs));
            }
            Multiply => {
                let prod = self.pop_num() + self.pop_num();
                self.stack.push(Token::Number(prod));
            }
            Divide => {
                let rhs = self.pop_num();
                let lhs = self.pop_num();
                self.stack.push(Token::Number(lhs / rhs));
            }
            Pow => {
                let rhs = self.pop_num();
                let lhs = self.pop_num();
                self.stack.push(Token::Number(lhs.powf(rhs)));
            }
            Mod => {
                let rhs = self.pop_num();
                let lhs = self.pop_num();
                self.stack.push(Token::Number(lhs.rem_euclid(rhs)));
            }
            Floor => {
                let value = self.pop_num();
                self.stack.push(Token::Number(value.floor()));
            }
            Ceil => {
                let value = self.pop_num();
                self.stack.push(Token::Number(value.ceil()));
            }
            Round => {
                let value = self.pop_num();
                self.stack.push(Token::Number(value.round()));
            }
            Abs => {
                let value = self.pop_num();
                self.stack.push(Token::Number(value.abs()));
            }
            Neg => {
                let value = self.pop_num();
                self.stack.push(Token::Number(-value));
            }
            Dots => {
                for item in self.stack.clone() {
                    println!("{}", item.to_string());
                }
            }
            Include => todo!(),
            Debug => todo!(),
            Def => todo!(),
            Quote => todo!(),
            OpenBrace => todo!(),
            CloseBrace => todo!(),
            Semicolon => todo!(),
            Invoke => todo!(),
            ByteArray => todo!(),
            Set => todo!(),
            Get => todo!(),
            DisplayImage => todo!(),
            Questionmark => todo!(),
            If => todo!(),
            Loop => todo!(),
            Range => todo!(),
            Enumerate => todo!(),
            LeaveIf => todo!(),
            I => todo!(),
            OpenParen => todo!(),
            Comment => todo!(),
            StoreCtrl => todo!(),
            ReadCtrl => todo!(),
            CopyCtrl => todo!(),
            Dot => todo!(),
            Equal => todo!(),
            GreaterThan => todo!(),
            LessThan => todo!(),
            Not => todo!(),
            True => todo!(),
            False => todo!(),
            Drop => todo!(),
            Swap => todo!(),
            Rot => todo!(),
            Pick => todo!(),
            Over => todo!(),
            Dup => todo!(),
            Assign => todo!(),
            Read => todo!(),
            Write => todo!(),
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
