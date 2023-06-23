use std::vec;

use crate::{
    parser::expr::*,
    lexer::token::Literal
};

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print(&self, expr: &Expr) {
        let data = self.print_string(expr);
        println!("{{\n{}}}", data);
    }

    fn print_string(&self, expr: &Expr) -> String {
        expr.accept(self, 0 as u16)
    }

    pub fn format(&self, name: &str, expr: Vec<&Expr>, prev_depth: u16) -> String {
        let tab = "  ";
        let mut depth: u16 = 0;
        let mut base = String::new();
        if expr.len() > 0 {
            base.push_str(&format!("{tab}{name} : {{\n"));
            depth += 2 + prev_depth;
            let branches = ["Left", "Right"];
            let len = expr.len();
            for (index, e) in expr.iter().enumerate() {
                if len == 2 {
                    base.push_str(
                        &format!("{}{}: {}\n", 
                             tab.repeat(depth as usize),
                             branches[index],
                             e.accept(self, depth).trim_end()
                        )
                    );
                } else {
                    base.push_str(
                        &format!("{}{}\n", 
                             tab.repeat(depth as usize),
                             e.accept(self, depth).trim_end()
                        )
                    );
                }
            }

            base.push_str(&format!("{}}},\n", tab.repeat((depth - 1) as usize)));
        } else {
            if prev_depth == 0 {
                base.push_str(&format!("{tab}{name}\n"));
            } else {
                base.push_str(&format!("{name}\n"));
            }
        }
        base
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_unary_expr(&self, expr: &UnaryExpr, depth: u16) -> String {
        self.format(&expr.operator.lexeme, vec![&expr.right], depth)
    }

    fn visit_binary_expr(&self, expr: &BinaryExpr, depth: u16) -> String {
        self.format(&expr.operator.lexeme, vec![&expr.left, &expr.right], depth)
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr, depth: u16) -> String {
        self.format(expr.value.to_string().as_str(), vec![], depth)
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr, depth: u16) -> String {
        self.format("Group", vec![&expr.expression], depth) 
    }
}
