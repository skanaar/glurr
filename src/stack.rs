use crate::model;
use model::Token;

pub trait Stack<T> {
    fn popp(&mut self) -> Token;
    fn pop_num(&mut self) -> f64;
    fn pop_jump(&mut self) -> usize;
    fn pop_bool(&mut self) -> bool;
}

impl Stack<Token> for Vec<Token> {
    fn popp(&mut self) -> Token {
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

    fn pop_bool(&mut self) -> bool {
        if let Some(Token::Bool(value)) = self.pop() { return value }
        panic!("expected a bool");
    }
}
