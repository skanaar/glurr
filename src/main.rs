use std::collections::HashMap;
use std::env;
use std::fs;

mod model;
use model::Nat;
use model::Mode;
use model::create_natives;
use model::Token;
use model::Token::*;

struct DictEntry {
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
    dict: Vec<DictEntry>,
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
            dict: Vec::new(),
        }
    }
}

impl VirtualMachine {
    fn in_mode(&self, mode: Mode) -> bool {
        return match self.ctrl.last() {
            Some(Control(m)) => *m == mode,
            _ => false,
        }
    }

    fn pop_num(&mut self) -> f64 {
        if let Some(Token::Number(value)) = self.stack.pop() {
            return value;
        }
        panic!("stack is empty");
    }

    fn pop(&mut self) -> Token {
        if let Some(token) = self.stack.pop() {
            return token;
        }
        panic!("stack is empty");
    }

    fn parse(&mut self, raw_token: &str) -> Token {
        if self.in_mode(Mode::Def) {
            self.ctrl.pop();
            if self.syms.iter().any(|e| *e == raw_token) {
                panic!("Symbol {} already defined", raw_token);
            }
            self.syms.push(raw_token.to_string());
            return Symbol(self.syms.len() - 1);
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
        if self.in_mode(Mode::Def) {
            return self.index + 1;
        }
        if self.in_mode(Mode::Compile) {
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
            Control(_) => panic!("cannot evaluate a control token"),
            Jump(index) => {
                self.ctrl.push(Jump(self.index + 1));
                return index;
            }
            Token::Word(i) => {
                self.ctrl.push(Jump(self.index + 1));
                return self.dict[i].jump
            }
            Bool(x) => self.stack.push(Bool(x)),
            tok => self.stack.push(tok)
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
            Drop => { self.stack.pop(); }
            Swap => {
                let a = self.pop();
                let b = self.pop();
                self.stack.push(a);
                self.stack.push(b);
            }
            Rot => {
                let a = self.pop();
                let b = self.pop();
                let c = self.pop();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(c);
            }
            Pick => {
                let offset = self.stack.len() - self.pop_num() as usize;
                if let Some(token) = self.stack.get(offset) {
                    self.stack.push(token.clone());
                }
                panic!("stack is empty");
            }
            Over => {
                let a = self.pop();
                let b = self.pop();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(b);
            }
            Dup => {
                let a = self.pop();
                self.stack.push(a);
                self.stack.push(a);
            }
            Include => todo!("Include"),
            Debug => todo!("Debug"),
            Def => {
                self.ctrl.push(Control(Mode::Def));
            }
            Quote => self.ctrl.push(Control(Mode::Quote)),
            OpenBrace => {
                self.ctrl.push(Control(Mode::Compile));
                self.stack.push(Jump(self.index + 1));
            }
            CloseBrace => {
                if let Some(Jump(index)) = self.ctrl.pop() {
                    return index;
                }
                panic!("no return jump on ctrl stack");
            }
            Semicolon => {
                if let Some(Jump(jump)) = self.stack.pop() {
                    if let Some(Symbol(i)) = self.stack.pop() {
                        let symbol = &self.syms[i];
                        self.dict.push(DictEntry {
                            word: symbol.clone(),
                            jump: jump
                        })
                    } else { panic!("; requires a symbol") }
                } else { panic!("; requires a symbol") }
            }
            Invoke => todo!("Invoke"),
            ByteArray => todo!("ByteArray"),
            Set => todo!("Set"),
            Get => todo!("Get"),
            DisplayImage => todo!("DisplayImage"),
            Questionmark => todo!("Questionmark"),
            If => todo!("If"),
            Loop => todo!("Loop"),
            Range => todo!("Range"),
            Enumerate => todo!("Enumerate"),
            LeaveIf => todo!("LeaveIf"),
            I => todo!("I"),
            OpenParen => todo!("OpenParen"),
            Comment => todo!("Comment"),
            StoreCtrl => todo!("StoreCtrl"),
            ReadCtrl => todo!("ReadCtrl"),
            CopyCtrl => todo!("CopyCtrl"),
            Dot => todo!("Dot"),
            Equal => todo!("Equal"),
            GreaterThan => todo!("GreaterThan"),
            LessThan => todo!("LessThan"),
            Not => todo!("Not"),
            True => self.stack.push(Bool(true)),
            False => self.stack.push(Bool(false)),
            Assign => todo!("Assign"),
            Read => todo!("Read"),
            Write => todo!("Write"),
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
                let token = self.parse(raw_tokens[self.index]);
                self.tokens.push(token);
                token
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
