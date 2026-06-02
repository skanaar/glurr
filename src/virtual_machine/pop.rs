use crate::{model::Token, stack::Stack};
use super::VirtualMachine;

impl VirtualMachine {
    pub fn pop_token(&mut self) -> Token {
        let token = self.stack.pop();
        if let Some(token) = token { return token }
        self.stop("stack is empty", token);
    }

    pub fn pop_num(&mut self) -> f64 {
        let token = self.stack.pop();
        if let Some(Token::Number(value)) = token { return value }
        self.stop("expected a number", token);
    }

    pub fn pop_bool(&mut self) -> bool {
        let token = self.stack.pop();
        if let Some(Token::Bool(value)) = token { return value }
        self.stop("expected a bool", token);
    }

    pub fn pop_str(&mut self) -> usize {
        let token = self.stack.pop();
        if let Some(Token::Str(value)) = token { return value }
        self.stop("expected a string", token);
    }

    pub fn pop_jump(&mut self) -> usize {
        let token = self.stack.pop();
        if let Some(Token::Jump(value)) = token { return value }
        self.stop("expected a jump", token);
    }

    pub fn pop_var(&mut self) -> usize {
        let token = self.stack.pop();
        if let Some(Token::Var(value)) = token { return value }
        self.stop("expected a variable", token);
    }

    pub fn pop_array(&mut self) -> usize {
        let token = self.stack.pop();
        if let Some(Token::Array(value)) = token { return value }
        self.stop("expected an array", token);
    }

    fn stop(&self, msg: &'static str, token: Option<Token>) -> ! {
        let word = self.tokens[self.index];
        let word_desc = self.serialize_token(&word);
        println!("\x1b[91m'{}' {}\x1b[0m", word_desc, msg);
        if self.flag_trace {
            let context = self.tokens[(self.index-5)..(self.index+1)].iter();
            let strings: Vec<String> = context.map(|x| self.serialize_token(x)).collect();
            println!("\x1b[93m{}\x1b[0m", strings.join(" "));
            let mut data_stack = self.stack.clone();
            if let Some(val) = token { data_stack.push(val); }
            print!("data stack: "); data_stack.print();
            print!("ctrl stack: "); self.ctrl.print();
            print!("loop stack: "); self.loops.print();
            println!("token pointer {}", self.index);
            panic!("{}", msg);
        }
        panic!("{}. run with --debug to inspect stacks", msg);
    }
}
