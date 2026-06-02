use crate::model;
use model::Token;

pub trait Stack<T> {
    fn pop_token(&mut self) -> Token;
    fn pop_num(&mut self) -> f64;
    fn pop_jump(&mut self) -> usize;
    fn print(&self);
}

impl Stack<Token> for Vec<Token> {
    fn pop_token(&mut self) -> Token {
        if let Some(token) = self.pop() { return token }
        panic!("stack is empty");
    }

    fn pop_num(&mut self) -> f64 {
        if let Some(Token::Number(value)) = self.pop() { return value }
        panic!("expected a number");
    }

    fn pop_jump(&mut self) -> usize {
        if let Some(Token::Jump(value)) = self.pop() { return value }
        panic!("expected a jump");
    }

    fn print(&self) {
        let strings: Vec<String> = self.iter().map(|x| x.to_string()).collect();
        println!("<{}> {}", self.len(), strings.join(" "));
    }
}
