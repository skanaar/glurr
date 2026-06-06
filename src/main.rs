use std::env;
use std::fs;
use std::time::Instant;

mod virtual_machine;
mod model;
mod stack;

use virtual_machine::VirtualMachine;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let flags: Vec<&String> =
        args.iter().filter(|e| e.chars().next() == Some('-')).collect();
    let files: Vec<&String> = args.iter().skip(flags.len()).collect();
    let start_file = fs::read_to_string(&files[0]).expect("Can't read file");

    let debug = flags.iter().any(|e| e.to_string() == "--debug".to_string());
    let time = flags.iter().any(|e| e.to_string() == "--time".to_string());

    let mut vm = VirtualMachine::new();
    vm.flag_debug = debug;
    for filename in files {
        let name = filename.clone();
        vm.include(name, fs::read_to_string(&filename).expect("could not read file"))
    }
    let start = Instant::now();
    vm.interpret(start_file);
    if time { println!("\nelapsed: {}ms", start.elapsed().as_millis()) }
}
