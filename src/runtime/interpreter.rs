use super::{
    callable::{Callable, LoxCallable},
    environment::Environment,
    loxfunction::LoxFunction,
};
use crate::{
    error::{loxerrorhandler::LoxErrorHandler, LoxError, LoxErrorsTypes, LoxResult},
    lexer::{literal::*, token::*, tokentype::TokenType},
    loxlib::loxnatives::Clock,
    parser::{expr::*, stmt::*},
};
use std::{
    cell::RefCell,
    ops::{Add, Div, Mul, Sub},
};
use std::{collections::HashMap, rc::Rc};

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    pub locals: RefCell<HashMap<Rc<Expr>, usize>>,
    pub environment: RefCell<Rc<RefCell<Environment>>>,
    pub error_handler: LoxErrorHandler,
    is_repl: bool,
    is_single_expr: RefCell<bool>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new()));

        if let Err(LoxResult::Error(e)) = globals.borrow_mut().define_native(
            &Token::new(TokenType::DefFn, "clock".to_string(), None, 0),
            Literal::Func(Callable {
                func: Rc::new(Clock {}),
            }),
        ) {
            LoxError::report(&e);
        }
        Self {
            globals: Rc::clone(&globals),
            locals: RefCell::new(HashMap::new()),
            error_handler: LoxErrorHandler::new(),
            environment: RefCell::new(Rc::clone(&globals)),
            is_repl: false,
            is_single_expr: RefCell::new(false),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Rc<Stmt>>) -> Result<(), LoxResult> {
        if stmts.len() == 1 {
            if let Some(stmt) = stmts.get(0) {
                match **stmt {
                    Stmt::Expression(_) => self.is_single_expr.replace(true),
                    _ => self.is_single_expr.replace(false),
                };
            }
        }
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }
    pub fn evaluate(&self, expr: Rc<Expr>) -> Result<Literal, LoxResult> {
        let val = expr.accept(expr.clone(), self, 0_u16)?;
        Ok(val)
    }

    pub fn execute(&self, stmt: Rc<Stmt>) -> Result<(), LoxResult> {
        stmt.accept(stmt.clone(), self, 0_u16)
    }

    pub fn resolve(&self, expr: Rc<Expr>, depth: usize) {
        self.locals.borrow_mut().insert(expr, depth);
    }

    pub fn is_truthy(&self, right: &Literal) -> bool {
        !matches!(right, Literal::None | Literal::Bool(false))
    }

    pub fn is_equal(&self, left: Literal, right: Literal) -> bool {
        left.equals(right)
    }

    fn look_up_variable(&self, name: &Token, expr: &Rc<Expr>) -> Result<Literal, LoxResult> {
        if let Some(d) = self.locals.borrow().get(expr) {
            return self.environment.borrow().borrow().get_at(*d, name);
        }

        self.globals.borrow().get(name)
    }

    fn check_num_unary(&self, operator: &Token, operand: &Literal) -> Result<(), LoxResult> {
        let err_type = if operator.lexeme == "-" {
            "number"
        } else {
            "bool"
        };
        if operator.lexeme == "-" && operand.get_typename() == "Number" {
            return Ok(());
        }

        if operator.lexeme == "!" && operand.get_typename() == "Bool" {
            return Ok(());
        }

        Err(self.error_handler.error(
            operator,
            LoxErrorsTypes::Type(format!("Operand must be of type {}", err_type)),
        ))
    }

    fn check_num_binary(
        &self,
        operator: &Token,
        left: &Literal,
        right: &Literal,
    ) -> Result<(), LoxResult> {
        if (left.get_typename() == "Number" && right.get_typename() == "Number")
            || (left.get_typename() == "String" || right.get_typename() == "String")
        {
            return Ok(());
        }

        match operator.token_type {
            TokenType::Minus | TokenType::Slash | TokenType::Star => Err(self.error_handler.error(
                operator,
                LoxErrorsTypes::Type("Operands must be numbers for".to_string()),
            )),
            TokenType::Plus => Err(self.error_handler.error(
                operator,
                LoxErrorsTypes::Type("Operands must be either numbers or strings for".to_string()),
            )),
            _ => Ok(()),
        }
    }

    fn check_compound_arithmetic(
        &self,
        operator: &Token,
        a: &Literal,
        b: &Literal,
    ) -> Result<(), LoxResult> {
        match operator.token_type {
            TokenType::PlusEqual => {
                if (a.get_typename() == "Number" && b.get_typename() == "Number")
                    || (a.get_typename() == "String" && b.get_typename() == "String")
                {
                    return Ok(());
                }
                Err(self.error_handler.error(
                    operator,
                    LoxErrorsTypes::Type(format!(
                        "Cannot add types '{}' and '{}' for",
                        a.get_typename(),
                        b.get_typename()
                    )),
                ))
            }
            TokenType::StarEqual => {
                if a.get_typename() == "Number" && b.get_typename() == "Number" {
                    return Ok(());
                } else if a.get_typename() == "String" && b.get_typename() == "String" {
                    return Err(self.error_handler.error(
                        operator,
                        LoxErrorsTypes::Type(format!(
                            "Cannot multiply on types '{}' and '{}' for",
                            a.get_typename(),
                            b.get_typename()
                        )),
                    ));
                } else if a.get_typename() == "String" || b.get_typename() == "Number" {
                    return Ok(());
                }
                Err(self.error_handler.error(
                    operator,
                    LoxErrorsTypes::Type(format!(
                        "Cannot multiply types '{}' and '{}' for",
                        a.get_typename(),
                        b.get_typename()
                    )),
                ))
            }
            TokenType::MinusEqual => {
                if a.get_typename() == "Number" && b.get_typename() == "Number" {
                    return Ok(());
                }

                Err(self.error_handler.error(
                    operator,
                    LoxErrorsTypes::Type(format!(
                        "Cannot subtract types '{}' and '{}' for",
                        a.get_typename(),
                        b.get_typename()
                    )),
                ))
            }
            TokenType::SlashEqual => {
                if a.get_typename() == "Number" && b.get_typename() == "Number" {
                    return Ok(());
                }

                Err(self.error_handler.error(
                    operator,
                    LoxErrorsTypes::Type(format!(
                        "Cannot divide types '{}' by '{}' for",
                        a.get_typename(),
                        b.get_typename()
                    )),
                ))
            }
            _ => Ok(()),
        }
    }

    pub fn set_is_repl(&mut self, is: bool) {
        self.is_repl = is;
    }

    fn check_arithmetic(
        &self,
        operator: &Token,
        expr: Result<Literal, String>,
    ) -> Result<Literal, LoxResult> {
        if let Err(err) = expr {
            return Err(self
                .error_handler
                .error(operator, LoxErrorsTypes::Type(err)));
        } else if let Ok(literal) = expr {
            return Ok(literal);
        }

        unreachable!("unreachable state reached: check_arithmetic");
    }

    fn evaluate_ternary(&self, expr: &TernaryExpr) -> Result<Literal, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;

        if let Literal::Bool(bool) = left {
            if bool {
                let middle = self.evaluate(expr.middle.clone())?;
                return Ok(middle);
            }
            let right = self.evaluate(expr.right.clone())?;
            return Ok(right);
        }
        Err(self.error_handler.error(
            &expr.operator,
            LoxErrorsTypes::Runtime("ternary operation failed.".to_string()),
        ))
    }

    fn do_comparison(
        &self,
        operator: &Token,
        left: Literal,
        right: Literal,
    ) -> Result<Literal, LoxResult> {
        let a = left.unwrap_number();
        let b = right.unwrap_number();

        match operator.token_type {
            TokenType::LessEqual => Ok(Literal::Bool(a <= b)),
            TokenType::Less => Ok(Literal::Bool(a < b)),
            TokenType::GreaterEqual => Ok(Literal::Bool(a >= b)),
            TokenType::Greater => Ok(Literal::Bool(a > b)),
            _ => Err(self.error_handler.error(
                operator,
                LoxErrorsTypes::Runtime("failed to perform comparison".to_string()),
            )),
        }
    }

    pub fn execute_block(
        &self,
        stmts: &[Rc<Stmt>],
        enclosing: Environment,
    ) -> Result<(), LoxResult> {
        let prev = self.environment.replace(Rc::new(RefCell::new(enclosing)));
        for stmt in stmts {
            if self.environment.borrow().borrow().continue_encountered {
                prev.borrow_mut().continue_encountered =
                    self.environment.borrow().borrow().continue_encountered;
                break;
            }
            if let Err(val) = self.execute(stmt.clone()) {
                match val {
                    LoxResult::Return(_) => {
                        self.environment.replace(prev);
                        return Err(val);
                    }
                    _ => return Err(val),
                }
            }
        }
        self.environment.replace(prev);
        Ok(())
    }
}

impl VisitorExpr<Literal> for Interpreter {
    fn visit_unary_expr(
        &self,
        _: Rc<Expr>,
        expr: &UnaryExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        let right = self.evaluate(expr.right.clone())?;

        self.check_num_unary(&expr.operator, &right)?;
        match expr.operator.token_type {
            TokenType::Minus => Ok(Literal::Number(-right.unwrap_number())),
            TokenType::Bang => Ok(Literal::Bool(!self.is_truthy(&right))),
            _ => unreachable!("Unary evaluation reached unreachable state."),
        }
    }

    fn visit_binary_expr(
        &self,
        _: Rc<Expr>,
        expr: &BinaryExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;
        let right = self.evaluate(expr.right.clone())?;

        self.check_num_binary(&expr.operator, &left, &right)?;
        match expr.operator.token_type {
            TokenType::Minus => self.check_arithmetic(&expr.operator, left - right),
            TokenType::Slash => self.check_arithmetic(&expr.operator, left / right),
            TokenType::Star => self.check_arithmetic(&expr.operator, left * right),
            TokenType::Plus => self.check_arithmetic(&expr.operator, left + right),
            TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(left, right))),
            TokenType::Equals => Ok(Literal::Bool(self.is_equal(left, right))),
            TokenType::Greater
            | TokenType::Less
            | TokenType::GreaterEqual
            | TokenType::LessEqual => self.do_comparison(&expr.operator, left, right),
            _ => todo!(""),
        }
    }

    fn visit_literal_expr(
        &self,
        _: Rc<Expr>,
        expr: &LiteralExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        Ok(expr.value.clone())
    }

    fn visit_grouping_expr(
        &self,
        _: Rc<Expr>,
        expr: &GroupingExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        self.evaluate(expr.expression.clone())
    }

    fn visit_ternary_expr(
        &self,
        _: Rc<Expr>,
        expr: &TernaryExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        self.evaluate_ternary(expr)
    }

    fn visit_variable_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &VariableExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        self.look_up_variable(&expr.name, &wrapper)
    }

    fn visit_assign_expr(
        &self,
        wrapper: Rc<Expr>,
        expr: &AssignExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        let value = self.evaluate(expr.value.clone())?;
        if let Some(dist) = self.locals.borrow().get(&wrapper) {
            self.environment.borrow().borrow_mut().mutate_at(*dist, &expr.name, value.dup())?;
        } else {
            self.globals.borrow_mut().mutate(&expr.name, value.dup())?;
        }

        Ok(value)
    }

    fn visit_logical_expr(
        &self,
        _: Rc<Expr>,
        expr: &LogicalExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        let left = self.evaluate(expr.left.clone())?;

        if expr.operator.token_type == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else if !self.is_truthy(&left) {
            return Ok(left);
        }

        self.evaluate(expr.right.clone())
    }

    fn visit_compoundassign_expr(
        &self,
        _: Rc<Expr>,
        expr: &CompoundAssignExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        let value = self.evaluate(expr.value.clone())?;
        let current_val = self.environment.borrow().borrow().get(&expr.name)?;
        self.is_single_expr.replace(false);
        self.check_compound_arithmetic(&expr.operator, &current_val, &value)?;
        match expr.operator.token_type {
            TokenType::PlusEqual => {
                if let Ok(v) = &current_val.add(value) {
                    self.environment
                        .borrow_mut()
                        .borrow_mut()
                        .mutate(&expr.name, v.dup())?;
                    return Ok(v.dup());
                }
            }
            TokenType::MinusEqual => {
                if let Ok(v) = &current_val.sub(value) {
                    self.environment
                        .borrow_mut()
                        .borrow_mut()
                        .mutate(&expr.name, v.dup())?;
                    return Ok(v.dup());
                }
            }
            TokenType::StarEqual => {
                if current_val.get_typename() == "String" {
                    let v = current_val
                        .unwrap_str()
                        .repeat(value.unwrap_number() as usize);
                    self.environment
                        .borrow_mut()
                        .borrow_mut()
                        .mutate(&expr.name, Literal::Str(v.clone()))?;
                    return Ok(Literal::Str(v));
                }
                if let Ok(v) = &current_val.mul(value) {
                    self.environment
                        .borrow_mut()
                        .borrow_mut()
                        .mutate(&expr.name, v.dup())?;
                    return Ok(v.dup());
                }
            }
            TokenType::SlashEqual => {
                if let Ok(v) = &current_val.div(value) {
                    self.environment
                        .borrow_mut()
                        .borrow_mut()
                        .mutate(&expr.name, v.dup())?;
                    return Ok(v.dup());
                }
            }
            _ => {}
        }
        Err(self
            .error_handler
            .error(&expr.operator, LoxErrorsTypes::Syntax("".to_string())))
    }

    fn visit_call_expr(&self, _: Rc<Expr>, expr: &CallExpr, _: u16) -> Result<Literal, LoxResult> {
        let callee = self.evaluate(expr.callee.clone())?;

        let mut args: Vec<Literal> = Vec::new();

        for arg in expr.args.iter() {
            args.push(self.evaluate(arg.clone())?);
        }

        if let Literal::Func(func) = callee {
            if args.len() != func.arity() {
                return Err(self.error_handler.error(
                    &expr.paren,
                    LoxErrorsTypes::Runtime(format!(
                        "Expected {} arguments but got {}",
                        func.arity(),
                        args.len()
                    )),
                ));
            }
            func.call(self, args)
        } else {
            Err(self.error_handler.error(
                &expr.paren,
                LoxErrorsTypes::Runtime("Can only call functions and classes".to_string()),
            ))
        }
    }

    fn visit_lambda_expr(
        &self,
        _: Rc<Expr>,
        expr: &LambdaExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        let function = LoxFunction::new_lambda(expr, &self.environment.borrow());
        Ok(Literal::Func(Callable {
            func: Rc::new(function),
        }))
    }

    fn visit_array_expr(
        &self,
        _: Rc<Expr>,
        expr: &ArrayExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        let mut arr = Vec::new();
        for val in &expr.arr {
            arr.push(self.evaluate(val.clone())?);
        }

        Ok(Literal::Array(arr))
    }

    fn visit_index_expr(
        &self,
        _: Rc<Expr>,
        expr: &IndexExpr,
        _: u16,
    ) -> Result<Literal, LoxResult> {
        let literal = self.evaluate(expr.identifier.clone())?;
        let index = self.evaluate(expr.index.clone())?;
        if let Literal::Array(arr) = literal {
            if index.get_typename() != "Number" {
                return Err(self.error_handler.error(
                    &expr.bracket,
                    LoxErrorsTypes::Runtime("Can only index arrays with numbers".to_string()),
                ));
            }

            let num = index.unwrap_number() as isize;
            let len = arr.len() as isize;
            if num > len {
                return Err(self.error_handler.error(
                    &expr.bracket,
                    LoxErrorsTypes::Runtime("Index out of bounds".to_string()),
                ));
            }
            if num < 0 {
                if len + num < 0 {
                    return Err(self.error_handler.error(
                        &expr.bracket,
                        LoxErrorsTypes::Runtime("Index out of bounds".to_string()),
                    ));
                }
                return Ok(arr.get((len + num) as usize).unwrap().dup());
            }
            Ok(arr.get(num as usize).unwrap().dup())
        } else {
            Err(self.error_handler.error(
                &expr.bracket,
                LoxErrorsTypes::Runtime("Can only index arrays".to_string()),
            ))
        }
    }
}

impl VisitorStmt<()> for Interpreter {
    fn visit_print_stmt(&self, _: Rc<Stmt>, stmt: &PrintStmt, _: u16) -> Result<(), LoxResult> {
        let value = self.evaluate(stmt.expr.clone())?;
        value.print_value();
        Ok(())
    }

    fn visit_expression_stmt(
        &self,
        _: Rc<Stmt>,
        stmt: &ExpressionStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        let val = self.evaluate(stmt.expr.clone())?;
        if self.is_repl && *self.is_single_expr.borrow() {
            val.print_value();
        }
        Ok(())
    }

    fn visit_let_stmt(&self, _: Rc<Stmt>, stmt: &LetStmt, _: u16) -> Result<(), LoxResult> {
        let val = if let Some(init) = &stmt.initializer {
            self.evaluate(init.clone())?
        } else {
            Literal::LiteralNone
        };
        self.environment
            .borrow_mut()
            .borrow_mut()
            .define(&stmt.name, val)?;
        Ok(())
    }

    fn visit_block_stmt(&self, _: Rc<Stmt>, stmt: &BlockStmt, _: u16) -> Result<(), LoxResult> {
        let e = Environment::new_enclosing(self.environment.borrow().clone());
        self.execute_block(&stmt.statements, e)?;
        Ok(())
    }

    fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt, _: u16) -> Result<(), LoxResult> {
        let cond = self.evaluate(stmt.condition.clone())?;
        if self.is_truthy(&cond) {
            self.execute(stmt.then_branch.clone())?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch.clone())?;
        }
        Ok(())
    }

    fn visit_while_stmt(&self, _: Rc<Stmt>, stmt: &WhileStmt, _: u16) -> Result<(), LoxResult> {
        while self.is_truthy(&self.evaluate(stmt.condition.clone())?) {
            if let Err(e) = self.execute(stmt.body.clone()) {
                match e {
                    LoxResult::Error(error) => return Err(LoxResult::Error(error)),
                    LoxResult::Break => break,
                    _ => {}
                }
            }
            if self.environment.borrow().borrow().continue_encountered {
                self.environment.borrow().borrow_mut().continue_encountered = false;
            }
        }
        Ok(())
    }

    fn visit_break_stmt(&self, _: Rc<Stmt>, _: &BreakStmt, _: u16) -> Result<(), LoxResult> {
        Err(LoxResult::Break)
    }

    fn visit_continue_stmt(
        &self,
        _: Rc<Stmt>,
        _: &ContinueStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        self.environment.borrow().borrow_mut().continue_encountered = true;
        Ok(())
    }

    fn visit_function_stmt(
        &self,
        _: Rc<Stmt>,
        stmt: &FunctionStmt,
        _: u16,
    ) -> Result<(), LoxResult> {
        let function = LoxFunction::new(stmt, &self.environment.borrow());
        self.environment.borrow_mut().borrow_mut().define(
            &stmt.name,
            Literal::Func(Callable {
                func: Rc::new(function),
            }),
        )?;
        Ok(())
    }

    fn visit_return_stmt(&self, _: Rc<Stmt>, stmt: &ReturnStmt, _: u16) -> Result<(), LoxResult> {
        let value = self.evaluate(stmt.value.clone())?;
        Err(LoxResult::Return(value))
    }

    fn visit_for_stmt(&self, _: Rc<Stmt>, stmt: &ForStmt, _: u16) -> Result<(), LoxResult> {
        if stmt.var.is_some() {
            self.execute(stmt.var.as_ref().unwrap().clone())?;
        }

        if stmt.condition.is_some() {
            while self.is_truthy(&self.evaluate(stmt.condition.as_ref().unwrap().clone())?) {
                if let Err(e) = self.execute(stmt.body.clone()) {
                    match e {
                        LoxResult::Error(error) => return Err(LoxResult::Error(error)),
                        LoxResult::Break => break,
                        _ => {}
                    }
                }
                if self.environment.borrow().borrow().continue_encountered {
                    if stmt.update_expr.is_some() {
                        self.evaluate(stmt.update_expr.as_ref().unwrap().clone())?;
                    }
                    continue;
                }
                if stmt.update_expr.is_some() {
                    self.evaluate(stmt.update_expr.as_ref().unwrap().clone())?;
                }
            }
        } else {
            loop {
                if let Err(e) = self.execute(stmt.body.clone()) {
                    match e {
                        LoxResult::Error(error) => return Err(LoxResult::Error(error)),
                        LoxResult::Break => break,
                        _ => {}
                    }
                }
                if self.environment.borrow().borrow().continue_encountered {
                    if stmt.update_expr.is_some() {
                        self.evaluate(stmt.update_expr.as_ref().unwrap().clone())?;
                    }
                    continue;
                }
                if stmt.update_expr.is_some() {
                    self.evaluate(stmt.update_expr.as_ref().unwrap().clone())?;
                }
            }
        }

        Ok(())
    }
}
