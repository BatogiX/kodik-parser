use kodik_rs::run;
use std::{
    env,
    io::{self, IsTerminal, Read},
    process::ExitCode,
};

#[tokio::main]
async fn main() -> ExitCode {
    let mut stdin = String::new();

    if !io::stdin().is_terminal()
        && let Err(err) = io::stdin().read_to_string(&mut stdin)
    {
        eprintln!("failed to read stdin: {err}");
        return ExitCode::FAILURE;
    }

    let args: Vec<String> = env::args().chain(stdin.split_whitespace().map(str::to_owned)).collect();

    run(args).await
}
