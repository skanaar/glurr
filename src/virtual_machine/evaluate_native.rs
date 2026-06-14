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
                let sum = self.pop_num() + self.pop_num();
                self.stack.push(Token::Number(sum));
            }
            Minus => {
                let rhs = self.pop_num();
                let lhs = self.pop_num();
                self.stack.push(Token::Number(lhs - rhs));
            }
            Multiply => {
                let prod = self.pop_num() * self.pop_num();
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
                self.stack.print();
            }
            CtrlDots => {
                self.ctrl.print();
            }
            Drop => { self.stack.pop(); }
            Swap => {
                let a = self.pop_token();
                let b = self.pop_token();
                self.stack.push(a);
                self.stack.push(b);
            }
            Rot => {
                let a = self.pop_token();
                let b = self.pop_token();
                let c = self.pop_token();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(c);
            }
            Pick => {
                let offset = self.pop_num();
                let index = self.stack.len() - offset as usize;
                if let Some(token) = self.stack.get(index) {
                    self.stack.push(token.clone());
                } else {
                    self.print_trace();
                    self.panic("stack is empty");
                }
            }
            Over => {
                let a = self.pop_token();
                let b = self.pop_token();
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(b);
            }
            Dup => {
                let a = self.pop_token();
                self.stack.push(a);
                self.stack.push(a);
            }
            Include => {
                let str_i = self.pop_str();
                let name = self.strs[str_i].clone();
                let Some(content) = self.includeables.get(&name).cloned() else {
                    self.panic("include not listed at startup")
                };
                self.include(&name, &content);
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
            Emit => {
                let token = self.pop_token();
                self.tokens.push(token);
            },
            OpenBrace => {
                self.ctrl.push(Control(Mode::Compile));
                self.stack.push(Jump(self.index + 1));
            }
            CloseBrace => {
                if let Some(Jump(index)) = self.ctrl.pop() {
                    return index;
                }
                self.print_trace();
                self.panic("no return jump on ctrl stack");
            }
            Semicolon => {
                if let Some(Jump(jump)) = self.stack.pop() {
                    if let Some(Symbol(symb_i)) = self.stack.pop() {
                        self.dict.push(super::DictEntry {
                            symbol: symb_i,
                            jump: jump
                        })
                    } else { self.panic("; requires a symbol") }
                } else { self.panic("; requires a jump") }
            }
            StoreCtrl => {
                let token = self.pop_token();
                self.ctrl.push(token);
            },
            ReadCtrl => self.stack.push(self.ctrl.pop_token()),
            CopyCtrl => {
                let val = self.ctrl.pop_token();
                self.ctrl.push(val.clone());
                self.stack.push(val.clone());
            },
            Invoke => {
                let jump = self.pop_jump();
                self.ctrl.push(Jump(self.index + 1));
                return jump;
            }
            Allot => {
                let len = self.pop_num() as usize;
                let mut array: Vec<f64> = Vec::with_capacity(len);
                for _ in 0..len { array.push(0.) }
                self.arrays.push(array);
                self.stack.push(Array(self.arrays.len() - 1))
            }
            JumpAsNumber => {
                let jump = self.pop_jump();
                self.stack.push(Number(jump as f64))
            },
            StringAsNumber => {
                let str = self.pop_str();
                self.stack.push(Number(str as f64))
            },
            VarAsNumber => {
                let var = self.pop_var();
                self.stack.push(Number(var as f64))
            },
            ArrayAsNumber => {
                let array = self.pop_array();
                self.stack.push(Number(array as f64))
            },
            NumberAsJump => {
                let num = self.pop_num();
                self.stack.push(Jump(num as usize))
            },
            NumberAsString => {
                let num = self.pop_num();
                self.stack.push(Str(num as usize))
            },
            NumberAsVar => {
                let num = self.pop_num();
                self.stack.push(Token::Var(num as usize))
            },
            NumberAsArray => {
                let num = self.pop_num();
                self.stack.push(Array(num as usize))
            },
            Set => {
                let array_ref = self.pop_array() as usize;
                let index = self.pop_num() as usize;
                let value = self.pop_num() as f64;
                let array = &mut self.arrays[array_ref];
                array[index] = value;
            }
            Get => {
                let array_ref = self.pop_array() as usize;
                let index = self.pop_num() as usize;
                let array = &mut self.arrays[array_ref];
                self.stack.push(Number(array[index] as f64));
            }
            DisplayImage => {
                let width = self.pop_num() as u32;
                let array_ref = self.pop_array();
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
                let false_val = self.pop_token();
                let true_val = self.pop_token();
                let cond = self.pop_bool();
                self.stack.push(if cond { true_val } else { false_val });
            }
            If => {
                let jump = self.pop_jump();
                let cond = self.pop_bool();
                if cond {
                    self.ctrl.push(Jump(self.index + 1));
                    return jump
                }
            },
            Infinite => {
                self.loops.push(Number(0.));
                self.loops.push(Number(0.));
                let jmp = self.pop_jump();
                self.loops.push(Jump(jmp));
            }
            Loop => {
                let jump = self.loops.pop_jump();
                self.loops.push(Jump(jump));
                self.ctrl.push(Jump(self.index));
                return jump
            }
            Range => {
                let to = self.pop_num();
                let from = self.pop_num();
                let jmp = self.pop_jump();
                self.loops.push(Number(to));
                self.loops.push(Number(from));
                self.loops.push(Jump(jmp));
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
                if self.pop_bool() {
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
                    self.panic("expected a number");
                }
            }
            OpenParen => { self.ctrl.push(Control(Mode::Comment)) }
            CloseParen => self.panic("unexpected CloseParen"),
            Dot => {
                let token = self.pop_token();
                match token {
                    Str(si) => print!("{}", self.strs[si]),
                    _ => print!("{}", token.to_string())
                }
            },
            Equal => {
                let right = self.pop_num();
                let left = self.pop_num();
                self.stack.push(Bool(left == right));
            }
            GreaterThan => {
                let right = self.pop_num();
                let left = self.pop_num();
                self.stack.push(Bool(left > right));
            }
            LessThan => {
                let right = self.pop_num();
                let left = self.pop_num();
                self.stack.push(Bool(left < right));
            }
            Not => {
                let cond = self.pop_bool();
                self.stack.push(Bool(!cond));
            }
            True => self.stack.push(Bool(true)),
            False => self.stack.push(Bool(false)),
            Read => {
                let index = self.pop_var();
                let token = self.vars[index];
                self.stack.push(token);
            },
            Write => {
                let index = self.pop_var();
                let token = self.pop_token();
                self.vars[index] = token;
            },
            Assert => {
                let cond = self.pop_bool();
                if !cond { self.panic("assertion failed") }
            },
            RevealTokens => {
                for token in &self.tokens {
                    self.serialize_token(token);
                }
            }
        }
        return self.index + 1;
    }
}
