mod ast_gen;
use ast_gen::*;
use std::io;

fn main() -> io::Result<()> {
    let expr_types = vec![
        "Binary ; left: Rc<Expr>, operator: Token, right: Rc<Expr>".to_string(),
        "Logical ; left: Rc<Expr>, operator: Token, right: Rc<Expr>".to_string(),
        "Grouping ; expression: Rc<Expr>".to_string(),
        "Literal ; value: Literal".to_string(),
        "Unary ; operator: Token, right: Rc<Expr>".to_string(),
        "Ternary ; left: Rc<Expr>, operator: Token, middle: Rc<Expr>, colon: Token, right: Rc<Expr>".to_string(),
        "Variable ; name: Token".to_string(),
        "Assign ; name: Token, value: Rc<Expr>".to_string(),
        "CompoundAssign ; name: Token, operator: Token, value: Rc<Expr>".to_string(),
        "Call ; callee: Rc<Expr>, paren: Token, args: Vec<Rc<Expr>>".to_string(),
        "Lambda ; params: Rc<Vec<Token>>, body: Rc<Vec<Rc<Stmt>>>".to_string(),
        "Array ; arr: Vec<Rc<Expr>>".to_string(),
        "Index ; identifier: Rc<Expr>, bracket: Token, index: Rc<Expr>".to_string(),
        "Get ; object: Rc<Expr>, name: Token".to_string(),
        "Set ; object: Rc<Expr>, name: Token, value: Rc<Expr>, operator: Token".to_string(),
        "Update ; var: Rc<Expr>, operator: Token, prefix: bool".to_string(),
        "This ; keyword: Token".to_string(),
    ];

    let expr_mods = vec![
        "crate::lexer::{token::Token, literal::Literal}".to_string(),
        "crate::error::LoxResult".to_string(),
        "super::stmt::*".to_string(),
        "std::rc::Rc".to_string(),
        "std::hash::{Hash, Hasher}".to_string()
    ];

    let stmt_type = vec![
        "Expression ; expr: Rc<Expr>".to_string(),
        "Print ; expr: Rc<Expr>".to_string(),
        "Let ; name: Token, initializer: Option<Rc<Expr>>".to_string(),
        "Block ; statements: Vec<Rc<Stmt>>".to_string(),
        "If ; condition: Rc<Expr>, then_branch: Rc<Stmt>, else_branch: Option<Rc<Stmt>>".to_string(),
        "While ; condition: Rc<Expr>, body: Rc<Stmt>".to_string(),
        "For ; var: Option<Rc<Stmt>>, condition: Option<Rc<Expr>>, update_expr: Option<Rc<Expr>>, body: Rc<Stmt>".to_string(),
        "Break ; token: Token".to_string(),
        "Continue ; token: Token".to_string(),
        "Function ; name: Token, params: Rc<Vec<Token>>, body: Rc<Vec<Rc<Stmt>>>, is_static: bool, is_pub: bool".to_string(),
        "Return ; keyword: Token, value: Rc<Expr>".to_string(),
        "Class ; name: Token, fields: Vec<Rc<Stmt>>, methods: Vec<Rc<Stmt>>".to_string(),
        "Field ; name: Token, is_pub: bool, is_static: bool".to_string(),
    ];

    let stmt_mods = vec![
        "crate::lexer::token::Token".to_string(),
        "crate::error::LoxResult".to_string(),
        "std::rc::Rc".to_string(),
        "super::expr::*".to_string(),
        "std::hash::{Hash, Hasher}".to_string()
    ];

    let out_dir = "./src/parser/";
    GenAst::gen("Expr", out_dir, expr_types, expr_mods)?;
    GenAst::gen("Stmt", out_dir, stmt_type, stmt_mods)
}
