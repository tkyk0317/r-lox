mod ast;
mod embedded;
mod environment;
mod eval;
mod scanner;
mod token;

use crate::embedded::func;
use crate::environment::Environment;
use crate::scanner::Scanner;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::vec::Vec;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0..=1 => repl(),
        2 => run(&args[1]),
        _ => println!("Usage: r-lox [script filename]"),
    };
}

// スクリプトファイル実行
fn run(file: &String) {
    let mut f = File::open(file).expect("can not found file: {:file?}");
    let mut content = String::new();
    f.read_to_string(&mut content)
        .expect("can not read file {:file?}");
    run_script(&content);
}

// REPL実行
//
// Ctrl+cで抜ける
fn repl() {
    let mut buffer = String::new();
    loop {
        io::stdin()
            .read_line(&mut buffer)
            .expect("can not read stdin");
        run_script(&buffer);
    }
}

// スクリプト実行
fn run_script(scripts: &String) {
    let scanner = Scanner::new(scripts);
    let tokens = scanner.scan();
    let ast = ast::Parser::new(&tokens).program();
    let env = Environment::new();
    let mut env = func::register_func(&env);

    ast.into_iter().for_each(|a| {
        let eval_ret = eval::eval(&a, &mut env);
        match eval_ret {
            Ok(result) => eval::print(result),
            Err(err) => println!("{:?}", err),
        };
    });
}
