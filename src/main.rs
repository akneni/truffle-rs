#![allow(unused)]
mod lexer;
mod parser;
mod utils;

use std::{collections::HashSet, default, fs, io::Stdout};
use parser::AstNode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use regex::Regex;
use lexer::Lexer;
use utils::{FnLst, VarLst};

fn main() {
    let code = fs::read_to_string("truffle/main.tr")
        .unwrap()
        .split("\n")
        .filter(|&line| !line.trim().starts_with("//"))
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
        .join("\n")
        .replace("  ", " ")
        .replace("\n\n", "\n");



    let mut lexer = Lexer::new(&code);

    while let Some(token) = lexer.next() {
        println!("{:?}", token);
    }

    let errors = lexer.validate_syntax();
    println!("\nLexer Errors: {:#?}\n\n\n\n\n", errors);

    let mut var_lst = VarLst::new();
    let mut fn_list = FnLst::new();

    let s = AstNode::generate_function(&lexer.tokens, &mut var_lst, &mut fn_list);
    println!("{:#?}", s);
}
