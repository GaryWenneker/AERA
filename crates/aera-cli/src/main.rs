#![forbid(unsafe_code)]

use aera_core::Target;
use std::process::ExitCode;

const USAGE: &str = "Usage: aera [info|doctor|help]";

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).take(2).collect();
    let target = Target::current();
    match args.as_slice() {
        [] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        [arg] if arg == "help" || arg == "--help" || arg == "-h" => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        [arg] if arg == "info" => {
            println!("AERA {} (hosted prototype)", env!("CARGO_PKG_VERSION"));
            println!("os={} arch={}", target.os, target.arch);
            ExitCode::SUCCESS
        }
        [arg] if arg == "doctor" => {
            println!("os={} arch={}", target.os, target.arch);
            println!("hosted_target_eligible={}", target.hosted_eligible());
            println!("Hardware readiness: not assessed");
            ExitCode::from(u8::from(!target.hosted_eligible()))
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
