use kodik::run;
use std::{env, process::ExitCode};

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    run(args).await
}
