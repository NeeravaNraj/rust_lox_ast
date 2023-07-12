mod ast_gen;
use ast_gen::*;
use std::io;

fn main() -> io::Result<()> {
    let expr_types = vec![
        "Binary ; left: Expr, operator: Token, right: Expr".to_string(),
        "Logical ; left: Expr, operator: Token, right: Expr".to_string(),
        "Grouping ; expression: Expr".to_string(),
        "Literal ; value: Literal".to_string(),
        "Unary ; operator: Token, right: Expr".to_string(),
        "Ternary ; left: Expr, operator: Token, middle: Expr, colon: Token, right: Expr".to_string(),
        "Variable ; name: Token".to_string(),
        "Assign ; name: Token, value: Expr".to_string(),
        "CompoundAssign ; name: Token, operator: Token, value: Expr".to_string(),
        "Call ; callee: Expr, paren: Token, args: Vec<Expr>".to_string(),
        "Lambda ; params: Rc<Vec<Token>>, body: Rc<Vec<Stmt>>".to_string(),
        "Array ; arr: Vec<Expr>".to_string(),
        "Index ; identifier: Box<Expr>, bracket: Token, index: Box<Expr>".to_string(),
    ];

    let expr_mods = vec![
        "crate::lexer::{token::Token, literal::Literal}".to_string(),
        "crate::error::LoxResult".to_string(),
        "super::stmt::*".to_string(),
        "std::rc::Rc".to_string()
    ];

    let stmt_type = vec![
        "Expression ; expr: Expr".to_string(),
        "Print ; expr: Expr".to_string(),
        "Let ; name: Token, initializer: Option<Expr>".to_string(),
        "Block ; statements: Vec<Stmt>".to_string(),
        "If ; condition: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>>".to_string(),
        "While ; condition: Expr, body: Box<Stmt>".to_string(),
        "For ; var: Option<Box<Stmt>>, condition: Option<Expr>, update_expr: Option<Expr>, body: Box<Stmt>".to_string(),
        "Break ; token: Token".to_string(),
        "Continue ; token: Token".to_string(),
        "Function ; name: Token, params: Rc<Vec<Token>>, body: Rc<Vec<Stmt>>".to_string(),
        "Return ; keyword: Token, value: Expr".to_string(),
    ];

    let stmt_mods = vec![
        "crate::lexer::token::Token".to_string(),
        "crate::error::LoxResult".to_string(),
        "std::rc::Rc".to_string(),
        "super::expr::*".to_string(),
    ];

    let out_dir = "./src/parser/";
    GenAst::gen("Expr", out_dir, expr_types, expr_mods)?;
    GenAst::gen("Stmt", out_dir, stmt_type, stmt_mods)
}
