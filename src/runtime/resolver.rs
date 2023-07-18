use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    error::{loxerrorhandler::LoxErrorHandler, loxwarninghandler::LoxWarningHandler, *},
    lexer::token::Token,
    parser::expr::*,
    parser::stmt::*,
};

use super::interpreter::Interpreter;

#[derive(PartialEq)]
enum FnType {
    None,
    Function,
}

#[derive(PartialEq)]
enum LoopType {
    None,
    Loop,
}

#[derive(PartialEq)]
struct VariableType {
    token: Token,
    define: bool,
    used: bool,
}

#[derive(PartialEq)]
enum Returned {
    None,
    Return(i32),
}
impl VariableType {
    fn new(tok: Token, d: bool, u: bool) -> Self {
        Self {
            token: tok,
            define: d,
            used: u,
        }
    }
}

pub struct Resolver<'a> {
    pub had_error: RefCell<bool>,
    interpreter: &'a Interpreter,
    scopes: RefCell<Vec<RefCell<HashMap<String, VariableType>>>>,
    error_handler: LoxErrorHandler,
    warning_handler: LoxWarningHandler,
    current_fn: RefCell<FnType>,
    current_loop: RefCell<LoopType>,
    returned: RefCell<Returned>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a Interpreter) -> Self {
        Self {
            interpreter,
            scopes: RefCell::new(Vec::new()),
            error_handler: LoxErrorHandler::new(),
            current_fn: RefCell::new(FnType::None),
            returned: RefCell::new(Returned::None),
            current_loop: RefCell::new(LoopType::None),
            had_error: RefCell::new(false),
            warning_handler: LoxWarningHandler::new(),
        }
    }

    fn resolve_statement(&self, statement: Rc<Stmt>) -> Result<(), LoxResult> {
        statement.accept(statement.clone(), self, 0_u16)
    }

    fn resolve_function(&self, function: &FunctionStmt, fn_type: FnType) -> Result<(), LoxResult> {
        let enclosing_fn = self.current_fn.replace(fn_type);
        self.begin_scope();
        for param in function.params.iter() {
            self.declare(param);
            self.define(param);
        }
        // self.resolve(&function.body)?;
        for s in function.body.iter() {
            if self.current_fn.borrow().eq(&FnType::Function) {
                unsafe {
                    match *self.returned.as_ptr() {
                        Returned::Return(line) => {
                            self.warning_handler.simple_warning(
                                line + 1,
                                LoxWarningTypes::DeadCode(format!(
                                    "Found unreachable code after line '{}' in function '{}'",
                                    line + 1,
                                    function.name.lexeme
                                )),
                            );
                            break;
                        }
                        _ => {}
                    }
                }
            }
            self.resolve_statement(s.clone())?;
        }
        self.end_scope();
        self.current_fn.replace(enclosing_fn);
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
        for scope in self.scopes.borrow().iter() {
            for var in scope.borrow().values() {
                if !var.used {
                    self.warning_handler.warn(
                        &var.token,
                        LoxWarningTypes::UnusedVariable("Unused variable".to_string()),
                    );
                }
            }
        }
        Ok(())
    }

    fn declare(&self, name: &Token) {
        if self.scopes.borrow().is_empty() {
            return;
        }

        if self
            .scopes
            .borrow()
            .last()
            .unwrap()
            .borrow()
            .contains_key(&name.lexeme)
        {
            LoxErrorHandler::report_asc(
                name,
                &LoxErrorsTypes::Parse(
                    "Already a variable with this name in this scope".to_string(),
                ),
            );
            return;
        }
        self.scopes.borrow().last().unwrap().borrow_mut().insert(
            name.lexeme.to_string(),
            VariableType::new(name.dup(), false, false),
        );
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
            .insert(
                name.lexeme.to_string(),
                VariableType::new(name.dup(), true, false),
            );
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
    fn visit_index_expr(&self, _: Rc<Expr>, expr: &IndexExpr, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(expr.identifier.clone())?;
        self.resolve_expr(expr.index.clone())?;
        Ok(())
    }

    fn visit_array_expr(&self, _: Rc<Expr>, expr: &ArrayExpr, _: u16) -> Result<(), LoxResult> {
        for el in expr.arr.iter() {
            self.resolve_expr(el.clone())?;
        }
        Ok(())
    }

    fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(expr.callee.clone())?;

        for arg in expr.args.iter() {
            self.resolve_expr(arg.clone())?;
        }

        Ok(())
    }

    fn visit_unary_expr(&self, _: Rc<Expr>, expr: &UnaryExpr, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_binary_expr(&self, _: Rc<Expr>, expr: &BinaryExpr, _: u16) -> Result<(), LoxResult> {
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

    fn visit_lambda_expr(&self, _: Rc<Expr>, expr: &LambdaExpr, _: u16) -> Result<(), LoxResult> {
        self.resolve(&*expr.body)?;
        Ok(())
    }

    fn visit_logical_expr(&self, _: Rc<Expr>, expr: &LogicalExpr, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_literal_expr(&self, _: Rc<Expr>, _: &LiteralExpr, _: u16) -> Result<(), LoxResult> {
        Ok(())
    }

    fn visit_ternary_expr(&self, _: Rc<Expr>, expr: &TernaryExpr, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(expr.left.clone())?;
        self.resolve_expr(expr.middle.clone())?;
        self.resolve_expr(expr.right.clone())?;
        Ok(())
    }

    fn visit_grouping_expr(
        &self,
        _: Rc<Expr>,
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
        if !self.scopes.borrow().is_empty() {
            if let Some(v_type) = self
                .scopes
                .borrow()
                .last()
                .unwrap()
                .borrow()
                .get(&expr.name.lexeme)
            {
                if !v_type.define {
                    return Err(self.error_handler.error(
                        &expr.name,
                        LoxErrorsTypes::ReferenceError(
                            "Can't read local variable in its own initializer".to_string(),
                        ),
                    ));
                }
            }
            if let Some(map) = self.scopes.borrow().last() {
                if let Some(val) = map.borrow_mut().get_mut(&expr.name.lexeme) {
                    val.used = true;
                }
            }
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
    fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.condition.clone())?;
        self.resolve_statement(stmt.then_branch.clone())?;
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_statement(else_branch.clone())?;
        }

        Ok(())
    }

    fn visit_let_stmt(&self, _: Rc<Stmt>, stmt: &LetStmt, _: u16) -> Result<(), LoxResult> {
        self.declare(&stmt.name);

        if let Some(init) = &stmt.initializer {
            self.resolve_expr(init.clone())?;
        }
        self.define(&stmt.name);
        Ok(())
    }

    fn visit_for_stmt(&self, _: Rc<Stmt>, stmt: &ForStmt, _: u16) -> Result<(), LoxResult> {
        if let Some(var) = &stmt.var {
            self.resolve_statement(var.clone())?;
        }
        if let Some(condition) = &stmt.condition {
            self.resolve_expr(condition.clone())?;
        }
        if let Some(expr) = &stmt.update_expr {
            self.resolve_expr(expr.clone())?;
        }

        let old = self.current_loop.replace(LoopType::Loop);
        self.resolve_statement(stmt.body.clone())?;
        self.current_loop.replace(old);
        Ok(())
    }

    fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expr.clone())?;
        Ok(())
    }

    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt, _: u16) -> Result<(), LoxResult> {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        if self.current_fn.borrow().eq(&FnType::Function) {
            self.returned.replace(Returned::None);
        }
        Ok(())
    }

    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt, _: u16) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.condition.clone())?;
        let old = self.current_loop.replace(LoopType::Loop);
        self.resolve_statement(stmt.body.clone())?;
        self.current_loop.replace(old);
        Ok(())
    }

    fn visit_break_stmt(&self, _: Rc<Stmt>, stmt: &BreakStmt, _: u16) -> Result<(), LoxResult> {
        if self.current_loop.borrow().eq(&LoopType::None) {
            self.had_error.replace(true);
            self.error_handler.error(
                &stmt.token,
                LoxErrorsTypes::Parse("Found 'break' outside loop body".to_string()),
            );
        }
        Ok(())
    }

    fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt, _: u16) -> Result<(), LoxResult> {
        if self.current_fn.borrow().eq(&FnType::None) {
            self.had_error.replace(true);
            self.error_handler.error(
                &stmt.keyword,
                LoxErrorsTypes::Parse("Unexpected return outside function".to_string()),
            );
        }
        self.resolve_expr(stmt.value.clone())?;
        self.returned.replace(Returned::Return(stmt.keyword.line));
        Ok(())
    }

    fn visit_continue_stmt(
        &self,
        _: Rc<Stmt>,
        stmt: &ContinueStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        if self.current_loop.borrow().eq(&LoopType::None) {
            self.had_error.replace(true);
            self.error_handler.error(
                &stmt.token,
                LoxErrorsTypes::Parse("Found 'continue' outside loop body".to_string()),
            );
        }
        Ok(())
    }

    fn visit_function_stmt(
        &self,
        _: Rc<Stmt>,
        stmt: &FunctionStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.declare(&stmt.name);
        self.define(&stmt.name);

        self.resolve_function(stmt, FnType::Function)?;
        Ok(())
    }

    fn visit_expression_stmt(
        &self,
        _: Rc<Stmt>,
        stmt: &ExpressionStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.resolve_expr(stmt.expr.clone())?;
        Ok(())
    }
}
