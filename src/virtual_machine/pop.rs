use crate::model::Token;
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
        let tok = if let Some(val) = token { val.to_string() } else { "-".to_string() };
        println!("\x1b[91m'{}' {} found {}\x1b[0m", word_desc, msg, tok);
        self.panic(msg);
    }
}
