use std::env;
use std::fs;
use std::time::Instant;

mod virtual_machine;
mod model;
mod stack;

use virtual_machine::VirtualMachine;

fn main() {
    let files: Vec<String> = env::args().skip(1).collect();
    let start_file = fs::read_to_string(&files[0]).expect("Could not read file");

    let mut vm = VirtualMachine::new();
    for filename in files {
        let name = filename.clone();
        vm.include(name, fs::read_to_string(&filename).expect("could not read file"))
    }
    let start = Instant::now();
    vm.interpret(start_file);
    println!("elapsed: {}ms", start.elapsed().as_millis());
}
