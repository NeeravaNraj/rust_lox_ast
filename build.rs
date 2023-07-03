mod ast_gen;
use std::io;
use ast_gen::*;

fn main() -> io::Result<()> {
    let expr_types = vec![
        "Binary ; left: Box<Expr>, operator: Token, right: Box<Expr>".to_string(),
        "Logical ; left: Box<Expr>, operator: Token, right: Box<Expr>".to_string(),
        "Grouping ; expression: Box<Expr>".to_string(),
        "Literal ; value: Literal".to_string(),
        "Unary ; operator: Token, right: Box<Expr>".to_string(),
        "Ternary ; left: Box<Expr>, operator: Token, middle: Box<Expr>, colon: Token, right: Box<Expr>".to_string(),
        "Variable ; name: Token".to_string(),
        "Assign ; name: Token, value: Box<Expr>".to_string(),
        "CompoundAssign ; name: Token, operator: Token, value: Box<Expr>".to_string(),
        "Call ; callee: Box<Expr>, paren: Token, args: Vec<Box<Expr>>".to_string(),
    ];

    let expr_mods = vec![
        "crate::lexer::token::{Token, Literal}".to_string(),
        "crate::errors::LoxError".to_string(),
    ];

    let stmt_type = vec![
        "Expression ; expr: Box<Expr>".to_string(),
        "Print ; expr: Box<Expr>".to_string(),
        "Let ; name: Token, initializer: Option<Box<Expr>>".to_string(),
        "Block ; statements: Vec<Box<Stmt>>".to_string(),
        "If ; condition: Box<Expr>, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>>".to_string(),
        "While ; condition: Box<Expr>, body: Box<Stmt>".to_string(),
        "Break ; token: Token".to_string(),
        "Continue ; token: Token".to_string(),
    ];

    let stmt_mods = vec![
        "crate::lexer::token::{Token}".to_string(),
        "crate::errors::LoxError".to_string(),
        "super::expr::*".to_string(),
    ];
    
    let out_dir = "./src/parser/";
    GenAst::gen("Expr", out_dir, expr_types, expr_mods)?;
    GenAst::gen("Stmt", out_dir, stmt_type, stmt_mods)
}
