use crate::{
    error::{loxerrorhandler::LoxErrorHandler, LoxErrorsTypes, LoxResult},
    lexer::literal::*,
    lexer::token::Token,
    lexer::tokentype::TokenType,
    parser::expr::{BinaryExpr, Expr, GroupingExpr, LiteralExpr, UnaryExpr},
};
use std::rc::Rc;

use super::{expr::*, stmt::*};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    curr: usize,
    error_handler: LoxErrorHandler,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens,
            curr: 0,
            error_handler: LoxErrorHandler::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, LoxResult> {
        let mut statments: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            statments.push(self.declaration()?);
        }

        Ok(statments)
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxResult> {
        let name = self.consume(
            TokenType::Identifier,
            LoxErrorsTypes::Syntax("Expected name for identifier".to_string()),
        )?;

        let initializer = if self.is_match(vec![TokenType::Assign]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expect ';' after".to_string()),
        )?;

        Ok(Stmt::Let(LetStmt::new(name, initializer)))
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, LoxResult> {
        let name = self.consume(
            TokenType::Identifier,
            LoxErrorsTypes::Syntax(format!("Expected {kind} name after")),
        )?;

        self.consume(
            TokenType::LeftParen,
            LoxErrorsTypes::Syntax("Expected '(' after".to_string()),
        )?;

        let mut params: Vec<Token> = Vec::new();

        if !self.check(TokenType::RightParen) {
            params.push(self.consume(
                TokenType::Identifier,
                LoxErrorsTypes::Syntax("Expected parameter identifier".to_string()),
            )?);

            while self.match_single_token(TokenType::Comma) {
                if params.len() >= 255 {
                    self.error_handler.error(
                        self.peek(),
                        LoxErrorsTypes::Syntax("Can't have more than 255 parameters".to_string()),
                    );
                }
                params.push(self.consume(
                    TokenType::Identifier,
                    LoxErrorsTypes::Syntax("Expected parameter identifier".to_string()),
                )?);
            }
        }

        self.consume(
            TokenType::RightParen,
            LoxErrorsTypes::Syntax("Expected ')' after parameters".to_string()),
        )?;

        self.consume(
            TokenType::LeftBrace,
            LoxErrorsTypes::Syntax(format!("Expected '{{' before {kind} body")),
        )?;

        let body: Vec<Stmt> = self.block_stmt()?;

        Ok(Stmt::Function(FunctionStmt::new(
            name,
            Rc::new(params),
            Rc::new(body),
        )))
    }

    fn declaration(&mut self) -> Result<Stmt, LoxResult> {
        let result = if self.match_single_token(TokenType::Let) {
            self.var_declaration()
        } else if self.match_single_token(TokenType::DefFn) {
            self.function("function")
        } else {
            self.statement()
        };
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn block_stmt(&mut self) -> Result<Vec<Stmt>, LoxResult> {
        let mut stmts: Vec<Stmt> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(
            TokenType::RightBrace,
            LoxErrorsTypes::Parse("Expected '}' after block".to_string()),
        )?;
        Ok(stmts)
    }

    fn print_statement(&mut self) -> Result<Stmt, LoxResult> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after".to_string()),
        )?;
        Ok(Stmt::Print(PrintStmt::new(expr)))
    }

    fn expr_statement(&mut self) -> Result<Stmt, LoxResult> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after".to_string()),
        )?;
        Ok(Stmt::Expression(ExpressionStmt::new(expr)))
    }

    fn if_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(
            TokenType::LeftParen,
            LoxErrorsTypes::Syntax("Expected '(' after".to_string()),
        )?;

        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            LoxErrorsTypes::Syntax("Expected ')' after".to_string()),
        )?;

        let then_branch = Box::new(self.statement()?);
        let mut alternative: Option<Box<Stmt>> = None;

        if self.match_single_token(TokenType::Elif) {
            alternative = Some(Box::new(self.if_statement()?));
        }

        if self.match_single_token(TokenType::Else) {
            alternative = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If(IfStmt::new(condition, then_branch, alternative)))
    }

    fn while_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(
            TokenType::LeftParen,
            LoxErrorsTypes::Syntax("Expected '(' after".to_string()),
        )?;

        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            LoxErrorsTypes::Syntax("Expected ')' after".to_string()),
        )?;

        let body = Box::new(self.statement()?);
        Ok(Stmt::While(WhileStmt::new(condition, body)))
    }

    fn for_statement(&mut self) -> Result<Stmt, LoxResult> {
        self.consume(
            TokenType::LeftParen,
            LoxErrorsTypes::Syntax("Expected '(' after".to_string()),
        )?;

        let mut initializer: Option<Box<Stmt>> = None;

        if self.peek().token_type == TokenType::Let {
            self.match_single_token(TokenType::Let);
            initializer = Some(Box::new(self.var_declaration()?));
        } else if !self.check(TokenType::Semicolon) {
            initializer = Some(Box::new(self.expr_statement()?));
        } else {
            self.consume(
                TokenType::Semicolon,
                LoxErrorsTypes::Syntax(
                    "Expected variable declaration or expression, got".to_string(),
                ),
            )?;
        }

        let mut condition: Option<Expr> = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after loop condition".to_string()),
        )?;

        let mut increment: Option<Expr> = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?);
        }

        self.consume(
            TokenType::RightParen,
            LoxErrorsTypes::Syntax("Expected ')' after for clauses".to_string()),
        )?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::For(ForStmt::new(
            initializer,
            condition,
            increment,
            body,
        )))
    }

    fn break_statement(&mut self) -> Result<Stmt, LoxResult> {
        let tok = self.previous();
        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after statement".to_string()),
        )?;
        Ok(Stmt::Break(BreakStmt::new(tok)))
    }

    fn continue_statement(&mut self) -> Result<Stmt, LoxResult> {
        let tok = self.previous();
        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after statement".to_string()),
        )?;
        Ok(Stmt::Continue(ContinueStmt::new(tok)))
    }

    fn return_statement(&mut self) -> Result<Stmt, LoxResult> {
        let keyword = self.previous();
        let mut value = Expr::Literal(LiteralExpr::new(Literal::None));
        if !self.check(TokenType::Semicolon) {
            value = self.expression()?;
        }

        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after".to_string()),
        )?;
        Ok(Stmt::Return(ReturnStmt::new(keyword, value)))
    }

    fn statement(&mut self) -> Result<Stmt, LoxResult> {
        if self.match_single_token(TokenType::Print) {
            return self.print_statement();
        }

        if self.match_single_token(TokenType::LeftBrace) {
            return Ok(Stmt::Block(BlockStmt::new(self.block_stmt()?)));
        }

        if self.match_single_token(TokenType::If) {
            return self.if_statement();
        }

        if self.match_single_token(TokenType::While) {
            return self.while_statement();
        }

        if self.match_single_token(TokenType::For) {
            return self.for_statement();
        }

        if self.match_single_token(TokenType::Break) {
            return self.break_statement();
        }

        if self.match_single_token(TokenType::Continue) {
            return self.continue_statement();
        }

        if self.match_single_token(TokenType::Return) {
            return self.return_statement();
        }
        self.expr_statement()
    }

    fn consume(&mut self, token: TokenType, error: LoxErrorsTypes) -> Result<Token, LoxResult> {
        if self.check(token) {
            return Ok(self.advance().dup());
        }

        Err(self.error_handler.error(&self.previous(), error))
    }

    fn lambda_fn(&mut self) -> Result<Expr, LoxResult> {
        self.consume(
            TokenType::LeftParen,
            LoxErrorsTypes::Syntax("Expected '(' after function declaration".to_string()),
        )?;

        let mut params: Vec<Token> = Vec::new();

        if !self.check(TokenType::RightParen) {
            params.push(self.consume(
                TokenType::Identifier,
                LoxErrorsTypes::Syntax("Expected identifier got".to_string()),
            )?);
            while self.match_single_token(TokenType::Comma) {
                if params.len() >= 255 {
                    self.error_handler.error(
                        self.peek(),
                        LoxErrorsTypes::Syntax("Can't have more than 255 parameters".to_string()),
                    );
                }
                params.push(self.consume(
                    TokenType::Identifier,
                    LoxErrorsTypes::Syntax("Expected identifier got".to_string()),
                )?);
            }
        }

        self.consume(
            TokenType::RightParen,
            LoxErrorsTypes::Syntax("Expected ')' after parameters".to_string()),
        )?;

        self.consume(
            TokenType::LeftBrace,
            LoxErrorsTypes::Syntax("Expected '{' before function body".to_string()),
        )?;

        let body = self.block_stmt()?;

        Ok(Expr::Lambda(LambdaExpr::new(
            Rc::new(params),
            Rc::new(body),
        )))
    }

    fn primary(&mut self) -> Result<Expr, LoxResult> {
        if self.match_single_token(TokenType::False) {
            return Ok(Expr::Literal(LiteralExpr::new(Literal::Bool(false))));
        }

        if self.match_single_token(TokenType::True) {
            return Ok(Expr::Literal(LiteralExpr::new(Literal::Bool(true))));
        }

        if self.match_single_token(TokenType::None) {
            return Ok(Expr::Literal(LiteralExpr::new(Literal::None)));
        }

        if self.match_single_token(TokenType::Identifier) {
            return Ok(Expr::Variable(VariableExpr::new(self.previous())));
        }

        if self.is_match(vec![TokenType::Number, TokenType::String]) {
            match self.previous().literal.as_ref().unwrap() {
                Literal::Number(literal) => {
                    return Ok(Expr::Literal(LiteralExpr::new(Literal::Number(*literal))))
                }
                Literal::Str(literal) => {
                    return Ok(Expr::Literal(LiteralExpr::new(Literal::Str(
                        literal.to_string(),
                    ))))
                }
                _ => {}
            }
        }

        if self.match_single_token(TokenType::DefLambda) {
            return self.lambda_fn();
        }

        if self.match_single_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(
                TokenType::RightParen,
                LoxErrorsTypes::Syntax("Expected ')' after expression, at".to_string()),
            )?;
            return Ok(Expr::Grouping(GroupingExpr::new(expr)));
        }

        if self.curr == 0 {
            return Err(self.error_handler.error(
                self.peek(),
                LoxErrorsTypes::Syntax(format!("Unexpected token",)),
            ));
        }

        Err(self.error_handler.error(
            &self.previous(),
            LoxErrorsTypes::Syntax("Expected expression after".to_string()),
        ))
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, LoxResult> {
        let mut args: Vec<Expr> = Vec::new();
        if self.check(TokenType::Semicolon) {
            return Err(self.error_handler.error(
                self.peek(), 
                LoxErrorsTypes::Syntax("Expected ')' after".to_string())
            ))
        }
        if !self.check(TokenType::RightParen) {
            if args.len() >= 255 {
                self.error_handler.error(
                    self.peek(),
                    LoxErrorsTypes::Parse("Cannot have more than 255 arguments".to_string()),
                );
            }
            args.push(self.expression()?);
            while self.match_single_token(TokenType::Comma) {
                args.push(self.expression()?);
            }
        }

        let paren = self.consume(
            TokenType::RightParen,
            LoxErrorsTypes::Syntax("Expected ')' after".to_string()),
        )?;
        Ok(Expr::Call(CallExpr::new(callee, paren, args)))
    }

    fn call(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.primary()?;

        loop {
            if self.match_single_token(TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxResult> {
        if self.is_match(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            return Ok(Expr::Unary(UnaryExpr::new(operator, self.unary()?)));
        }

        self.call()
    }

    fn factor(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.unary()?;

        while self.is_match(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            expr = Expr::Binary(BinaryExpr::new(expr, operator, self.unary()?));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.factor()?;

        while self.is_match(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            expr = Expr::Binary(BinaryExpr::new(expr, operator, self.factor()?));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.term()?;

        while self.is_match(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            expr = Expr::Binary(BinaryExpr::new(expr, operator, self.term()?));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.comparison()?;

        while self.is_match(vec![TokenType::BangEqual, TokenType::Equals]) {
            let operator = self.previous();
            expr = Expr::Binary(BinaryExpr::new(expr, operator, self.comparison()?));
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.equality()?;

        while self.match_single_token(TokenType::And) {
            let op = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(LogicalExpr::new(expr, op, right));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, LoxResult> {
        let mut expr = self.and()?;

        while self.match_single_token(TokenType::Or) {
            let op = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(LogicalExpr::new(expr, op, right));
        }
        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Expr, LoxResult> {
        let expr = self.or()?;

        if self.match_single_token(TokenType::QuestionMark) {
            let operator = self.tokens.get(self.curr - 1).unwrap().dup();
            let middle = self.expression()?;
            if self.match_single_token(TokenType::Colon) {
                let colon = self.previous();
                return Ok(Expr::Ternary(TernaryExpr::new(
                    expr,
                    operator,
                    middle,
                    colon,
                    self.expression()?,
                )));
            }
            return Err(self.error_handler.error(
                &self.previous(),
                LoxErrorsTypes::Syntax("Incomplete ternary operation,".to_string()),
            ));
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr, LoxResult> {
        let expr = self.ternary()?;

        if self.match_single_token(TokenType::Assign) {
            let token = self.previous();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(var) => {
                    let name = var.name;
                    return Ok(Expr::Assign(AssignExpr::new(name, value)));
                }
                _ => {
                    return Err(self.error_handler.error(
                        &token,
                        LoxErrorsTypes::Parse("Invalid assignment target".to_string()),
                    ));
                }
            }
        }

        Ok(expr)
    }

    fn compound_assignment(&mut self) -> Result<Expr, LoxResult> {
        let expr = self.assignment()?;

        if self.is_match(vec![
            TokenType::StarEqual,
            TokenType::SlashEqual,
            TokenType::PlusEqual,
            TokenType::MinusEqual,
        ]) {
            let token = self.previous();
            let value = self.primary()?;

            match self.peek().token_type {
                TokenType::SlashEqual
                | TokenType::StarEqual
                | TokenType::MinusEqual
                | TokenType::PlusEqual => {
                    return Err(self.error_handler.error(
                        self.peek(),
                        LoxErrorsTypes::Syntax("Cannot chain compound assignment".to_string()),
                    ))
                }
                _ => {}
            }

            match expr {
                Expr::Variable(var) => {
                    let name = var.name;
                    return Ok(Expr::CompoundAssign(CompoundAssignExpr::new(
                        name, token, value,
                    )));
                }
                _ => {
                    return Err(self.error_handler.error(
                        &token,
                        LoxErrorsTypes::Syntax("Invalid assignment target for".to_string()),
                    ));
                }
            }
        }
        Ok(expr)
    }

    fn expression(&mut self) -> Result<Expr, LoxResult> {
        self.compound_assignment()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.curr).unwrap()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.curr - 1).unwrap().dup()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.curr += 1;
        }
        self.previous()
    }

    fn check(&self, token: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token
    }

    fn match_single_token(&mut self, token: TokenType) -> bool {
        if self.check(token) {
            self.advance();
            return true;
        }
        false
    }

    fn is_match(&mut self, toks: Vec<TokenType>) -> bool {
        for token in toks {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            match self.peek().token_type {
                TokenType::Class
                | TokenType::DefFn
                | TokenType::Let
                | TokenType::For
                | TokenType::If
                | TokenType::Else
                | TokenType::Return
                | TokenType::Print
                | TokenType::While => return,
                _ => (),
            };

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use super::*;
    use crate::Scanner;

    struct AstTraverser<'a> {
        statements: &'a Vec<Stmt>,
        strings: Vec<String>,
    }

    impl<'a> AstTraverser<'a> {
        fn new(statements: &'a Vec<Stmt>) -> Self {
            Self {
                statements,
                strings: Vec::new(),
            }
        }

        fn gen(&mut self) -> Result<&Vec<String>, LoxResult> {
            for stmt in self.statements {
                let str = self.execute(&stmt)?;
                self.strings.push(str);
            }
            Ok(&self.strings)
        }

        fn evaluate(&self, expr: &Expr) -> Result<String, LoxResult> {
            expr.accept(self, 0_u16)
        }

        fn execute(&self, stmt: &Stmt) -> Result<String, LoxResult> {
            stmt.accept(self, 0_u16)
        }
    }

    impl<'a> VisitorExpr<String> for AstTraverser<'a> {
        fn visit_call_expr(&self, expr: &CallExpr, _: u16) -> Result<String, LoxResult> {
            let callee = self.evaluate(&expr.callee)?;
            let mut str = format!("CallExpr {callee}");
            for arg in &expr.args {
                str = str.add(&self.evaluate(&arg)?);
            }

            Ok(str)
        }

        fn visit_unary_expr(&self, expr: &UnaryExpr, _: u16) -> Result<String, LoxResult> {
            let op = &expr.operator.lexeme;
            let right = self.evaluate(&expr.right)?;
            let str = format!("UnaryExpr {op} {right}");
            Ok(str)
        }

        fn visit_binary_expr(&self, expr: &BinaryExpr, _: u16) -> Result<String, LoxResult> {
            let left = self.evaluate(&expr.left)?;
            let right = self.evaluate(&expr.right)?;
            let str = format!("BinaryExpr {left} {} {right}", expr.operator.lexeme);
            Ok(str)
        }

        fn visit_assign_expr(&self, expr: &AssignExpr, _: u16) -> Result<String, LoxResult> {
            let val = self.evaluate(&expr.value)?;
            Ok(format!("AssignExpr {} = {val}", expr.name.lexeme))
        }

        fn visit_lambda_expr(&self, expr: &LambdaExpr, _: u16) -> Result<String, LoxResult> {
            let mut params = "".to_string();
            for (i, param) in expr.params.iter().enumerate() {
                params.push_str(&param.lexeme);
                if expr.params.len() - 1 != i {
                    params.push_str(", ");
                }
            }
            let mut body = "{ ".to_string();

            for stmt in expr.body.iter() {
                body.push_str(&self.execute(&stmt)?);
            }

            body.push_str(" }");
            let str = format!("LambdaExpr lm({}) {}", params.trim(), body);

            Ok(str)
        }

        fn visit_logical_expr(&self, expr: &LogicalExpr, _: u16) -> Result<String, LoxResult> {
            let left = self.evaluate(&expr.left)?;
            let right = self.evaluate(&expr.right)?;
            let str = format!("LogicalExpr {left} {} {right}", expr.operator.lexeme);

            Ok(str)
        }

        fn visit_literal_expr(&self, expr: &LiteralExpr, _: u16) -> Result<String, LoxResult> {
            let mut str = format!("LiteralExpr {}", expr.value);
            Ok(str)
        }

        fn visit_ternary_expr(&self, expr: &TernaryExpr, _: u16) -> Result<String, LoxResult> {
            let left = self.evaluate(&expr.left)?;
            let middle = self.evaluate(&expr.middle)?;
            let right = self.evaluate(&expr.right)?;
            let str = format!("TernaryExpr {left} ? {middle} : {right}");

            Ok(str)
        }

        fn visit_grouping_expr(&self, expr: &GroupingExpr, _: u16) -> Result<String, LoxResult> {
            let inner = self.evaluate(&expr.expression)?;
            let str = format!("GroupingExpr ({inner})");

            Ok(str)
        }

        fn visit_variable_expr(&self, expr: &VariableExpr, _: u16) -> Result<String, LoxResult> {
            let str = format!("VariableExpr {}", expr.name.lexeme);
            Ok(str)
        }

        fn visit_compoundassign_expr(
            &self,
            expr: &CompoundAssignExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let val = self.evaluate(&expr.value)?;
            let str = format!(
                "CompoundAssignExpr {} {} {val}",
                expr.name.lexeme, expr.operator.lexeme
            );
            Ok(str)
        }
    }

    impl<'a> VisitorStmt<String> for AstTraverser<'a> {
        fn visit_if_stmt(&self, stmt: &IfStmt, _: u16) -> Result<String, LoxResult> {
            let mut str = format!("IfStmt ");

            Ok(str)
        }

        fn visit_let_stmt(&self, stmt: &LetStmt, _: u16) -> Result<String, LoxResult> {
            let mut initializer = "not initialized".to_string();
            if stmt.initializer.is_some() {
                initializer = self.evaluate(stmt.initializer.as_ref().unwrap())?;
            }
            let str = format!("LetStmt {} = {}", stmt.name.lexeme, initializer);
            Ok(str)
        }

        fn visit_for_stmt(&self, stmt: &ForStmt, _: u16) -> Result<String, LoxResult> {
            let mut str = format!("ForStmt");

            Ok(str)
        }

        fn visit_print_stmt(&self, stmt: &PrintStmt, _: u16) -> Result<String, LoxResult> {
            let expr = self.evaluate(&stmt.expr)?;
            let str = format!("PrintStmt {expr}");

            Ok(str)
        }

        fn visit_block_stmt(&self, stmt: &BlockStmt, _: u16) -> Result<String, LoxResult> {
            let mut str = format!("BlockStmt");

            Ok(str)
        }

        fn visit_while_stmt(&self, stmt: &WhileStmt, _: u16) -> Result<String, LoxResult> {
            let mut str = format!("WhileStmt");

            Ok(str)
        }

        fn visit_break_stmt(&self, stmt: &BreakStmt, _: u16) -> Result<String, LoxResult> {
            let mut str = format!("BreakStmt");

            Ok(str)
        }

        fn visit_return_stmt(&self, stmt: &ReturnStmt, _: u16) -> Result<String, LoxResult> {
            let val = self.evaluate(&stmt.value)?;
            let str = format!("ReturnStmt {val}");

            Ok(str)
        }

        fn visit_continue_stmt(&self, stmt: &ContinueStmt, _: u16) -> Result<String, LoxResult> {
            let mut str = format!("ContinueStmt");

            Ok(str)
        }

        fn visit_function_stmt(&self, stmt: &FunctionStmt, _: u16) -> Result<String, LoxResult> {
            let mut str = format!("FunctionStmt ");

            Ok(str)
        }

        fn visit_expression_stmt(
            &self,
            stmt: &ExpressionStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            let expr = self.evaluate(&stmt.expr)?;
            let str = format!("ExpressionStmt {expr}");
            Ok(str)
        }
    }

    fn perform(src: &str, expected: Vec<&str>) {
        let e_handler = LoxErrorHandler::new();
        let mut scanner = Scanner::new(src, &e_handler);
        if let Ok(tokens) = scanner.scan_tokens() {
            let mut parser = Parser::new(tokens);
            if let Ok(ast) = parser.parse() {
                let mut tr = AstTraverser::new(&ast);
                match tr.gen() {
                    Ok(strings) => {
                        for (a, b) in strings.iter().zip(expected.iter()) {
                            assert_eq!(a, b);
                        }
                    }
                    Err(_) => panic!("failed {src}"),
                }
            } else {
                panic!("failed {src}")
            }
        } else {
            panic!("failed {src}")
        }
    }

    fn perform_err(src: &str, expected: LoxErrorsTypes) {
        let e_handler = LoxErrorHandler::new();
        let mut scanner = Scanner::new(src, &e_handler);
        if let Ok(tokens) = scanner.scan_tokens() {
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(_) => panic!("failed {src}"),
                Err(err) => match err {
                    LoxResult::Error(err) => assert_eq!(err.error_type, expected),
                    _ => {}
                },
            }
        } else {
            panic!("failed {src}")
        }
    }

    #[test]
    fn binary_add_numbers() {
        let src = "1 + 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } + LiteralExpr Number { 2 }"];
        perform(src, expected);
    }

    #[test]
    fn binary_sub_numbers() {
        let src = "1 - 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } - LiteralExpr Number { 2 }"];
        perform(src, expected);
    }

    #[test]
    fn binary_mul_numbers() {
        let src = "1 * 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } * LiteralExpr Number { 2 }"];
        perform(src, expected);
    }

    #[test]
    fn binary_div_numbers() {
        let src = "1 / 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } / LiteralExpr Number { 2 }"];
        perform(src, expected);
    }

    #[test]
    fn binary_predence_number() {
        let src = "6 / 3 - 2;";
        let expected = vec!["ExpressionStmt BinaryExpr BinaryExpr LiteralExpr Number { 6 } / LiteralExpr Number { 3 } - LiteralExpr Number { 2 }"];
        perform(src, expected);
    }

    #[test]
    fn binary_grouping_numbers() {
        let src = "(1 / 2);";
        let expected = vec!["ExpressionStmt GroupingExpr (BinaryExpr LiteralExpr Number { 1 } / LiteralExpr Number { 2 })"];
        perform(src, expected);
    }
    #[test]
    fn binary_add_strings() {
        let src = "\"str\" + \"str\";";
        let expected = vec!["ExpressionStmt BinaryExpr LiteralExpr String { \"str\" } + LiteralExpr String { \"str\" }"];
        perform(src, expected)
    }

    #[test]
    fn binary_add_grouping() {
        let src = "(1 * 2) + 2;";
        let expected = vec!["ExpressionStmt BinaryExpr GroupingExpr (BinaryExpr LiteralExpr Number { 1 } * LiteralExpr Number { 2 }) + LiteralExpr Number { 2 }"];
        perform(src, expected)
    }

    #[test]
    fn binary_add_complex() {
        let src = "(1 * 2) + (6 / 3);";
        let expected = vec!["ExpressionStmt BinaryExpr GroupingExpr (BinaryExpr LiteralExpr Number { 1 } * LiteralExpr Number { 2 }) + GroupingExpr (BinaryExpr LiteralExpr Number { 6 } / LiteralExpr Number { 3 })"];
        perform(src, expected)
    }

    #[test]
    fn binary_add_fn_call() {
        let src = "function() + function();";
        let expected = vec!["ExpressionStmt BinaryExpr CallExpr VariableExpr function + CallExpr VariableExpr function"];
        perform(src, expected)
    }

    #[test]
    fn grouping_err() {
        let src = "(1 + 2 + 3";
        let expected = LoxErrorsTypes::Syntax("Expected ')' after expression, at".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn expr_semicolon_err() {
        let src = "13";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn binary_variable_add() {
        let src = "a + b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a + VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_variable_sub() {
        let src = "a - b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a - VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_variable_mul() {
        let src = "a * b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a * VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_variable_div() {
        let src = "a / b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a / VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_variable_grouping_binary() {
        let src = "(a - c) / b;";
        let expected = vec!["ExpressionStmt BinaryExpr GroupingExpr (BinaryExpr VariableExpr a - VariableExpr c) / VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binry_fn_call_grouping() {
        let src = "(a() - b()) * c();";
        let expected = vec!["ExpressionStmt BinaryExpr GroupingExpr (BinaryExpr CallExpr VariableExpr a - CallExpr VariableExpr b) * CallExpr VariableExpr c"];
        perform(src, expected)
    }

    #[test]
    fn binary_super_complex() {
        let src = "(a() - x) * c() + (x / 2);";
        let expected = vec!["ExpressionStmt BinaryExpr BinaryExpr GroupingExpr (BinaryExpr CallExpr VariableExpr a - VariableExpr x) * CallExpr VariableExpr c + GroupingExpr (BinaryExpr VariableExpr x / LiteralExpr Number { 2 })"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_equality() {
        let src = "1 == 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } == LiteralExpr Number { 2 }"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_inequality() {
        let src = "1 != 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } != LiteralExpr Number { 2 }"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_greater() {
        let src = "1 > 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } > LiteralExpr Number { 2 }"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_greater_equal() {
        let src = "1 >= 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } >= LiteralExpr Number { 2 }"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_less() {
        let src = "1 < 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } < LiteralExpr Number { 2 }"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_less_equal() {
        let src = "1 <= 2;";
        let expected =
            vec!["ExpressionStmt BinaryExpr LiteralExpr Number { 1 } <= LiteralExpr Number { 2 }"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_equality() {
        let src = "a == b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a == VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_inequality() {
        let src = "a != b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a != VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_greater() {
        let src = "a > b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a > VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_greater_equal() {
        let src = "a >= b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a >= VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_less() {
        let src = "a < b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a < VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_less_equal() {
        let src = "a <= b;";
        let expected = vec!["ExpressionStmt BinaryExpr VariableExpr a <= VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_fn_equality() {
        let src = "a() == b();";
        let expected =
            vec!["ExpressionStmt BinaryExpr CallExpr VariableExpr a == CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_fn_inequality() {
        let src = "a() != b();";
        let expected =
            vec!["ExpressionStmt BinaryExpr CallExpr VariableExpr a != CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_fn_greater() {
        let src = "a() > b();";
        let expected =
            vec!["ExpressionStmt BinaryExpr CallExpr VariableExpr a > CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_fn_greater_equal() {
        let src = "a() >= b();";
        let expected =
            vec!["ExpressionStmt BinaryExpr CallExpr VariableExpr a >= CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_fn_less() {
        let src = "a() < b();";
        let expected =
            vec!["ExpressionStmt BinaryExpr CallExpr VariableExpr a < CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_fn_less_equal() {
        let src = "a() <= b();";
        let expected =
            vec!["ExpressionStmt BinaryExpr CallExpr VariableExpr a <= CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_grouping_equality() {
        let src = "(a == b);";
        let expected =
            vec!["ExpressionStmt GroupingExpr (BinaryExpr VariableExpr a == VariableExpr b)"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_grouping_inequality() {
        let src = "(a != b);";
        let expected =
            vec!["ExpressionStmt GroupingExpr (BinaryExpr VariableExpr a != VariableExpr b)"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_grouping_greater() {
        let src = "(a > b);";
        let expected =
            vec!["ExpressionStmt GroupingExpr (BinaryExpr VariableExpr a > VariableExpr b)"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_grouping_greater_equal() {
        let src = "(a >= b);";
        let expected =
            vec!["ExpressionStmt GroupingExpr (BinaryExpr VariableExpr a >= VariableExpr b)"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_grouping_less() {
        let src = "(a < b);";
        let expected =
            vec!["ExpressionStmt GroupingExpr (BinaryExpr VariableExpr a < VariableExpr b)"];
        perform(src, expected)
    }

    #[test]
    fn binary_logical_variable_grouping_less_equal() {
        let src = "(a <= b);";
        let expected =
            vec!["ExpressionStmt GroupingExpr (BinaryExpr VariableExpr a <= VariableExpr b)"];
        perform(src, expected)
    }

    #[test]
    fn unary_negate() {
        let src = "-1;";
        let expected = vec!["ExpressionStmt UnaryExpr - LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn unary_negate_arithmetic() {
        let src = "-3 + 2;";
        let expected = vec!["ExpressionStmt BinaryExpr UnaryExpr - LiteralExpr Number { 3 } + LiteralExpr Number { 2 }"];
        perform(src, expected)
    }

    #[test]
    fn unary_negate_grouping() {
        let src = "-(3 + 2);";
        let expected = vec!["ExpressionStmt UnaryExpr - GroupingExpr (BinaryExpr LiteralExpr Number { 3 } + LiteralExpr Number { 2 })"];
        perform(src, expected)
    }

    #[test]
    fn unary_negate_variable() {
        let src = "-a;";
        let expected = vec!["ExpressionStmt UnaryExpr - VariableExpr a"];
        perform(src, expected)
    }

    #[test]
    fn unary_negate_variable_arithmetic() {
        let src = "-a * b;";
        let expected =
            vec!["ExpressionStmt BinaryExpr UnaryExpr - VariableExpr a * VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn unary_negate_variable_grouping() {
        let src = "-(a * b);";
        let expected = vec![
            "ExpressionStmt UnaryExpr - GroupingExpr (BinaryExpr VariableExpr a * VariableExpr b)",
        ];
        perform(src, expected)
    }

    #[test]
    fn unary_negate_fn() {
        let src = "-func();";
        let expected = vec!["ExpressionStmt UnaryExpr - CallExpr VariableExpr func"];
        perform(src, expected)
    }

    #[test]
    fn unary_negate_fn_arithmetic() {
        let src = "-func() + func_b();";
        let expected = vec!["ExpressionStmt BinaryExpr UnaryExpr - CallExpr VariableExpr func + CallExpr VariableExpr func_b"];
        perform(src, expected)
    }

    #[test]
    fn unary_negate_fn_grouping() {
        let src = "-(func() + func_b());";
        let expected = vec!["ExpressionStmt UnaryExpr - GroupingExpr (BinaryExpr CallExpr VariableExpr func + CallExpr VariableExpr func_b)"];
        perform(src, expected)
    }

    #[test]
    fn unary_not_bool() {
        let src = "!true;";
        let expected = vec!["ExpressionStmt UnaryExpr ! LiteralExpr true"];
        perform(src, expected)
    }

    #[test]
    fn unary_not_chaining() {
        let src = "!!!!true;";
        let expected = vec!["ExpressionStmt UnaryExpr ! UnaryExpr ! UnaryExpr ! UnaryExpr ! LiteralExpr true"];
        perform(src, expected)
    }

    #[test]
    fn unary_not_variable() {
        let src = "!a;";
        let expected = vec!["ExpressionStmt UnaryExpr ! VariableExpr a"];
        perform(src, expected)
    }

    #[test]
    fn unary_not_fn() {
        let src = "!a();";
        let expected = vec!["ExpressionStmt UnaryExpr ! CallExpr VariableExpr a"];
        perform(src, expected)
    }

    #[test]
    fn unary_not_binary_grouping() {
        let src = "!(1 == 2);";
        let expected = vec!["ExpressionStmt UnaryExpr ! GroupingExpr (BinaryExpr LiteralExpr Number { 1 } == LiteralExpr Number { 2 })"];
        perform(src, expected)
    }

    #[test]
    fn unary_not_binary_grouping_variale() {
        let src = "!(a == b);";
        let expected = vec![
            "ExpressionStmt UnaryExpr ! GroupingExpr (BinaryExpr VariableExpr a == VariableExpr b)",
        ];
        perform(src, expected)
    }

    #[test]
    fn unary_not_binary_grouping_fn() {
        let src = "!(a() == b());";
        let expected = vec!["ExpressionStmt UnaryExpr ! GroupingExpr (BinaryExpr CallExpr VariableExpr a == CallExpr VariableExpr b)"];
        perform(src, expected)
    }

    #[test]
    fn logical_and() {
        let src = "true and true;";
        let expected = vec!["ExpressionStmt LogicalExpr LiteralExpr true and LiteralExpr true"];
        perform(src, expected)
    }

    #[test]
    fn logical_or() {
        let src = "true or true;";
        let expected = vec!["ExpressionStmt LogicalExpr LiteralExpr true or LiteralExpr true"];
        perform(src, expected)
    }

    #[test]
    fn logical_and_variable() {
        let src = "a and b;";
        let expected = vec!["ExpressionStmt LogicalExpr VariableExpr a and VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn logical_or_variable() {
        let src = "a or b;";
        let expected = vec!["ExpressionStmt LogicalExpr VariableExpr a or VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn logical_and_fn() {
        let src = "a() and b();";
        let expected =
            vec!["ExpressionStmt LogicalExpr CallExpr VariableExpr a and CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn logical_or_fn() {
        let src = "a() or b();";
        let expected =
            vec!["ExpressionStmt LogicalExpr CallExpr VariableExpr a or CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn logical_unary_and() {
        let src = "!true and true;";
        let expected =
            vec!["ExpressionStmt LogicalExpr UnaryExpr ! LiteralExpr true and LiteralExpr true"];
        perform(src, expected)
    }

    #[test]
    fn logical_unary_or() {
        let src = "true or !true;";
        let expected =
            vec!["ExpressionStmt LogicalExpr LiteralExpr true or UnaryExpr ! LiteralExpr true"];
        perform(src, expected)
    }

    #[test]
    fn logical_unary_and_variable() {
        let src = "!a and b;";
        let expected =
            vec!["ExpressionStmt LogicalExpr UnaryExpr ! VariableExpr a and VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn logical_unary_or_variable() {
        let src = "a or !b;";
        let expected =
            vec!["ExpressionStmt LogicalExpr VariableExpr a or UnaryExpr ! VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn logical_unary_and_fn() {
        let src = "!a() and b();";
        let expected = vec!["ExpressionStmt LogicalExpr UnaryExpr ! CallExpr VariableExpr a and CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn logical_unary_or_fn() {
        let src = "a() or !b();";
        let expected = vec!["ExpressionStmt LogicalExpr CallExpr VariableExpr a or UnaryExpr ! CallExpr VariableExpr b"];
        perform(src, expected)
    }

    #[test]
    fn logical_grouping() {
        let src = "(true or false) and true;";
        let expected = vec!["ExpressionStmt LogicalExpr GroupingExpr (LogicalExpr LiteralExpr true or LiteralExpr false) and LiteralExpr true"];
        perform(src, expected)
    }

    #[test]
    fn logical_grouping_variable() {
        let src = "(a or b) and c;";
        let expected = vec!["ExpressionStmt LogicalExpr GroupingExpr (LogicalExpr VariableExpr a or VariableExpr b) and VariableExpr c"];
        perform(src, expected)
    }

    #[test]
    fn logical_grouping_fn() {
        let src = "(a() or b()) and c();";
        let expected = vec!["ExpressionStmt LogicalExpr GroupingExpr (LogicalExpr CallExpr VariableExpr a or CallExpr VariableExpr b) and CallExpr VariableExpr c"];
        perform(src, expected)
    }

    #[test]
    fn logical_grouping_unary() {
        let src = "!(true or false) and true;";
        let expected = vec!["ExpressionStmt LogicalExpr UnaryExpr ! GroupingExpr (LogicalExpr LiteralExpr true or LiteralExpr false) and LiteralExpr true"];
        perform(src, expected)
    }

    #[test]
    fn logical_grouping_unary_variable() {
        let src = "!(a or b) and c;";
        let expected = vec!["ExpressionStmt LogicalExpr UnaryExpr ! GroupingExpr (LogicalExpr VariableExpr a or VariableExpr b) and VariableExpr c"];
        perform(src, expected)
    }

    #[test]
    fn logical_grouping_unary_fn() {
        let src = "!(a() or b()) and c();";
        let expected = vec!["ExpressionStmt LogicalExpr UnaryExpr ! GroupingExpr (LogicalExpr CallExpr VariableExpr a or CallExpr VariableExpr b) and CallExpr VariableExpr c"];
        perform(src, expected)
    }

    #[test]
    fn logical_and_chaining() {
        let src = "a and b and c;";
        let expected = vec!["ExpressionStmt LogicalExpr LogicalExpr VariableExpr a and VariableExpr b and VariableExpr c"];
        perform(src, expected)
    }

    #[test]
    fn logical_or_chaining() {
        let src = "a or b or c;";
        let expected = vec!["ExpressionStmt LogicalExpr LogicalExpr VariableExpr a or VariableExpr b or VariableExpr c"];
        perform(src, expected)
    }

    #[test]
    fn logical_mix_chaining() {
        let src = "a and b or c and d;";
        let expected = vec!["ExpressionStmt LogicalExpr LogicalExpr VariableExpr a and VariableExpr b or LogicalExpr VariableExpr c and VariableExpr d"];
        perform(src, expected)
    }

    #[test]
    fn logical_complex() {
        let src = "!a() or b and (!c or d());";
        let expected = vec!["ExpressionStmt LogicalExpr UnaryExpr ! CallExpr VariableExpr a or LogicalExpr VariableExpr b and GroupingExpr (LogicalExpr UnaryExpr ! VariableExpr c or CallExpr VariableExpr d)"];
        perform(src, expected)
    }

    #[test]
    fn variable() {
        let src = "a;";
        let expected = vec!["ExpressionStmt VariableExpr a"];
        perform(src, expected)
    }

    #[test]
    fn variable_assignment() {
        let src = "a = 1;";
        let expected = vec!["ExpressionStmt AssignExpr a = LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn function_call() {
        let src = "a();";
        let expected = vec!["ExpressionStmt CallExpr VariableExpr a"];
        perform(src, expected)
    }

    #[test]
    fn compound_assign_add() {
        let src = "a += 1;";
        let expected = vec!["ExpressionStmt CompoundAssignExpr a += LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn compound_assign_sub() {
        let src = "a -= 1;";
        let expected = vec!["ExpressionStmt CompoundAssignExpr a -= LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn compound_assign_mul() {
        let src = "a *= 1;";
        let expected = vec!["ExpressionStmt CompoundAssignExpr a *= LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn compound_assign_div() {
        let src = "a /= 1;";
        let expected = vec!["ExpressionStmt CompoundAssignExpr a /= LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn lambda_fn() {
        let src = "lm() {};";
        let expected = vec!["ExpressionStmt LambdaExpr lm() {  }"];
        perform(src, expected)
    }

    #[test]
    fn lambda_fn_params() {
        let src = "lm(a, b, c) {};";
        let expected = vec!["ExpressionStmt LambdaExpr lm(a, b, c) {  }"];
        perform(src, expected)
    }

    #[test]
    fn lambda_fn_body() {
        let src = "lm(a, b, c) {
            return 1 + 2;
        };";
        let expected = vec!["ExpressionStmt LambdaExpr lm(a, b, c) { ReturnStmt BinaryExpr LiteralExpr Number { 1 } + LiteralExpr Number { 2 } }"];
        perform(src, expected)
    }

    #[test]
    fn ternary() {
        let src = "true ? true : false;";
        let expected = vec![
            "ExpressionStmt TernaryExpr LiteralExpr true ? LiteralExpr true : LiteralExpr false",
        ];
        perform(src, expected)
    }

    #[test]
    fn ternary_simple() {
        let src = "1 == 2 ? true : false;";
        let expected = vec!["ExpressionStmt TernaryExpr BinaryExpr LiteralExpr Number { 1 } == LiteralExpr Number { 2 } ? LiteralExpr true : LiteralExpr false"];
        perform(src, expected)
    }

    #[test]
    fn ternary_simple_logical() {
        let src = "a and b ? true : false;";
        let expected = vec!["ExpressionStmt TernaryExpr LogicalExpr VariableExpr a and VariableExpr b ? LiteralExpr true : LiteralExpr false"];
        perform(src, expected)
    }

    #[test]
    fn ternary_nested() {
        let src = "a and b ? a or b ? true : false : x == y ? true : false;";
        let expected = vec!["ExpressionStmt TernaryExpr LogicalExpr VariableExpr a and VariableExpr b ? TernaryExpr LogicalExpr VariableExpr a or VariableExpr b ? LiteralExpr true : LiteralExpr false : TernaryExpr BinaryExpr VariableExpr x == VariableExpr y ? LiteralExpr true : LiteralExpr false"];
        perform(src, expected)
    }

    #[test]
    fn ternary_complex() {
        let src = "a() == b ? !(x and y) : z == y;";
        let expected = vec!["ExpressionStmt TernaryExpr BinaryExpr CallExpr VariableExpr a == VariableExpr b ? UnaryExpr ! GroupingExpr (LogicalExpr VariableExpr x and VariableExpr y) : BinaryExpr VariableExpr z == VariableExpr y"];
        perform(src, expected)
    }

    #[test]
    fn print_statement() {
        let src = "print 1;";
        let expected = vec!["PrintStmt LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn variable_declaration() {
        let src = "let a = 1;";
        let expected = vec!["LetStmt a = LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn variable_declaration_uninitialized() {
        let src = "let a;";
        let expected = vec!["LetStmt a = not initialized"];
        perform(src, expected)
    }

    #[test]
    fn variable_declaration_complex() {
        let src = "let v = a() == b ? !(x and y) : z == y;";
        let expected = vec!["LetStmt v = TernaryExpr BinaryExpr CallExpr VariableExpr a == VariableExpr b ? UnaryExpr ! GroupingExpr (LogicalExpr VariableExpr x and VariableExpr y) : BinaryExpr VariableExpr z == VariableExpr y"];
        perform(src, expected)
    }

    #[test]
    fn compound_assign_invalid_target() {
        let src = "a + b += 1;";
        let expected = LoxErrorsTypes::Syntax("Invalid assignment target for".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn compound_assign_invalid() {
        let src = "+= 1;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn compound_assign_invalid_chaining() {
        let src = "a += b += 1;";
        let expected = LoxErrorsTypes::Syntax("Cannot chain compound assignment".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn compound_assign_invalid_expression() {
        let src = "a +=  += 1;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn compound_assign_semicolon() {
        let src = "a += 1";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn assignment_literal() {
        let src = "a = true;";
        let expected = vec!["ExpressionStmt AssignExpr a = LiteralExpr true"];
        perform(src, expected);
    }

    #[test]
    fn assignment_expression() {
        let src = "a = 1 + 2;";
        let expected = vec!["ExpressionStmt AssignExpr a = BinaryExpr LiteralExpr Number { 1 } + LiteralExpr Number { 2 }"];
        perform(src, expected);
    }

    #[test]
    fn assignment_equality() {
        let src = "a = 1 == 2;";
        let expected = vec!["ExpressionStmt AssignExpr a = BinaryExpr LiteralExpr Number { 1 } == LiteralExpr Number { 2 }"];
        perform(src, expected);
    }

    #[test]
    fn assignment_logical() {
        let src = "a = true and false;";
        let expected = vec!["ExpressionStmt AssignExpr a = LogicalExpr LiteralExpr true and LiteralExpr false"];
        perform(src, expected);
    }

    #[test]
    fn assignment_function_call() {
        let src = "a = b();";
        let expected = vec!["ExpressionStmt AssignExpr a = CallExpr VariableExpr b"];
        perform(src, expected);
    }

    #[test]
    fn assignment_ternary() {
        let src = "a = b and c ? true : false;";
        let expected = vec!["ExpressionStmt AssignExpr a = TernaryExpr LogicalExpr VariableExpr b and VariableExpr c ? LiteralExpr true : LiteralExpr false"];
        perform(src, expected);
    }

    #[test]
    fn assignment_lambda() {
        let src = "a = lm() {};";
        let expected = vec!["ExpressionStmt AssignExpr a = LambdaExpr lm() {  }"];
        perform(src, expected);
    }

    #[test]
    fn assignment_chain() {
        let src = "a = b = c;";
        let expected = vec!["ExpressionStmt AssignExpr a = AssignExpr b = VariableExpr c"];
        perform(src, expected);
    }

    #[test]
    fn assignment_no_expression() {
        let src = "a =;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn assignment_double_assignment() {
        let src = "a = = 2;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn assignment_semicolon() {
        let src = "a = 1";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn ternary_no_middle() {
        let src = "true ? ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn ternary_incomplete() {
        let src = "true ? true  ;";
        let expected = LoxErrorsTypes::Syntax("Incomplete ternary operation,".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn ternary_no_right() {
        let src = "true ? true : ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn ternary_no_semicolon() {
        let src = "true ? true : false";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn logical_or_no_rhs() {
        let src = "true or;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn logical_or_no_operands() {
        let src = "or;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn logical_or_invalid_chain() {
        let src = "true or or false;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }
    
    #[test]
    fn logical_or_semicolon() {
        let src = "true or false";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn logical_and_no_rhs() {
        let src = "true and;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn logical_and_no_operands() {
        let src = "and;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn logical_and_invalid_chain() {
        let src = "true and and false;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn logical_and_semicolon() {
        let src = "true and false";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn equality_no_rhs() {
        let src = "1 == ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn equality_no_operands() {
        let src = " == ;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }
    
    #[test]
    fn inequality_no_rhs() {
        let src = "1 != ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn inequality_no_operands() {
        let src = " != ;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn equality_no_semicolon() {
        let src = "1 == 1";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }
    
    #[test]
    fn inequality_no_semicolon() {
        let src = "1 != 2";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn comparison_less_no_rhs() {
        let src = "1 < ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn comparison_lessequal_no_rhs() {
        let src = "1 <= ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn comparison_greater_no_rhs() {
        let src = "1 > ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }
    
    #[test]
    fn comparison_greaterequal_no_rhs() {
        let src = "1 >= ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn comparison_less_no_operands() {
        let src = " < ;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn comparison_lessequal_no_operands() {
        let src = " <= ;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn comparison_greater_no_operands() {
        let src = " > ;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }
    
    #[test]
    fn comparison_greaterequal_no_operands() {
        let src = " >= ;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn comparison_less_no_semicolon() {
        let src = "1 < 2";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn comparison_lessequal_no_semicolon() {
        let src = "1 <= 2";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn comparison_greater_no_semicolon() {
        let src = "1 > 2";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }
    
    #[test]
    fn comparison_greaterequal_no_semicolon() {
        let src = "1 >= 2";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn binary_no_operands_add() {
        let src = "+;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn binary_no_operands_mul() {
        let src = "*;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn binary_no_operands_div() {
        let src = "/;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token".to_string());
        perform_err(src, expected);
    }

    #[test]
    fn binary_no_rhs_add() {
        let src = "1 + ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn binary_no_rhs_sub() {
        let src = "1 - ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn binary_no_rhs_mul() {
        let src = "1 * ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn binary_no_rhs_div() {
        let src = "1 / ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn binary_no_semicolon_add() {
        let src = "1 + 2";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn binary_no_semicolon_sub() {
        let src = "1 - 2";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn binary_no_semicolon_mul() {
        let src = "1 * 2";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn binary_no_semicolon_div() {
        let src = "1 / 2";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }


    #[test]
    fn unary_negate_no_rhs() {
        let src = "-;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn unary_not_no_rhs() {
        let src = "!;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn unary_negate_no_semicolon() {
        let src = "-1";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn unary_not_no_semicolon() {
        let src = "!true";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn function_call_unclosed() {
        let src = "a(b;";
        let expected = LoxErrorsTypes::Syntax("Expected ')' after".to_string());
        perform_err(src, expected)
    }
    
    #[test]
    fn function_call_no_expr() {
        let src = "a(;";
        let expected = LoxErrorsTypes::Syntax("Expected ')' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn function_call_no_semicolon() {
        let src = "a()";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn print_no_expr(){
        let src = "print ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn print_no_semicolon(){
        let src = "print a";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }
}
