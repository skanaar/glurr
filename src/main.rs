use std::env;
use std::fs;
use std::time::Instant;

mod virtual_machine;
mod model;
mod stack;
mod debugger;

use virtual_machine::VirtualMachine;
use debugger::Debugger;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let flags: Vec<&String> =
        args.iter().filter(|e| e.chars().next() == Some('-')).collect();
    let files: Vec<&String> = args.iter().skip(flags.len()).collect();
    let entry_name = files[0].clone();
    let entry_source = fs::read_to_string(&entry_name).expect("Can't read file");

    let debug = has(&flags, "--debug") || has(&flags, "-d");
    let report = has(&flags, "--report") || has(&flags, "-r");
    let time = has(&flags, "--time") || has(&flags, "-t");

    let mut vm = VirtualMachine::new();
    vm.flag_report = report;
    for filename in files {
        let name = filename.clone();
        let source = fs::read_to_string(&filename).expect("Can't read file");
        vm.register_file(name, source);
    }
    vm.include(entry_name, entry_source);
    if debug {
        let mut app = Debugger::new(vm);
        app.run().expect("debugger error");
    } else {
        let start = Instant::now();
        vm.interpret();
        if time { println!("\nelapsed: {}ms", start.elapsed().as_millis()) }
    }
}

fn has(args: &Vec<&String>, needle: &str) -> bool {
    args.iter().any(|e| e.to_string() == needle.to_string())
}
