use std::{env, process};
use r_lox_ast::Lox;

fn main() {
    let args: Vec<_> = env::args().collect();

    let mut lox = Lox::new();
    if args.len() == 1 {
        lox.set_is_repl(true);
        lox.run_prompt();
    } else if args.len() > 1 {
        lox.run_file(&args[1]);
    } else {
        println!("Usage: cargo run [script_path] or cargo run (runs the repl)");
        process::exit(64);
    }
}
