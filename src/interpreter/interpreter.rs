use crate::{
    lexer::{token::*, tokentype::TokenType},
    parser::expr::*, errors::{LoxError, RuntimeError::RuntimeErrorHandler, LoxErrorsTypes},
};

pub struct Interpreter {
    error_handler: RuntimeErrorHandler,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            error_handler: RuntimeErrorHandler::new()
        }
    }

    pub fn evaluate(&self, expr: &Expr) -> Result<Literal, LoxError> {
        expr.accept(self, 0 as u16)
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
        } else if left.get_typename() == "String" && right.get_typename() == "String" {
            return  Ok(());
        }

        match operator.token_type {
            TokenType::Minus |
            TokenType::Slash |
            TokenType::Star  => Err(self.error_handler.error(operator, LoxErrorsTypes::TypeError("Operands must be numbers for".to_string()))),
            TokenType::Plus  => Err(self.error_handler.error(operator, LoxErrorsTypes::TypeError("Operands must be either numbers or strings  for".to_string()))),
            _ => Ok(())
        }
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
}

impl Visitor<Literal> for Interpreter {
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
            TokenType::Minus =>  {
                self.check_arithmetic(&expr.operator, left - right)
            },
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

    fn visit_ternary_expr(&self, expr: &TernaryExpr, depth: u16) -> Result<Literal, LoxError> {
        self.evaluate_ternary(&expr) 
    }
}
