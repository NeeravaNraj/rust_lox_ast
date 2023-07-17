use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    error::{loxerrorhandler::LoxErrorHandler, *},
    lexer::token::Token,
    parser::expr::*,
    parser::stmt::*,
};

use super::interpreter::Interpreter;

pub struct Resolver<'a> {
    interpreter: &'a Interpreter,
    scopes: RefCell<Vec<RefCell<HashMap<String, bool>>>>,
    error_handler: LoxErrorHandler,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a Interpreter) -> Self {
        Self {
            interpreter,
            scopes: RefCell::new(Vec::new()),
            error_handler: LoxErrorHandler::new(),
        }
    }

    fn resolve_statement(&self, statement: Rc<Stmt>) -> Result<(), LoxResult> {
        statement.accept(statement.clone(), self, 0_u16)
    }

    fn resolve_function(&self, function: &FunctionStmt) -> Result<(), LoxResult> {
        self.begin_scope();
        for param in function.params.iter() {
            self.declare(param);
            self.define(param);
        }
        self.resolve(&function.body);
        self.end_scope();
        Ok(())
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

    pub fn resolve(&self, statements: &Vec<Rc<Stmt>>) -> Result<(), LoxResult> {
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

    fn resolve_local(&self, expr: Rc<Expr>, name: &Token) {
        for (i, scope) in self.scopes.borrow().iter().rev().enumerate() {
            if scope.borrow().contains_key(&name.lexeme) {
                self.interpreter
                    .resolve(expr.clone(), self.scopes.borrow().len() - 1 - i);
                return;
            }
        }
    }
}

impl<'a> VisitorExpr<()> for Resolver<'a> {
    fn visit_index_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &IndexExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.identifier.clone())?;
        self.resolve_expr(expr.index.clone())?;
        Ok(())
    }

    fn visit_array_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &ArrayExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        for el in expr.arr.iter() {
            self.resolve_expr(el.clone())?;
        }
        Ok(())
    }

    fn visit_call_expr(
        &self, 
        wrapper: Rc<Expr>, 
        expr: &CallExpr,
        _: u16
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.callee.clone())?;

        for arg in expr.args.iter() {
            self.resolve_expr(arg.clone())?;
        }

        Ok(())
    }

    fn visit_unary_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &UnaryExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_binary_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &BinaryExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_assign_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &AssignExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.value.clone())?;
        self.resolve_local(wrapper.clone(), &expr.name);
        Ok(())
    }

    fn visit_lambda_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &LambdaExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve(&*expr.body)?;
        Ok(())
    }

    fn visit_logical_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &LogicalExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_literal_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &LiteralExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_ternary_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &TernaryExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone());
        self.resolve_expr(expr.middle.clone());
        self.resolve_expr(expr.right.clone());
        Ok(())
    }

    fn visit_grouping_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &GroupingExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.expression.clone())?;
        Ok(())
    }

    fn visit_variable_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &VariableExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        if !self.scopes.borrow().is_empty()
            && self
                .scopes
                .borrow()
                .last()
                .unwrap()
                .borrow()
                .get(&expr.name.lexeme)
                == Some(&false)
        {
            return Err(self.error_handler.error(
                &expr.name,
                LoxErrorsTypes::ReferenceError(
                    "Can't read local variable in its own initializer".to_string(),
                ),
            ));
        }

        self.resolve_local(wrapper.clone(), &expr.name);
        Ok(())
    }

    fn visit_compoundassign_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &CompoundAssignExpr,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(expr.value.clone())?;
        self.resolve_local(wrapper.clone(), &expr.name);
        Ok(())
    }
}

impl<'a> VisitorStmt<()> for Resolver<'a> {
    fn visit_if_stmt(&self, wrapper: Rc<Stmt>, stmt: &IfStmt, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_statement(stmt.then_branch.clone())?;
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_statement(else_branch.clone())?;
        }

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
        if let Some(var) = &stmt.var {
            self.resolve_statement(var.clone())?;
        }
        if let Some(condition) = &stmt.condition {
            self.resolve_expr(condition.clone())?;
        }
        if let Some(expr) = &stmt.update_expr {
            self.resolve_expr(expr.clone())?;
        }

        self.resolve_statement(stmt.body.clone())?;
        Ok(())
    }

    fn visit_print_stmt(
        &self,
        wrapper: Rc<Stmt>,
        stmt: &PrintStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expr.clone())?;
        Ok(())
    }

    fn visit_block_stmt(
        &self,
        wrapper: Rc<Stmt>,
        stmt: &BlockStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_while_stmt(
        &self,
        wrapper: Rc<Stmt>,
        stmt: &WhileStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_statement(stmt.body.clone())?;
        Ok(())
    }

    fn visit_break_stmt(
        &self,
        wrapper: Rc<Stmt>,
        stmt: &BreakStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_return_stmt(
        &self,
        wrapper: Rc<Stmt>,
        stmt: &ReturnStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.value.clone())?;
        Ok(())
    }

    fn visit_continue_stmt(
        &self,
        wrapper: Rc<Stmt>,
        stmt: &ContinueStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_function_stmt(
        &self,
        wrapper: Rc<Stmt>,
        stmt: &FunctionStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        self.define(&stmt.name);

        self.resolve_function(stmt);
        Ok(())
    }

    fn visit_expression_stmt(
        &self,
        wrapper: Rc<Stmt>,
        stmt: &ExpressionStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expr.clone())?;
        Ok(())
    }
}
