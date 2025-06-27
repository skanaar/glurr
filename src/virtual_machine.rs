use std::collections::HashMap;

use crate::stack;
use stack::Stack;

use crate::model;
use model::{create_natives, Mode, Nat, Token};
use model::Token::*;

pub struct DictEntry {
    word: String,
    jump: usize,
}

pub struct VirtualMachine {
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
    pub fn new() -> Self {
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

    pub fn interpret(&mut self, source: String) {
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

    pub fn parse(&mut self, raw_token: &str) -> Token {
        if let Some(Control(Mode::Def)) = self.ctrl.last() {
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

    pub fn evaluate(&mut self, token: Token) -> usize {
        if let Some(Control(Mode::Def)) = self.ctrl.last() {
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

    pub fn evaluate_native(&mut self, native: Nat) -> usize {
        use model::Nat::*;
        match native {
            Plus => {
                let sum = self.stack.pop_num() + self.stack.pop_num();
                self.stack.push(Token::Number(sum));
            }
            Minus => {
                let rhs = self.stack.pop_num();
                let lhs = self.stack.pop_num();
                self.stack.push(Token::Number(lhs - rhs));
            }
            Multiply => {
                let prod = self.stack.pop_num() + self.stack.pop_num();
                self.stack.push(Token::Number(prod));
            }
            Divide => {
                let rhs = self.stack.pop_num();
                let lhs = self.stack.pop_num();
                self.stack.push(Token::Number(lhs / rhs));
            }
            Pow => {
                let rhs = self.stack.pop_num();
                let lhs = self.stack.pop_num();
                self.stack.push(Token::Number(lhs.powf(rhs)));
            }
            Mod => {
                let rhs = self.stack.pop_num();
                let lhs = self.stack.pop_num();
                self.stack.push(Token::Number(lhs.rem_euclid(rhs)));
            }
            Floor => {
                let value = self.stack.pop_num();
                self.stack.push(Token::Number(value.floor()));
            }
            Ceil => {
                let value = self.stack.pop_num();
                self.stack.push(Token::Number(value.ceil()));
            }
            Round => {
                let value = self.stack.pop_num();
                self.stack.push(Token::Number(value.round()));
            }
            Abs => {
                let value = self.stack.pop_num();
                self.stack.push(Token::Number(value.abs()));
            }
            Neg => {
                let value = self.stack.pop_num();
                self.stack.push(Token::Number(-value));
            }
            Dots => {
                for item in self.stack.clone() {
                    println!("{}", item.to_string());
                }
            }
            CtrlDots => {
                for item in self.ctrl.clone() {
                    println!("{}", item.to_string());
                }
            }
            Drop => { self.stack.pop(); }
            Swap => {
                let a = self.stack.popp();
                let b = self.stack.popp();
                self.stack.push(a);
                self.stack.push(b);
            }
            Rot => {
                let a = self.stack.popp();
                let b = self.stack.popp();
                let c = self.stack.popp();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(c);
            }
            Pick => {
                let offset = self.stack.len() - self.stack.pop_num() as usize;
                if let Some(token) = self.stack.get(offset) {
                    self.stack.push(token.clone());
                }
                panic!("stack is empty");
            }
            Over => {
                let a = self.stack.popp();
                let b = self.stack.popp();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(b);
            }
            Dup => {
                let a = self.stack.popp();
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
            StoreCtrl => {
                let value = self.stack.popp();
                self.ctrl.push(value)
            },
            ReadCtrl => {
                if let Some(value) = self.ctrl.pop() {
                    self.stack.push(value)
                }
            },
            CopyCtrl => {
                if let Some(value) = self.ctrl.last() {
                    self.stack.push(value.clone())
                }
            },
            Invoke => todo!("Invoke"),
            ByteArray => todo!("ByteArray"),
            Set => todo!("Set"),
            Get => todo!("Get"),
            DisplayImage => todo!("DisplayImage"),
            Questionmark => todo!("Questionmark"),
            If => {
                let no = self.stack.pop_jump();
                let yes = self.stack.pop_jump();
                let cond = self.stack.pop_bool();
                return if cond { yes } else { no }
            },
            Loop => todo!("Loop"),
            Range => {
                self.ctrl.push(Number(self.stack.pop_num()));
                self.ctrl.push(Number(self.stack.pop_num()));
                self.ctrl.push(Jump(self.stack.pop_jump()));
            }
            Enumerate => {
                let jump = self.ctrl.pop_jump();
                let from = self.ctrl.pop_num();
                let to = self.ctrl.pop_num();
                if from < to {
                    self.ctrl.push(Number(to));
                    self.ctrl.push(Number(from + 1.));
                    self.ctrl.push(Jump(jump));
                    self.ctrl.push(Jump(self.index));
                    return jump
                }
            }
            LeaveIf => {
                if self.stack.pop_bool() {
                    let jump = self.ctrl.pop_jump();
                    self.ctrl.pop_jump();
                    self.ctrl.pop_num();
                    self.ctrl.pop_num();
                    return jump + 1
                }
            }
            I => {
                if let Some(Number(i)) = self.ctrl.get(self.ctrl.len() - 3) {
                    self.stack.push(Number(i - 1.));
                } else {
                    panic!("expected a number");
                }
            }
            OpenParen => todo!("OpenParen"),
            Comment => todo!("Comment"),
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
}
