use std::env;
use std::fs;

mod virtual_machine;
mod model;
mod stack;

use virtual_machine::VirtualMachine;


fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    let mut vm = VirtualMachine::new();
    vm.interpret(contents);
}
