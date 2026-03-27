mod cache;

use kodik::run;
use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    run(&args)
}
