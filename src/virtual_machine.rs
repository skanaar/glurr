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
    vars: Vec<Token>,
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
            vars: Vec::new(),
        }
    }

    pub fn interpret(&mut self, source: String) {
        let raw_tokens: Vec<&str> = source
            .split(char::is_whitespace)
            .map(|s| s)
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
        if let Some(Control(Mode::Quote)) = self.ctrl.last() {
            if let Word(word) = token {
                self.ctrl.pop();
                let jump = self.dict[word].jump;
                self.stack.push(Jump(jump));
                return self.index + 1;
            }
            panic!("expected a word");
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
            Empty => {},
            Symbol(index) => self.stack.push(Symbol(index)),
            Str(_) => panic!("strings not supported"),
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
                let prod = self.stack.pop_num() * self.stack.pop_num();
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
                self.stack.print();
            }
            CtrlDots => {
                self.ctrl.print();
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
                } else {
                    panic!("stack is empty");
                }
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
                self.stack.push(a.clone());
            }
            Include => todo!("Include"),
            Debug => {}
            Def => {
                self.ctrl.push(Control(Mode::Def));
            }
            Var => {
                self.ctrl.push(Control(Mode::Var));
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
            StoreCtrl => self.ctrl.push(self.stack.popp()),
            ReadCtrl => self.stack.push(self.ctrl.popp()),
            CopyCtrl => {
                let val = self.ctrl.popp();
                self.ctrl.push(val.clone());
                self.stack.push(val.clone());
            },
            Invoke => {
                let jump = self.stack.pop_jump();
                self.ctrl.push(Jump(self.index + 1));
                return jump;
            }
            ByteArray => todo!("ByteArray"),
            Set => todo!("Set"),
            Get => todo!("Get"),
            DisplayImage => todo!("DisplayImage"),
            Questionmark => {
                let false_val = self.stack.popp();
                let true_val = self.stack.popp();
                let cond = self.stack.pop_bool();
                self.stack.push(if cond { true_val } else { false_val });
            }
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
            OpenParen => { self.ctrl.push(Control(Mode::Comment)) }
            CloseParen => todo!("CloseParen"),
            Comment => todo!("Comment"),
            Dot => println!("{}", self.stack.popp().to_string()),
            Equal => {
                let right = self.stack.pop_num();
                let left = self.stack.pop_num();
                self.stack.push(Bool(left == right));
            }
            GreaterThan => {
                let right = self.stack.pop_num();
                let left = self.stack.pop_num();
                self.stack.push(Bool(left > right));
            }
            LessThan => {
                let right = self.stack.pop_num();
                let left = self.stack.pop_num();
                self.stack.push(Bool(left < right));
            }
            Not => {
                let cond = self.stack.pop_bool();
                self.stack.push(Bool(!cond));
            }
            True => self.stack.push(Bool(true)),
            False => self.stack.push(Bool(false)),
            Assign => todo!("Assign"),
            Read => {
                let index = self.stack.pop_var();
                if let Number(value) = self.vars[index] {
                    self.stack.push(Number(value));
                } else {
                    panic!("vars can only be bound to numbers")
                }
            },
            Write => {
                let index = self.stack.pop_var();
                let value = self.stack.pop_num();
                self.vars[index] = Number(value);
            },
        }
        return self.index + 1;
    }
}
