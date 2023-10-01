mod scanner;
mod token;

use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::vec::Vec;

use crate::scanner::Scanner;

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
    println!("run script: {:?}", tokens);
}
