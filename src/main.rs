#![feature(iter_intersperse)]

#[macro_use]
mod util;

mod debruijn;
mod eval;
mod hir;
mod layout;
mod layout_of;
mod lir;
mod lower;
mod name;
mod parser;

use std::io::{BufRead, Write};

use crate::eval::eval_root_expr;
use crate::layout_of::layout_of;
use crate::lower::{lower_layout, lower_root_expr};
use crate::parser::Parser;

fn main() {
    print_prompt();

    for line in std::io::stdin().lock().lines().map(Result::unwrap) {
        let line = line.trim();

        match line {
            "q" | "quit" | ":q" | ":quit" => break,
            _ => handle_input(line),
        }

        print_prompt();
    }
}

fn handle_input(line: &str) {
    match line.split_once(" ") {
        Some((":hir", src)) => {
            println!("{}", parse(src))
        }
        Some((":lir", src)) => {
            println!("{}", lower_root_expr(parse(src)))
        }
        Some((":lyt" | ":layout", src)) => {
            println!("{}", layout_of(parse_ty(src)))
        }
        Some((":t" | ":hty" | ":hirty", src)) => {
            println!("{}", parse(src).ty())
        }
        Some((":lty" | ":lirty", src)) => {
            println!("{}", lower_root_expr(parse(src)).ty())
        }
        Some((":size", src)) => {
            println!("Size: {}", lower_layout(layout_of(parse_ty(src))).packed_size())
        }
        Some((cmd, _)) if cmd.trim_start().starts_with(':') => {
            eprintln!("error: unknown REPL command '{}'", cmd)
        }
        _ => {
            println!("{}", parse_and_eval(line))
        }
    }
}

fn parse_and_eval(src: &str) -> lir::Value {
    let hir_expr = parse(src);
    let lir_expr = lower_root_expr(hir_expr);
    eval_root_expr(lir_expr)
}

fn parse(src: &str) -> hir::Expr {
    Parser::parse(src.to_owned())
}

fn parse_ty(src: &str) -> hir::Ty {
    Parser::parse_ty_toplevel(src.to_owned())
}

fn print_prompt() {
    print!("> ");
    std::io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests;
