use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use crate::{
    error::{loxerrorhandler::LoxErrorHandler, *},
    lexer::token::Token,
    parser::expr::*,
    parser::stmt::*,
};

use super::interpreter::Interpreter;

pub struct Resolver {
    interpreter: Interpreter,
    scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
    error_handler: LoxErrorHandler,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: RefCell::new(Vec::new()),
            error_handler: LoxErrorHandler::new(),
        }
    }

    fn resolve_statement(&self, statement: Rc<Stmt>) -> Result<(), LoxResult> {
        statement.accept(statement.clone(), self, 0_u16)
    }

    fn resolve_expr(&self, expr: Rc<Expr>) -> Result<(), LoxResult> {
        expr.accept(expr.clone(), self, 0_u16)
    }

    fn begin_scope(&self) {
        self.scopes.borrow_mut().push(RefCell::new(HashMap::new()));
    }

    fn end_scope(&self) {
        self.scopes.borrow_mut().pop();
    }

    fn resolve(&self, statements: &Vec<Rc<Stmt>>) -> Result<(), LoxResult> {
        for statement in statements {
            self.resolve_statement(statement.clone())?;
        }
        Ok(())
    }

    fn declare(&self, name: &Token) {
        if self.scopes.borrow().is_empty() {
            return;
        }

        self.scopes
            .borrow()
            .last()
            .unwrap()
            .borrow_mut()
            .insert(name.lexeme.to_string(), false);
    }

    fn define(&self, name: &Token) {
        if self.scopes.borrow().is_empty() {
            return;
        }

        self.scopes
            .borrow_mut()
            .last()
            .unwrap()
            .borrow_mut()
            .insert(name.lexeme.to_string(), true);
    }

    fn resolve_local(&self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.borrow().iter().rev().enumerate() {
            if scope.borrow().contains_key(&name.lexeme) {
                // self.interpreter.resolve(expr, self.scopes.borrow().len() - 1 - i);
                return
            } 
        }
    }
}

impl VisitorExpr<()> for Resolver {
    fn visit_index_expr(&self, wrapper: Rc<Expr>, expr: &IndexExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_array_expr(&self, wrapper: Rc<Expr>, expr: &ArrayExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_call_expr(&self, wrapper: Rc<Expr>, expr: &CallExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_unary_expr(&self, wrapper: Rc<Expr>, expr: &UnaryExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_binary_expr(&self, wrapper: Rc<Expr>, expr: &BinaryExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_assign_expr(&self, wrapper: Rc<Expr>, expr: &AssignExpr, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(expr.value.clone())?;
        // self.resolve_local(&Expr::Assign(Box::new(expr)), &expr.name);
        Ok(())
    }

    fn visit_lambda_expr(&self, wrapper: Rc<Expr>, expr: &LambdaExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_logical_expr(&self, wrapper: Rc<Expr>, expr: &LogicalExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_literal_expr(&self, wrapper: Rc<Expr>, expr: &LiteralExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_ternary_expr(&self, wrapper: Rc<Expr>, expr: &TernaryExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_grouping_expr(&self, wrapper: Rc<Expr>, expr: &GroupingExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_variable_expr(&self, wrapper: Rc<Expr>, expr: &VariableExpr, _: u16) -> Result<(), LoxResult> {
        if !self.scopes.borrow().is_empty() {
            if let Some(map) = self.scopes.borrow().last() {
                if let Some(var) = map.borrow().get(&expr.name.lexeme) {
                    if var == &false {
                        return Err(self.error_handler.error(
                            &expr.name,
                            LoxErrorsTypes::ReferenceError(
                                "Can't read local variable in its own initializer".to_string(),
                            ),
                        ));
                    }
                }
            }
        }

        self.resolve_local(&Expr::Variable(VariableExpr::new(expr.name.dup())), &expr.name);
        Ok(())
    }

    fn visit_compoundassign_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &CompoundAssignExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        Ok(())
    }
}

impl VisitorStmt<()> for Resolver {
    fn visit_if_stmt(&self, wrapper: Rc<Stmt>, stmt: &IfStmt, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_let_stmt(&self, wrapper: Rc<Stmt>, stmt: &LetStmt, _: u16) -> Result<(), LoxResult> {
        self.declare(&stmt.name);

        if let Some(init) = &stmt.initializer {
            self.resolve_expr(init.clone());
        }
        self.define(&stmt.name);
        Ok(())
    }

    fn visit_for_stmt(&self, wrapper: Rc<Stmt>, stmt: &ForStmt, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_print_stmt(&self, wrapper: Rc<Stmt>, stmt: &PrintStmt, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_block_stmt(&self, wrapper: Rc<Stmt>, stmt: &BlockStmt, _: u16) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_while_stmt(&self, wrapper: Rc<Stmt>, stmt: &WhileStmt, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_break_stmt(&self, wrapper: Rc<Stmt>, stmt: &BreakStmt, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_return_stmt(&self, wrapper: Rc<Stmt>, stmt: &ReturnStmt, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_continue_stmt(&self, wrapper: Rc<Stmt>, stmt: &ContinueStmt, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_function_stmt(&self, wrapper: Rc<Stmt>, stmt: &FunctionStmt, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_expression_stmt(&self, wrapper: Rc<Stmt>, stmt: &ExpressionStmt, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }
}
