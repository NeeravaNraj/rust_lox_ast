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
        let mut else_branch: Option<Box<Stmt>> = None;

        if self.match_single_token(TokenType::Else) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If(IfStmt::new(condition, then_branch, else_branch)))
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
                        LoxErrorsTypes::Syntax("Can't have more than 255 parameters".to_string())
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

        if self.match_single_token(TokenType::DefFn) {
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
                LoxErrorsTypes::Syntax(format!(
                    "leading '{}' is not supported",
                    self.peek().lexeme
                )),
            ));
        }

        Err(self.error_handler.error(
            &self.previous(),
            LoxErrorsTypes::Syntax("Expected expression after".to_string()),
        ))
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, LoxResult> {
        let mut args: Vec<Expr> = Vec::new();
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
            let right = self.equality()?;
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
            let value = self.compound_assignment()?;

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
                        LoxErrorsTypes::Parse("Invalid assignment target".to_string()),
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
