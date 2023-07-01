use crate::{
    lexer::{token::*, tokentype::TokenType},
    parser::{expr::*, stmt::*},
    errors::{
        LoxError,
        RuntimeError::RuntimeErrorHandler, 
        LoxErrorsTypes
    },
};
use std::cell::RefCell;
use std::rc::Rc;
use super::environment::Environment;

pub struct Interpreter {
    environment: RefCell<Rc<RefCell<Environment>>>,
    error_handler: RuntimeErrorHandler,
    is_repl: bool,
    is_single_expr: RefCell<bool>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            error_handler: RuntimeErrorHandler::new(),
            environment: RefCell::new(Rc::new(RefCell::new(Environment::new()))),
            is_repl: false,
            is_single_expr: RefCell::new(false),
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Box<Stmt>>) -> Result<(), LoxError> {
        if stmts.len() == 1 {
            if let Some(stmt) = stmts.get(0) {
                match **stmt {
                    Stmt::Expression(_) => self.is_single_expr.replace(true),
                    _ => self.is_single_expr.replace(false)
                };
            }
        }
        for stmt in stmts {
            self.execute(&stmt)?;
        }
        Ok(())
    }
    pub fn evaluate(&self, expr: &Expr) -> Result<Literal, LoxError> {
        let val = expr.accept(self, 0 as u16)?;
        Ok(val)
    }

    pub fn execute(&self, stmt: &Stmt) -> Result<(), LoxError> {
        stmt.accept(self, 0 as u16)
    }

    pub fn is_truthy(&self, right: &Literal) -> bool {
        !matches!(right, Literal::None | Literal::Bool(false))
    }

    pub fn is_equal(&self, left: Literal, right: Literal) -> bool {
        left.equals(right)
    }

    fn check_num_unary(&self, operator: &Token, operand: &Literal) -> Result<(), LoxError> {
        if operand.get_typename() == "Number" {
            return Ok(());
        }
        Err(self.error_handler.error(
                operator, 
                LoxErrorsTypes::SyntaxError("Operand must be a number".to_string()
        )))
    }

    fn check_num_binary(&self, operator: &Token, left: &Literal, right: &Literal) -> Result<(), LoxError> {
        if left.get_typename() == "Number" && right.get_typename() == "Number" {
            return Ok(());
        } else if left.get_typename() == "String" || right.get_typename() == "String" {
            return  Ok(());
        }

        match operator.token_type {
            TokenType::Minus |
            TokenType::Slash |
            TokenType::Star  => Err(self.error_handler.error(operator, LoxErrorsTypes::TypeError("Operands must be numbers for".to_string()))),
            TokenType::Plus  => Err(self.error_handler.error(operator, LoxErrorsTypes::TypeError("Operands must be either numbers or strings for".to_string()))),
            _ => Ok(())
        }
    }

    pub fn set_is_repl(&mut self, is: bool) {
        self.is_repl = is;
    }

    fn check_arithmetic(&self, operator: &Token, expr: Result<Literal, String>) -> Result<Literal, LoxError> {
        if let Err(err) = expr {
            return Err(self.error_handler.error(operator, LoxErrorsTypes::TypeError(err)));
        } else if let Ok(literal) = expr {
            return Ok(literal);
        }

        unreachable!("unreachable state reached: check_arithmetic");
    }

    fn evaluate_ternary(&self, expr: &TernaryExpr) -> Result<Literal, LoxError> {
        let left = self.evaluate(&expr.left)?;

        if let Literal::Bool(bool) = left {
            if bool {
                let middle = self.evaluate(&expr.middle)?;
                return Ok(middle);
            } 
            let right = self.evaluate(&expr.right)?;
            return Ok(right);
        }
        Err(self.error_handler.error(&expr.operator, LoxErrorsTypes::RuntimeError("ternary operation failed.".to_string())))
    }

    fn do_comparison(&self, operator: &Token, left: Literal, right: Literal) -> Result<Literal, LoxError> {
        let a = left.unwrap_number();
        let b = right.unwrap_number();

        match operator.token_type {
            TokenType::LessEqual    => Ok(Literal::Bool(a <= b)),
            TokenType::Less         => Ok(Literal::Bool(a < b)),
            TokenType::GreaterEqual => Ok(Literal::Bool(a >= b)),
            TokenType::Greater      => Ok(Literal::Bool(a > b)),
            _ => Err(self.error_handler.error(operator, LoxErrorsTypes::RuntimeError("failed to perform comparison".to_string())))
        }
    }

    fn execute_block(&self, stmts: &[Box<Stmt>], enclosing: Environment) -> Result<(), LoxError> {
        let prev = self.environment.replace(Rc::new(RefCell::new(enclosing)));
        self.environment.borrow_mut().borrow_mut().loop_started = prev.borrow().loop_started;
        stmts
            .iter()
            .try_for_each(|stmt| {
                if self.environment.borrow().borrow().break_encountered {
                    prev.borrow_mut().break_encountered = self.environment.borrow().borrow().break_encountered;
                    return Ok(())
                }
                self.execute(stmt)
            })?;
        self.environment.replace(prev);
        Ok(())
    }
}

impl VisitorExpr<Literal> for Interpreter {
    fn visit_unary_expr(&self, expr: &UnaryExpr, _: u16) -> Result<Literal, LoxError> {
        let right = self.evaluate(&expr.right)?;
        
        self.check_num_unary(&expr.operator, &right)?;
        match expr.operator.token_type {
            TokenType::Minus => Ok(Literal::Number(-right.unwrap_number())),
            TokenType::Bang => Ok(Literal::Bool(self.is_truthy(&right))),
            _ => unreachable!("Unary evaluation reached unreachable state.")
        }
    }

    fn visit_binary_expr(&self, expr: &BinaryExpr, _: u16) -> Result<Literal, LoxError> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;
        
        self.check_num_binary(&expr.operator, &left, &right)?;
        match expr.operator.token_type {
            TokenType::Minus => self.check_arithmetic(&expr.operator, left - right),
            TokenType::Slash => self.check_arithmetic(&expr.operator, left / right),
            TokenType::Star  => self.check_arithmetic(&expr.operator, left * right),
            TokenType::Plus  => self.check_arithmetic(&expr.operator, left + right),
            TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(left, right))),
            TokenType::Equals    => Ok(Literal::Bool(self.is_equal(left, right))),
            TokenType::Greater |
            TokenType::Less |
            TokenType::GreaterEqual |
            TokenType::LessEqual    => self.do_comparison(&expr.operator, left, right),
            _ => todo!("")
        }    
    }

    fn visit_literal_expr(&self, expr: &LiteralExpr, _: u16) -> Result<Literal, LoxError> {
        Ok(expr.value.clone())
    }

    fn visit_grouping_expr(&self, expr: &GroupingExpr, _: u16) -> Result<Literal, LoxError> {
        self.evaluate(&expr.expression)
    }

    fn visit_ternary_expr(&self, expr: &TernaryExpr, _: u16) -> Result<Literal, LoxError> {
        self.evaluate_ternary(&expr) 
    }

    fn visit_variable_expr(&self, expr: &VariableExpr, _: u16) -> Result<Literal, LoxError> {
        let val = self.environment.borrow().borrow().get(&expr.name)?;
        if val == Literal::LiteralNone {
            return Err(self.error_handler.error(
                &expr.name,
                LoxErrorsTypes::RuntimeError("Undefined variable".to_string()),
            ));
        }

        Ok(val)
    }

    fn visit_assign_expr(&self, expr: &AssignExpr, _: u16) -> Result<Literal, LoxError> {
        let value = self.evaluate(&expr.value)?;
        self.is_single_expr.replace(false);
        self.environment
            .borrow_mut()
            .borrow_mut()
            .mutate(&expr.name, value.dup())?;

        Ok(value)
    }

    fn visit_logical_expr(&self, expr: &LogicalExpr, _: u16) -> Result<Literal, LoxError> {
        let left = self.evaluate(&expr.left)?;

        if expr.operator.token_type == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !self.is_truthy(&left) {
                return Ok(left);
            }
        }

        Ok(self.evaluate(&expr.right)?)
    }
}

impl VisitorStmt<()> for Interpreter {
    fn visit_print_stmt(&self, stmt: &PrintStmt, _: u16) -> Result<(), LoxError> {
        let value = self.evaluate(&stmt.expr)?;
        value.print_value();
        Ok(())
    }

    fn visit_expression_stmt(&self, stmt: &ExpressionStmt, _: u16) -> Result<(), LoxError> {
        let val = self.evaluate(&stmt.expr)?;
        if self.is_repl && *self.is_single_expr.borrow() {
            val.print_value();
        }
        Ok(())
    }

    fn visit_let_stmt(&self, stmt: &LetStmt, _: u16) -> Result<(), LoxError> {
        let val = if let Some(init) = &stmt.initializer {
            self.evaluate(&init)?
        } else {
            Literal::LiteralNone
        };
        self.environment
            .borrow_mut()
            .borrow_mut()
            .define(&stmt.name, val)?;
        Ok(())
    }

    fn visit_block_stmt(&self, stmt: &BlockStmt, _: u16) -> Result<(), LoxError> {
        let e = Environment::new_enclosing(self.environment.borrow().clone());
        self.execute_block(&stmt.statements, e)?;
        Ok(())
    }

    fn visit_if_stmt(&self, stmt: &IfStmt, _: u16) -> Result<(), LoxError> {
        let cond = self.evaluate(&stmt.condition)?;
        if self.is_truthy(&cond) {
            self.execute(&stmt.then_branch)?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(&else_branch)?;
        }
        Ok(())
    }

    fn visit_while_stmt(&self, stmt: &WhileStmt, _: u16) -> Result<(), LoxError> {
        self.environment.borrow_mut().borrow_mut().loop_started = true;
        while self.is_truthy(&self.evaluate(&stmt.condition)?) {
            self.execute(&stmt.body)?;
            if self.environment.borrow().borrow().break_encountered {
                self.environment.borrow_mut().borrow_mut().loop_started = false;
                self.environment.borrow_mut().borrow_mut().break_encountered = false;
                return Ok(());
            }
        }
        Ok(()) 
    }

    fn visit_break_stmt(&self, stmt: &BreakStmt, _: u16) -> Result<(), LoxError> {
        if self.environment.borrow().borrow().loop_started {
            self.environment.borrow_mut().borrow_mut().break_encountered = true;
            return Ok(());
        } else {
            return Err(self.error_handler.error(
                &stmt.token, 
                LoxErrorsTypes::RuntimeError("Found 'break' outside loop block".to_string())
            ));
        }
    }
}
