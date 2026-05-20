use image::ImageBuffer;
use crate::stack::Stack;
use crate::model::{self, Nat};
use crate::model::Mode;
use crate::model::Token::*;
use crate::model::Token;
use super::VirtualMachine;

impl VirtualMachine {
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
                let a = self.stack.pop_token();
                let b = self.stack.pop_token();
                self.stack.push(a);
                self.stack.push(b);
            }
            Rot => {
                let a = self.stack.pop_token();
                let b = self.stack.pop_token();
                let c = self.stack.pop_token();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(c);
            }
            Pick => {
                let offset = self.stack.pop_num();
                let index = self.stack.len() - offset as usize;
                if let Some(token) = self.stack.get(index) {
                    self.stack.push(token.clone());
                } else {
                    panic!("stack is empty");
                }
            }
            Over => {
                let a = self.stack.pop_token();
                let b = self.stack.pop_token();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(b);
            }
            Dup => {
                let a = self.stack.pop_token();
                self.stack.push(a);
                self.stack.push(a);
            }
            Include => {
                let str_i = self.stack.pop_str();
                let name = self.strs[str_i].clone();
                let Some(content) = self.includeables.get(&name) else {
                    panic!("include name not listed at startup")
                };
                let tokens: Vec<String> = content
                    .split(char::is_whitespace)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
                self.source_stack_push(tokens);
            }
            Debug => {}
            Def => {
                self.ctrl.push(Control(Mode::Def));
            }
            Var => {
                self.ctrl.push(Control(Mode::Var));
            }
            Consume => todo!("todo"),
            Quote => self.ctrl.push(Control(Mode::Quote)),
            Emit => self.tokens.push(self.stack.pop_token()),
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
                    if let Some(Symbol(symb_i)) = self.stack.pop() {
                        self.dict.push(super::DictEntry {
                            symbol: symb_i,
                            jump: jump
                        })
                    } else { panic!("; requires a symbol") }
                } else { panic!("; requires a jump") }
            }
            StoreCtrl => self.ctrl.push(self.stack.pop_token()),
            ReadCtrl => self.stack.push(self.ctrl.pop_token()),
            CopyCtrl => {
                let val = self.ctrl.pop_token();
                self.ctrl.push(val.clone());
                self.stack.push(val.clone());
            },
            Invoke => {
                let jump = self.stack.pop_jump();
                self.ctrl.push(Jump(self.index + 1));
                return jump;
            }
            Allot => {
                let len = self.stack.pop_num() as usize;
                let mut array: Vec<f64> = Vec::with_capacity(len);
                for _ in 0..len { array.push(0.) }
                self.arrays.push(array);
                self.stack.push(Array(self.arrays.len() - 1))
            }
            Set => {
                let array_ref = self.stack.pop_array() as usize;
                let index = self.stack.pop_num() as usize;
                let value = self.stack.pop_num() as f64;
                let array = &mut self.arrays[array_ref];
                array[index] = value;
            }
            Get => {
                let array_ref = self.stack.pop_array() as usize;
                let index = self.stack.pop_num() as usize;
                let array = &mut self.arrays[array_ref];
                self.stack.push(Number(array[index] as f64));
            }
            DisplayImage => {
                let width = self.stack.pop_num() as u32;
                let array_ref = self.stack.pop_array();
                let array = &mut self.arrays[array_ref];
                let height = array.len() as u32 / (width * 4);
                let img = ImageBuffer::from_fn(width, height, |x, y| {
                    let i = 4 * (x + width * y) as usize;
                    image::Rgb([
                        array[i+0] as u8,
                        array[i+1] as u8,
                        array[i+2] as u8
                    ])
                });
                let res = img.save("./output.png");
                res.expect("failed to write image")
            }
            Questionmark => {
                let false_val = self.stack.pop_token();
                let true_val = self.stack.pop_token();
                let cond = self.stack.pop_bool();
                self.stack.push(if cond { true_val } else { false_val });
            }
            If => {
                let jump = self.stack.pop_jump();
                let cond = self.stack.pop_bool();
                if cond {
                    self.ctrl.push(Jump(self.index + 1));
                    return jump
                }
            },
            Infinite => {
                self.loops.push(Number(0.));
                self.loops.push(Number(0.));
                self.loops.push(Jump(self.stack.pop_jump()));
            }
            Loop => {
                let jump = self.loops.pop_jump();
                self.loops.push(Jump(jump));
                self.ctrl.push(Jump(self.index));
                return jump
            }
            Range => {
                self.loops.push(Number(self.stack.pop_num()));
                self.loops.push(Number(self.stack.pop_num()));
                self.loops.push(Jump(self.stack.pop_jump()));
            }
            Enumerate => {
                let jump = self.loops.pop_jump();
                let from = self.loops.pop_num();
                let to = self.loops.pop_num();
                if from < to {
                    self.loops.push(Number(to));
                    self.loops.push(Number(from + 1.));
                    self.loops.push(Jump(jump));
                    self.ctrl.push(Jump(self.index));
                    return jump
                }
            }
            LeaveIf => {
                if self.stack.pop_bool() {
                    let jump = self.ctrl.pop_jump();
                    self.loops.pop_jump();
                    self.loops.pop_num();
                    self.loops.pop_num();
                    return jump + 1
                }
            }
            I => {
                if let Some(Number(i)) = self.loops.get(self.loops.len() - 2) {
                    self.stack.push(Number(i - 1.));
                } else {
                    panic!("expected a number");
                }
            }
            OpenParen => { self.ctrl.push(Control(Mode::Comment)) }
            CloseParen => panic!("unexpected CloseParen"),
            Dot => {
                let token = self.stack.pop_token();
                match token {
                    Str(si) => println!("\"{}\"", self.strs[si]),
                    _ => println!("{}", token.to_string())
                }
            },
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
            Read => {
                let index = self.stack.pop_var();
                let token = self.vars[index];
                self.stack.push(token);
            },
            Write => {
                let index = self.stack.pop_var();
                let token = self.stack.pop_token();
                self.vars[index] = token;
            },
            Assert => {
                let cond = self.stack.pop_bool();
                if !cond { panic!("assertion failed") }
            },
            RevealTokens => {
                for token in &self.tokens {
                    match token {
                        Jump(jmp) => {
                            if let Some(word) = self.dict.iter().find(|e| e.jump == *jmp) {
                                println!("Jump({})", self.syms[word.symbol])
                            } else {
                                println!("Jump({})", jmp);
                            }
                        },
                        _ => println!("{}", token.to_string())
                    }
                }
            }
        }
        return self.index + 1;
    }
}
