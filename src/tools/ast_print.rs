// use std::vec;
//
// use crate::{
//     parser::{expr::*, stmt::*},
//     errors::LoxError, lexer::token::Literal
// };
//
// pub struct AstPrinter;
//
// #[allow(dead_code, unused)]
// impl AstPrinter {
//     pub fn new() -> Self {
//         Self
//     }
//
//     pub fn print(&self, expr: &Expr) {
//         if let Ok(data) = self.print_string(expr){
//             println!("{{\n{}}}", data);
//         }
//     }
//
//     fn print_string(&self, expr: &Expr) -> Result<String,LoxError> {
//         expr.accept(self, 0 as u16)
//     }
//
//     pub fn format(&self, name: &str, expr: Vec<&Expr>, prev_depth: u16) -> Result<String, LoxError> {
//         let tab = "  ";
//         let mut depth: u16 = 0;
//         let mut base = String::new();
//         if expr.len() > 0 {
//             base.push_str(&format!("{tab}{name} : {{\n"));
//             depth += 2 + prev_depth;
//             let branches = ["Left", "Right"];
//             let len = expr.len();
//             for (index, e) in expr.iter().enumerate() {
//                 if len == 2 {
//                     base.push_str(
//                         &format!("{}{}: {}\n",
//                              tab.repeat(depth as usize),
//                              branches[index],
//                              e.accept(self, depth)?.trim_end()
//                         )
//                     );
//                 } else {
//                     base.push_str(
//                         &format!("{}{}\n",
//                              tab.repeat(depth as usize),
//                              e.accept(self, depth)?.trim_end()
//                         )
//                     );
//                 }
//             }
//
//             base.push_str(&format!("{}}},\n", tab.repeat((depth - 1) as usize)));
//         } else {
//             if prev_depth == 0 {
//                 base.push_str(&format!("{tab}{name}\n"));
//             } else {
//                 base.push_str(&format!("{name}\n"));
//             }
//         }
//         Ok(base)
//     }
// }
//
// impl VisitorExpr<String> for AstPrinter {
//     fn visit_unary_expr(&self, expr: &UnaryExpr, depth: u16) -> Result<String, LoxError> {
//         self.format(&expr.operator.lexeme, vec![&expr.right], depth)
//     }
//
//     fn visit_binary_expr(&self, expr: &BinaryExpr, depth: u16) -> Result<String, LoxError> {
//         self.format(&expr.operator.lexeme, vec![&expr.left, &expr.right], depth)
//     }
//
//     fn visit_literal_expr(&self, expr: &LiteralExpr, depth: u16) -> Result<String, LoxError> {
//         self.format(expr.value.to_string().as_str(), vec![], depth)
//     }
//
//     fn visit_grouping_expr(&self, expr: &GroupingExpr, depth: u16) -> Result<String, LoxError> {
//         self.format("Group", vec![&expr.expression], depth)
//     }
//
//     fn visit_ternary_expr(&self, expr: &TernaryExpr, depth: u16) -> Result<String, LoxError> {
//         self.format("Ternary", vec![&expr.left, &expr.middle, &expr.right], depth)
//     }
//
//     fn visit_variable_expr(&self, expr: &VariableExpr, depth: u16) -> Result<String, LoxError> {
//         self.format(&expr.name.lexeme, vec![], depth)
//     }
//
//     fn visit_assign_expr(&self, expr: &AssignExpr, depth: u16) -> Result<String, LoxError> {
//         self.format(
//             format!("Assign {}", expr.name).as_str(),
//             vec![&expr.value],
//             depth
//         )
//     }
// }
//
// impl VisitorStmt<String> for AstPrinter {
//     fn visit_expression_stmt(&self, stmt: &ExpressionStmt, depth: u16) -> Result<String, LoxError> {
//         self.format("ExprStmt", vec![&stmt.expr], depth)
//     }
//
//     fn visit_print_stmt(&self, stmt: &PrintStmt, depth: u16) -> Result<String, LoxError> {
//         self.format("PrintStmt", vec![&stmt.expr], depth)
//     }
//
//     fn visit_let_stmt(&self, stmt: &LetStmt, depth: u16) -> Result<String, LoxError> {
//         if let Some(init) = &stmt.initializer {
//             self.format(&stmt.name.lexeme, vec![&init], depth)
//         } else {
//             let expr = Box::new(Expr::Literal(LiteralExpr::new(Literal::None)));
//             self.format(&stmt.name.lexeme, vec![&expr], depth)
//         }
//     }
//
//     fn visit_block_stmt(&self, stmt: &BlockStmt, depth: u16) -> Result<String, LoxError> {
//         for s in stmt.statements {
//             let expr = s.accept(self, depth)?;
//         }
//     }
// }
