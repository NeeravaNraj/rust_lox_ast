use crate::{
    error::{loxerrorhandler::LoxErrorHandler, LoxErrorsTypes, LoxResult},
    lexer::literal::*,
    lexer::token::Token,
    lexer::tokentype::TokenType,
    loxlib::string::loxstring::LoxString,
    parser::expr::{BinaryExpr, Expr, GroupingExpr, LiteralExpr, UnaryExpr},
};
use std::rc::Rc;

use super::{expr::*, stmt::*};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    curr: usize,
    error_handler: LoxErrorHandler,
    current_token: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens,
            curr: 0,
            error_handler: LoxErrorHandler::new(),
            current_token: None,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Rc<Stmt>>, LoxResult> {
        let mut statments: Vec<Rc<Stmt>> = Vec::new();
        while !self.is_at_end() {
            statments.push(self.declaration()?);
        }

        Ok(statments)
    }

    fn var_declaration(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let name = self.consume(
            TokenType::Identifier,
            LoxErrorsTypes::Syntax("Expected name for identifier".to_string()),
        )?;

        let initializer = if self.is_match(vec![TokenType::Assign]) {
            let val = self.expression()?;
            Some(val)
        } else {
            None
        };
        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expect ';' after".to_string()),
        )?;

        Ok(Rc::new(Stmt::Let(LetStmt::new(name, initializer))))
    }

    fn function(
        &mut self,
        ident: Option<Token>,
        kind: &str,
        is_static: bool,
        is_pub: bool,
    ) -> Result<Rc<Stmt>, LoxResult> {
        let name = if let Some(n) = ident {
            n
        } else {
            self.consume(
                TokenType::Identifier,
                LoxErrorsTypes::Syntax(format!("Expected {kind} name after")),
            )?
        };

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

        let body = self.block_stmt()?;

        Ok(Rc::new(Stmt::Function(FunctionStmt::new(
            name,
            Rc::new(params),
            Rc::new(body),
            is_static,
            is_pub,
        ))))
    }

    fn declaration(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let result = if self.match_single_token(TokenType::Let) {
            self.var_declaration()
        } else if self.match_single_token(TokenType::DefFn) {
            self.function(None, "function", false, false)
        } else {
            self.statement()
        };
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn block_stmt(&mut self) -> Result<Vec<Rc<Stmt>>, LoxResult> {
        let mut stmts: Vec<Rc<Stmt>> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(
            TokenType::RightBrace,
            LoxErrorsTypes::Syntax("Expected '}' after block".to_string()),
        )?;
        Ok(stmts)
    }

    fn expr_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after".to_string()),
        )?;
        Ok(Rc::new(Stmt::Expression(ExpressionStmt::new(expr))))
    }

    fn if_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        self.consume(
            TokenType::LeftParen,
            LoxErrorsTypes::Syntax("Expected '(' after".to_string()),
        )?;

        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            LoxErrorsTypes::Syntax("Expected ')' after".to_string()),
        )?;

        let then_branch = self.statement()?;
        let mut alternative: Option<Rc<Stmt>> = None;

        if self.match_single_token(TokenType::Elif) {
            alternative = Some(self.if_statement()?);
        }

        if self.match_single_token(TokenType::Else) {
            alternative = Some(self.statement()?);
        }

        Ok(Rc::new(Stmt::If(IfStmt::new(
            condition,
            then_branch,
            alternative,
        ))))
    }

    fn while_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        self.consume(
            TokenType::LeftParen,
            LoxErrorsTypes::Syntax("Expected '(' after".to_string()),
        )?;

        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            LoxErrorsTypes::Syntax("Expected ')' after".to_string()),
        )?;

        let body = self.statement()?;
        Ok(Rc::new(Stmt::While(WhileStmt::new(condition, body))))
    }

    fn for_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        self.consume(
            TokenType::LeftParen,
            LoxErrorsTypes::Syntax("Expected '(' after".to_string()),
        )?;

        let mut initializer: Option<Rc<Stmt>> = None;

        if self.peek().token_type == TokenType::Let {
            self.match_single_token(TokenType::Let);
            initializer = Some(self.var_declaration()?);
        } else if !self.check(TokenType::Semicolon) {
            initializer = Some(self.expr_statement()?);
        } else {
            self.consume(
                TokenType::Semicolon,
                LoxErrorsTypes::Syntax(
                    "Expected variable declaration or expression, got".to_string(),
                ),
            )?;
        }

        let mut condition: Option<Rc<Expr>> = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after loop condition".to_string()),
        )?;

        let mut increment: Option<Rc<Expr>> = None;
        if !self.check(TokenType::RightParen) {
            increment = Some(self.expression()?);
        }

        self.consume(
            TokenType::RightParen,
            LoxErrorsTypes::Syntax("Expected ')' after for clauses".to_string()),
        )?;

        let body = self.statement()?;

        Ok(Rc::new(Stmt::For(ForStmt::new(
            initializer,
            condition,
            increment,
            body,
        ))))
    }

    fn break_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let tok = self.previous();
        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after statement".to_string()),
        )?;
        Ok(Rc::new(Stmt::Break(BreakStmt::new(tok))))
    }

    fn continue_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let tok = self.previous();
        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after statement".to_string()),
        )?;
        Ok(Rc::new(Stmt::Continue(ContinueStmt::new(tok))))
    }

    fn return_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let keyword = self.previous();
        let mut value = Rc::new(Expr::Literal(LiteralExpr::new(Literal::None)));
        if !self.check(TokenType::Semicolon) {
            value = self.expression()?;
        }

        self.consume(
            TokenType::Semicolon,
            LoxErrorsTypes::Syntax("Expected ';' after".to_string()),
        )?;
        Ok(Rc::new(Stmt::Return(ReturnStmt::new(keyword, value))))
    }

    fn class_field(
        &mut self,
        methods: &mut Vec<Rc<Stmt>>,
        fields: &mut Vec<Rc<Stmt>>,
        is_private: bool,
        is_static: bool,
    ) -> Result<(), LoxResult> {
        let prev = self.previous();
        if self.match_single_token(TokenType::Identifier) {
            let name = self.previous();
            if name.lexeme == "init"
                && matches!(
                    prev.token_type,
                    TokenType::Private | TokenType::Public | TokenType::Static
                )
            {
                return Err(self.error_handler.error(
                    &name,
                    LoxErrorsTypes::Syntax(format!(
                        "'init' is reserved for class construction cannot make '{}'",
                        prev.lexeme
                    )),
                ));
            }
            if self.check(TokenType::LeftParen) {
                methods.push(self.function(Some(name), "method", is_static, !is_private)?);
                return Ok(());
            }

            if self.match_single_token(TokenType::Assign) {
                let value = self.expression()?;
                fields.push(Rc::new(Stmt::Field(FieldStmt::new(
                    name.dup(),
                    !is_private,
                    Some(value),
                    is_static,
                ))));
                self.consume(
                    TokenType::Semicolon,
                    LoxErrorsTypes::Syntax("Expected ';' after expression".to_string()),
                )?;
                return Ok(());
            }

            if self.match_single_token(TokenType::Semicolon) {
                fields.push(Rc::new(Stmt::Field(FieldStmt::new(
                    name.dup(),
                    !is_private,
                    None,
                    is_static,
                ))));
                return Ok(());
            }

            return Err(self.error_handler.error(
                &name,
                LoxErrorsTypes::Syntax("Expected ';' after".to_string()),
            ));
        }
        if self.check(TokenType::Static) && is_private {
            return Err(self.error_handler.error(
                self.peek(),
                LoxErrorsTypes::Syntax("Cannot make private property 'static'".to_string()),
            ));
        }

        Err(self.error_handler.error(
            &prev, 
            LoxErrorsTypes::Syntax(format!("Unexpected token '{}' after", self.peek().lexeme))
        ))
    }

    fn class_statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        let name = self.consume(
            TokenType::Identifier,
            LoxErrorsTypes::Syntax("Expected identifier for class".to_string()),
        )?;

        self.consume(
            TokenType::LeftBrace,
            LoxErrorsTypes::Syntax("Expected '{' before class body".to_string()),
        )?;

        let mut fields: Vec<Rc<Stmt>> = Vec::new();
        let mut methods: Vec<Rc<Stmt>> = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            if self.match_single_token(TokenType::Public) {
                self.class_field(&mut methods, &mut fields, false, false)?;
            } else if self.match_single_token(TokenType::Private) {
                self.class_field(&mut methods, &mut fields, true, false)?;
            } else if self.match_single_token(TokenType::Static) {
                self.class_field(&mut methods, &mut fields, false, true)?;
            } else {
                self.class_field(&mut methods, &mut fields, true, false)?;
            }
        }

        self.consume(
            TokenType::RightBrace,
            LoxErrorsTypes::Syntax("Expected '}' after class body".to_string()),
        )?;

        Ok(Rc::new(Stmt::Class(ClassStmt::new(name, fields, methods))))
    }

    fn statement(&mut self) -> Result<Rc<Stmt>, LoxResult> {
        if self.match_single_token(TokenType::LeftBrace) {
            return Ok(Rc::new(Stmt::Block(BlockStmt::new(self.block_stmt()?))));
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

        if self.match_single_token(TokenType::Class) {
            return self.class_statement();
        }

        self.expr_statement()
    }

    fn consume(&mut self, token: TokenType, error: LoxErrorsTypes) -> Result<Token, LoxResult> {
        if self.check(token) {
            return Ok(self.advance().dup());
        }

        Err(self.error_handler.error(&self.previous(), error))
    }

    fn lambda_fn(&mut self) -> Result<Rc<Expr>, LoxResult> {
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

        Ok(Rc::new(Expr::Lambda(LambdaExpr::new(
            Rc::new(params),
            Rc::new(body),
        ))))
    }

    fn array_expr(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let mut elems = Vec::new();

        if !self.check(TokenType::RightBracket) {
            elems.push(self.expression()?);
            while self.match_single_token(TokenType::Comma) {
                elems.push(self.expression()?);
            }
        }

        self.consume(
            TokenType::RightBracket,
            LoxErrorsTypes::Syntax("Expected ']' after".to_string()),
        )?;

        Ok(Rc::new(Expr::Array(ArrayExpr::new(elems))))
    }

    fn primary(&mut self) -> Result<Rc<Expr>, LoxResult> {
        if self.match_single_token(TokenType::False) {
            return Ok(Rc::new(Expr::Literal(LiteralExpr::new(Literal::Bool(
                false,
            )))));
        }

        if self.match_single_token(TokenType::True) {
            return Ok(Rc::new(Expr::Literal(LiteralExpr::new(Literal::Bool(
                true,
            )))));
        }

        if self.match_single_token(TokenType::None) {
            return Ok(Rc::new(Expr::Literal(LiteralExpr::new(Literal::None))));
        }

        if self.match_single_token(TokenType::Identifier) {
            self.current_token = Some(self.previous());
            return Ok(Rc::new(Expr::Variable(VariableExpr::new(self.previous()))));
        }

        if self.is_match(vec![TokenType::Number, TokenType::String]) {
            match self.previous().literal.as_ref().unwrap() {
                Literal::Number(literal) => {
                    return Ok(Rc::new(Expr::Literal(LiteralExpr::new(Literal::Number(
                        literal.clone(),
                    )))))
                }
                Literal::Str(literal) => {
                    return Ok(Rc::new(Expr::Literal(LiteralExpr::new(Literal::Str(
                        Rc::new(LoxString::new(literal.string.borrow().to_string())),
                    )))))
                }
                _ => {}
            }
        }

        if self.match_single_token(TokenType::DefLambda) {
            return self.lambda_fn();
        }

        if self.match_single_token(TokenType::This) {
            return Ok(Rc::new(Expr::This(ThisExpr::new(self.previous()))));
        }

        if self.match_single_token(TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(
                TokenType::RightParen,
                LoxErrorsTypes::Syntax("Expected ')' after expression, at".to_string()),
            )?;
            return Ok(Rc::new(Expr::Grouping(GroupingExpr::new(expr))));
        }

        if self.match_single_token(TokenType::LeftBracket) {
            return self.array_expr();
        }

        if self.curr == 0 {
            return Err(self.error_handler.error(
                self.peek(),
                LoxErrorsTypes::Syntax(format!("Unexpected token",)),
            ));
        }

        // panic!("ohad");
        Err(self.error_handler.error(
            &self.previous(),
            LoxErrorsTypes::Syntax("Expected expression after".to_string()),
        ))
    }

    fn postfix_operation(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let expr = self.primary()?;
        if self.is_match(vec![TokenType::PlusPlus, TokenType::MinusMinus]) {
            let op = self.previous();
            return Ok(Rc::new(Expr::Update(UpdateExpr::new(
                expr.clone(),
                op,
                false,
            ))));
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Rc<Expr>) -> Result<Rc<Expr>, LoxResult> {
        let mut args: Vec<Rc<Expr>> = Vec::new();
        if self.check(TokenType::Semicolon) {
            return Err(self.error_handler.error(
                self.peek(),
                LoxErrorsTypes::Syntax("Expected ')' after".to_string()),
            ));
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
        Ok(Rc::new(Expr::Call(CallExpr::new(callee, paren, args))))
    }

    fn call(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let mut expr = self.postfix_operation()?;

        loop {
            if self.match_single_token(TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_single_token(TokenType::Dot) {
                let name = self.consume(
                    TokenType::Identifier,
                    LoxErrorsTypes::Syntax("Expected property name after".to_string()),
                )?;
                expr = Rc::new(Expr::Get(GetExpr::new(expr, name)))
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_index(&mut self, var: Rc<Expr>) -> Result<Rc<Expr>, LoxResult> {
        let bracket = self.previous();
        let index = self.expression()?;
        self.consume(
            TokenType::RightBracket,
            LoxErrorsTypes::Syntax("Expected ']' after".to_string()),
        )?;
        Ok(Rc::new(Expr::Index(IndexExpr::new(var, bracket, index))))
    }

    fn array_subscript(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let mut expr = self.call()?;

        loop {
            if self.match_single_token(TokenType::LeftBracket) {
                expr = self.finish_index(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn prefix_operation(&mut self) -> Result<Rc<Expr>, LoxResult> {
        if self.is_match(vec![TokenType::PlusPlus, TokenType::MinusMinus]) {
            let op = self.previous();
            let var = self.primary()?;
            return Ok(Rc::new(Expr::Update(UpdateExpr::new(var, op, true))));
        }
        self.array_subscript()
    }

    fn unary(&mut self) -> Result<Rc<Expr>, LoxResult> {
        if self.is_match(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            return Ok(Rc::new(Expr::Unary(UnaryExpr::new(
                operator,
                self.unary()?,
            ))));
        }

        self.prefix_operation()
    }

    fn factor(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let mut expr = self.unary()?;

        while self.is_match(vec![TokenType::Slash, TokenType::Star, TokenType::Modulus]) {
            let operator = self.previous();
            expr = Rc::new(Expr::Binary(BinaryExpr::new(expr, operator, self.unary()?)));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let mut expr = self.factor()?;

        while self.is_match(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            expr = Rc::new(Expr::Binary(BinaryExpr::new(
                expr,
                operator,
                self.factor()?,
            )));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let mut expr = self.term()?;

        while self.is_match(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            expr = Rc::new(Expr::Binary(BinaryExpr::new(expr, operator, self.term()?)));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let mut expr = self.comparison()?;

        while self.is_match(vec![TokenType::BangEqual, TokenType::Equals]) {
            let operator = self.previous();
            expr = Rc::new(Expr::Binary(BinaryExpr::new(
                expr,
                operator,
                self.comparison()?,
            )));
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let mut expr = self.equality()?;

        while self.match_single_token(TokenType::And) {
            let op = self.previous();
            let right = self.equality()?;
            expr = Rc::new(Expr::Logical(LogicalExpr::new(expr, op, right)));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let mut expr = self.and()?;

        while self.match_single_token(TokenType::Or) {
            let op = self.previous();
            let right = self.and()?;
            expr = Rc::new(Expr::Logical(LogicalExpr::new(expr, op, right)));
        }
        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let expr = self.or()?;

        if self.match_single_token(TokenType::QuestionMark) {
            let operator = self.tokens.get(self.curr - 1).unwrap().dup();
            let middle = self.expression()?;
            if self.match_single_token(TokenType::Colon) {
                let colon = self.previous();
                return Ok(Rc::new(Expr::Ternary(TernaryExpr::new(
                    expr,
                    operator,
                    middle,
                    colon,
                    self.expression()?,
                ))));
            }
            return Err(self.error_handler.error(
                &self.previous(),
                LoxErrorsTypes::Syntax("Incomplete ternary operation,".to_string()),
            ));
        }

        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let expr = self.ternary()?;

        if self.match_single_token(TokenType::Assign) {
            let token = self.previous();
            let value = self.assignment()?;

            match &*expr {
                Expr::Variable(var) => {
                    let name = var.name.dup();
                    return Ok(Rc::new(Expr::Assign(AssignExpr::new(name, value))));
                }
                Expr::Get(prop) => {
                    return Ok(Rc::new(Expr::Set(SetExpr::new(
                        prop.object.clone(),
                        prop.name.dup(),
                        value,
                        token,
                    ))));
                }
                Expr::Index(ind) => {
                    if self.current_token.is_none() {
                        return Err(self.error_handler.error(
                            &token,
                            LoxErrorsTypes::Syntax("Unexpected token".to_string()),
                        ));
                    }
                    return Ok(Rc::new(Expr::UpdateIndex(UpdateIndexExpr::new(
                        self.current_token.as_ref().unwrap().dup(),
                        ind.identifier.clone(),
                        ind.bracket.dup(),
                        ind.index.clone(),
                        value,
                    ))));
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

    fn compound_assignment(&mut self) -> Result<Rc<Expr>, LoxResult> {
        let expr = self.assignment()?;

        if self.is_match(vec![
            TokenType::StarEqual,
            TokenType::SlashEqual,
            TokenType::PlusEqual,
            TokenType::MinusEqual,
            TokenType::ModEqual
        ]) {
            let token = self.previous();
            let value = self.primary()?;

            match self.peek().token_type {
                TokenType::SlashEqual
                | TokenType::ModEqual
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

            match &*expr {
                Expr::Variable(var) => {
                    let name = var.name.dup();
                    return Ok(Rc::new(Expr::CompoundAssign(CompoundAssignExpr::new(
                        name, token, value,
                    ))));
                }

                Expr::Get(prop) => {
                    return Ok(Rc::new(Expr::Set(SetExpr::new(
                        prop.object.clone(),
                        prop.name.dup(),
                        value,
                        token,
                    ))));
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

    fn expression(&mut self) -> Result<Rc<Expr>, LoxResult> {
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
                | TokenType::While => return,
                _ => (),
            };

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scanner;
    use std::ops::Add;
    use std::rc::Rc;

    struct AstTraverser<'a> {
        statements: &'a Vec<Rc<Stmt>>,
        strings: Vec<String>,
    }

    impl<'a> AstTraverser<'a> {
        fn new(statements: &'a Vec<Rc<Stmt>>) -> Self {
            Self {
                statements,
                strings: Vec::new(),
            }
        }

        fn gen(&mut self) -> Result<&Vec<String>, LoxResult> {
            for stmt in self.statements {
                let str = self.execute(stmt.clone())?;
                self.strings.push(str);
            }
            Ok(&self.strings)
        }

        fn evaluate(&self, expr: Rc<Expr>) -> Result<String, LoxResult> {
            expr.accept(expr.clone(), self, 0_u16)
        }

        fn execute(&self, stmt: Rc<Stmt>) -> Result<String, LoxResult> {
            stmt.accept(stmt.clone(), self, 0_u16)
        }
    }

    impl<'a> VisitorExpr<String> for AstTraverser<'a> {
        fn visit_call_expr(
            &self,
            _: Rc<Expr>,
            expr: &CallExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let callee = self.evaluate(expr.callee.clone())?;
            let mut str = format!("CallExpr {callee}");
            if expr.args.len() > 0 {
                str.push(' ');
            }
            for (i, arg) in expr.args.iter().enumerate() {
                str = str.add(&self.evaluate(arg.clone())?);
                if expr.args.len() > 1 && expr.args.len() - 1 != i {
                    str.push(' ');
                }
            }

            Ok(str)
        }

        fn visit_unary_expr(
            &self,
            _: Rc<Expr>,
            expr: &UnaryExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let op = &expr.operator.lexeme;
            let right = self.evaluate(expr.right.clone())?;
            let str = format!("UnaryExpr {op} {right}");
            Ok(str)
        }

        fn visit_binary_expr(
            &self,
            _: Rc<Expr>,
            expr: &BinaryExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let left = self.evaluate(expr.left.clone())?;
            let right = self.evaluate(expr.right.clone())?;
            let str = format!("BinaryExpr {left} {} {right}", expr.operator.lexeme);
            Ok(str)
        }

        fn visit_assign_expr(
            &self,
            _: Rc<Expr>,
            expr: &AssignExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let val = self.evaluate(expr.value.clone())?;
            Ok(format!("AssignExpr {} = {val}", expr.name.lexeme))
        }

        fn visit_lambda_expr(
            &self,
            _: Rc<Expr>,
            expr: &LambdaExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let mut params = "".to_string();
            for (i, param) in expr.params.iter().enumerate() {
                params.push_str(&param.lexeme);
                if expr.params.len() > 1 && expr.params.len() - 1 != i {
                    params.push_str(", ");
                }
            }
            let mut body = "{ ".to_string();

            for stmt in expr.body.iter() {
                body.push_str(&self.execute(stmt.clone())?);
            }

            body.push_str(" }");
            let str = format!("LambdaExpr lm({}) {}", params.trim(), body);

            Ok(str)
        }

        fn visit_logical_expr(
            &self,
            _: Rc<Expr>,
            expr: &LogicalExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let left = self.evaluate(expr.left.clone())?;
            let right = self.evaluate(expr.right.clone())?;
            let str = format!("LogicalExpr {left} {} {right}", expr.operator.lexeme);

            Ok(str)
        }

        fn visit_literal_expr(
            &self,
            _: Rc<Expr>,
            expr: &LiteralExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let str = format!("LiteralExpr {}", expr.value);
            Ok(str)
        }

        fn visit_ternary_expr(
            &self,
            _: Rc<Expr>,
            expr: &TernaryExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let left = self.evaluate(expr.left.clone())?;
            let middle = self.evaluate(expr.middle.clone())?;
            let right = self.evaluate(expr.right.clone())?;
            let str = format!("TernaryExpr {left} ? {middle} : {right}");

            Ok(str)
        }

        fn visit_grouping_expr(
            &self,
            _: Rc<Expr>,
            expr: &GroupingExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let inner = self.evaluate(expr.expression.clone())?;
            let str = format!("GroupingExpr ({inner})");

            Ok(str)
        }

        fn visit_variable_expr(
            &self,
            _: Rc<Expr>,
            expr: &VariableExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let str = format!("VariableExpr {}", expr.name.lexeme);
            Ok(str)
        }

        fn visit_compoundassign_expr(
            &self,
            _: Rc<Expr>,
            expr: &CompoundAssignExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let val = self.evaluate(expr.value.clone())?;
            let str = format!(
                "CompoundAssignExpr {} {} {val}",
                expr.name.lexeme, expr.operator.lexeme
            );
            Ok(str)
        }

        fn visit_array_expr(
            &self,
            _: Rc<Expr>,
            expr: &ArrayExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let mut str = String::from("ArrayExpr ");
            let len = expr.arr.len();
            str.push('[');
            for (i, el) in expr.arr.iter().enumerate() {
                str.push_str(&self.evaluate(el.clone())?);
                if len > 1 && len - 1 != i {
                    str.push_str(", ");
                }
            }
            str.push(']');
            Ok(str)
        }

        fn visit_index_expr(
            &self,
            _: Rc<Expr>,
            expr: &IndexExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let name = self.evaluate(expr.identifier.clone())?;
            let index = self.evaluate(expr.index.clone())?;
            let str = format!("IndexExpr {} {}", name, index);
            Ok(str)
        }

        fn visit_get_expr(&self, _: Rc<Expr>, expr: &GetExpr, _: u16) -> Result<String, LoxResult> {
            let obj = self.evaluate(expr.object.clone())?;
            Ok(format!("GetExpr {} -> {}", obj, expr.name.lexeme))
        }

        fn visit_set_expr(&self, _: Rc<Expr>, expr: &SetExpr, _: u16) -> Result<String, LoxResult> {
            let obj = self.evaluate(expr.object.clone())?;
            let val = self.evaluate(expr.value.clone())?;
            Ok(format!("SetExpr {} -> {} = {}", obj, expr.name.lexeme, val))
        }

        fn visit_this_expr(&self, _: Rc<Expr>, _: &ThisExpr, _: u16) -> Result<String, LoxResult> {
            Ok("ThisExpr".to_string())
        }

        fn visit_update_expr(
            &self,
            _: Rc<Expr>,
            expr: &UpdateExpr,
            _: u16,
        ) -> Result<String, LoxResult> {
            let var = self.evaluate(expr.var.clone())?;
            if expr.prefix {
                return Ok(format!("UpdateExpr {} {}", expr.operator.lexeme, var));
            }
            Ok(format!("UpdateExpr {1} {0}", expr.operator.lexeme, var))
        }

        fn visit_updateindex_expr(&self, _: Rc<Expr>, expr: &UpdateIndexExpr, _: u16) -> Result<String, LoxResult> {
            let val = self.evaluate(expr.value.clone())?;
            let index = self.evaluate(expr.index.clone())?;
            let ident = self.evaluate(expr.identifier.clone())?;

            Ok(format!("UpdateIndex {ident}[{index} = {val}]"))
        }
    }

    impl<'a> VisitorStmt<String> for AstTraverser<'a> {
        fn visit_if_stmt(&self, _: Rc<Stmt>, stmt: &IfStmt, _: u16) -> Result<String, LoxResult> {
            let cond = self.evaluate(stmt.condition.clone())?;
            let body = self.execute(stmt.then_branch.clone())?;
            let mut str = format!("IfStmt ({}) ", cond);

            str.push_str(&body);
            if stmt.else_branch.is_some() {
                let else_branch = self.execute(stmt.else_branch.as_ref().unwrap().clone())?;
                str.push_str(" else ");
                str.push_str(&else_branch);
            }
            Ok(str)
        }

        fn visit_let_stmt(&self, _: Rc<Stmt>, stmt: &LetStmt, _: u16) -> Result<String, LoxResult> {
            let mut initializer = "not initialized".to_string();
            if stmt.initializer.is_some() {
                initializer = self.evaluate(stmt.initializer.as_ref().unwrap().clone())?;
            }
            let str = format!("LetStmt {} = {}", stmt.name.lexeme, initializer);
            Ok(str)
        }

        fn visit_for_stmt(&self, _: Rc<Stmt>, stmt: &ForStmt, _: u16) -> Result<String, LoxResult> {
            let var_decl = if let Some(s) = &stmt.var {
                self.execute(s.clone())?
            } else {
                "".to_string()
            };

            let condition = if let Some(c) = &stmt.condition {
                self.evaluate(c.clone())?
            } else {
                "".to_string()
            };

            let update = if let Some(u) = &stmt.update_expr {
                self.evaluate(u.clone())?
            } else {
                "".to_string()
            };

            let body = self.execute(stmt.body.clone())?;
            let str = format!("ForStmt ({};{};{}) {}", var_decl, condition, update, body);

            Ok(str)
        }

        fn visit_block_stmt(
            &self,
            _: Rc<Stmt>,
            stmt: &BlockStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            let mut str = format!("BlockStmt");

            str.push_str(" { ");
            for (i, val) in stmt.statements.iter().enumerate() {
                let line = self.execute(val.clone())?;
                str.push_str(&line);
                if stmt.statements.len() > 1 && stmt.statements.len() - 1 != i {
                    str.push(' ');
                }
            }
            str.push_str(" }");
            Ok(str)
        }

        fn visit_while_stmt(
            &self,
            _: Rc<Stmt>,
            stmt: &WhileStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            let condition = self.evaluate(stmt.condition.clone())?;
            let body = self.execute(stmt.body.clone())?;
            let str = format!("WhileStmt ({}) {}", condition, body);
            Ok(str)
        }

        fn visit_break_stmt(
            &self,
            _: Rc<Stmt>,
            _: &BreakStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            let str = format!("BreakStmt");
            Ok(str)
        }

        fn visit_return_stmt(
            &self,
            _: Rc<Stmt>,
            stmt: &ReturnStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            let val = self.evaluate(stmt.value.clone())?;
            let str = format!("ReturnStmt {val}");

            Ok(str)
        }

        fn visit_continue_stmt(
            &self,
            _: Rc<Stmt>,
            _: &ContinueStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            let str = format!("ContinueStmt");
            Ok(str)
        }

        fn visit_function_stmt(
            &self,
            _: Rc<Stmt>,
            stmt: &FunctionStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            let mut params = "".to_string();
            for (i, param) in stmt.params.iter().enumerate() {
                params.push_str(&param.lexeme);
                if stmt.params.len() - 1 != i {
                    params.push_str(", ");
                }
            }
            let mut body = "{ ".to_string();

            for (i, val) in stmt.body.iter().enumerate() {
                body.push_str(&self.execute(val.clone())?);
                if stmt.body.len() > 1 && stmt.body.len() - 1 != i {
                    body.push(' ');
                }
            }

            body.push_str(" }");
            let str = format!(
                "FunctionStmt {}({}) {}",
                stmt.name.lexeme,
                params.trim(),
                body
            );

            Ok(str)
        }

        fn visit_expression_stmt(
            &self,
            _: Rc<Stmt>,
            stmt: &ExpressionStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            let expr = self.evaluate(stmt.expr.clone())?;
            let str = format!("ExpressionStmt {expr}");
            Ok(str)
        }

        fn visit_class_stmt(
            &self,
            _: Rc<Stmt>,
            stmt: &ClassStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            let mut methods = "".to_string();
            for s in stmt.methods.iter() {
                methods.push_str(self.execute(s.clone())?.as_str());
            }
            Ok(format!("ClassStmt {} {{ {} }}", stmt.name.lexeme, methods))
        }

        fn visit_field_stmt(
            &self,
            _: Rc<Stmt>,
            stmt: &FieldStmt,
            _: u16,
        ) -> Result<String, LoxResult> {
            if stmt.is_pub {
                return Ok(format!("FieldStmt public {}", stmt.name.lexeme));
            }

            Ok(format!("FieldStmt private {}", stmt.name.lexeme))
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
        let expected =
            vec!["ExpressionStmt UnaryExpr ! UnaryExpr ! UnaryExpr ! UnaryExpr ! LiteralExpr true"];
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
        let expected = vec![
            "ExpressionStmt AssignExpr a = LogicalExpr LiteralExpr true and LiteralExpr false",
        ];
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
    fn function_call_calling_call() {
        let src = "a()();";
        let expected = vec!["ExpressionStmt CallExpr CallExpr VariableExpr a"];
        perform(src, expected);
    }

    #[test]
    fn function_call_no_semicolon() {
        let src = "a()";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn if_statements() {
        let src = "if (x > 1) {
            y = 5;
        }";
        let expected = vec!["IfStmt (BinaryExpr VariableExpr x > LiteralExpr Number { 1 }) BlockStmt { ExpressionStmt AssignExpr y = LiteralExpr Number { 5 } }"];
        perform(src, expected)
    }

    #[test]
    fn if_statements_nested_block() {
        let src = "if (x > 1) {
            {}
        }";
        let expected = vec!["IfStmt (BinaryExpr VariableExpr x > LiteralExpr Number { 1 }) BlockStmt { BlockStmt {  } }"];
        perform(src, expected)
    }

    #[test]
    fn if_else_statements() {
        let src = "if (x > 1) {
            y = 5;
        } else {
            y = x;
        }";
        let expected = vec!["IfStmt (BinaryExpr VariableExpr x > LiteralExpr Number { 1 }) BlockStmt { ExpressionStmt AssignExpr y = LiteralExpr Number { 5 } } else BlockStmt { ExpressionStmt AssignExpr y = VariableExpr x }"];
        perform(src, expected)
    }

    #[test]
    fn if_elif_statements() {
        let src = "if (x > 1) {
            y = 5;
        } elif (x < 1) {
            y = x;
        }";
        let expected = vec!["IfStmt (BinaryExpr VariableExpr x > LiteralExpr Number { 1 }) BlockStmt { ExpressionStmt AssignExpr y = LiteralExpr Number { 5 } } else IfStmt (BinaryExpr VariableExpr x < LiteralExpr Number { 1 }) BlockStmt { ExpressionStmt AssignExpr y = VariableExpr x }"];
        perform(src, expected)
    }

    #[test]
    fn if_elif_else_statements() {
        let src = "if (x > 1) {
            y = 5;
        } elif (x < 1) {
            y = x;
        } else {
            y = 0;
        }";
        let expected = vec!["IfStmt (BinaryExpr VariableExpr x > LiteralExpr Number { 1 }) BlockStmt { ExpressionStmt AssignExpr y = LiteralExpr Number { 5 } } else IfStmt (BinaryExpr VariableExpr x < LiteralExpr Number { 1 }) BlockStmt { ExpressionStmt AssignExpr y = VariableExpr x } else BlockStmt { ExpressionStmt AssignExpr y = LiteralExpr Number { 0 } }"];
        perform(src, expected)
    }

    #[test]
    fn if_statements_nesting() {
        let src = "if (x > 1) {
            if (x < 1) {
                y = 5;
            }
        }";
        let expected = vec!["IfStmt (BinaryExpr VariableExpr x > LiteralExpr Number { 1 }) BlockStmt { IfStmt (BinaryExpr VariableExpr x < LiteralExpr Number { 1 }) BlockStmt { ExpressionStmt AssignExpr y = LiteralExpr Number { 5 } } }"];
        perform(src, expected)
    }

    #[test]
    fn if_statements_multiple_conditions() {
        let src = "if (x > 1 and x < -5) {
            y = 5;
        }";
        let expected = vec!["IfStmt (LogicalExpr BinaryExpr VariableExpr x > LiteralExpr Number { 1 } and BinaryExpr VariableExpr x < UnaryExpr - LiteralExpr Number { 5 }) BlockStmt { ExpressionStmt AssignExpr y = LiteralExpr Number { 5 } }"];
        perform(src, expected)
    }

    #[test]
    fn if_statements_binary_operation_in_condition() {
        let src = "if (x + 1 == 2) {
            y = true;
        }";
        let expected = vec!["IfStmt (BinaryExpr BinaryExpr VariableExpr x + LiteralExpr Number { 1 } == LiteralExpr Number { 2 }) BlockStmt { ExpressionStmt AssignExpr y = LiteralExpr true }"];
        perform(src, expected)
    }

    #[test]
    fn if_statements_complex_condition() {
        let src = "if ((x - 2) * y < 100) {
            y += 1;
        }";
        let expected = vec!["IfStmt (BinaryExpr BinaryExpr GroupingExpr (BinaryExpr VariableExpr x - LiteralExpr Number { 2 }) * VariableExpr y < LiteralExpr Number { 100 }) BlockStmt { ExpressionStmt CompoundAssignExpr y += LiteralExpr Number { 1 } }"];
        perform(src, expected)
    }

    #[test]
    fn if_statements_inline_stmt() {
        let src = "if ((x - 2) * y < 100) print(y + 1);";
        let expected = vec!["IfStmt (BinaryExpr BinaryExpr GroupingExpr (BinaryExpr VariableExpr x - LiteralExpr Number { 2 }) * VariableExpr y < LiteralExpr Number { 100 }) ExpressionStmt CallExpr VariableExpr print BinaryExpr VariableExpr y + LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn if_else_statements_inline_stmt() {
        let src = "
            if ((x - 2) * y < 100) print(y + 1);
            else print(y);
        ";
        let expected = vec!["IfStmt (BinaryExpr BinaryExpr GroupingExpr (BinaryExpr VariableExpr x - LiteralExpr Number { 2 }) * VariableExpr y < LiteralExpr Number { 100 }) ExpressionStmt CallExpr VariableExpr print BinaryExpr VariableExpr y + LiteralExpr Number { 1 } else ExpressionStmt CallExpr VariableExpr print VariableExpr y"];
        perform(src, expected)
    }

    #[test]
    fn if_elif_else_statements_inline_stmt() {
        let src = "
            if ((x - 2) * y < 100) print(y + 1);
            elif (x == 100) print(y - 1);
            else print(y);
        ";
        let expected = vec!["IfStmt (BinaryExpr BinaryExpr GroupingExpr (BinaryExpr VariableExpr x - LiteralExpr Number { 2 }) * VariableExpr y < LiteralExpr Number { 100 }) ExpressionStmt CallExpr VariableExpr print BinaryExpr VariableExpr y + LiteralExpr Number { 1 } else IfStmt (BinaryExpr VariableExpr x == LiteralExpr Number { 100 }) ExpressionStmt CallExpr VariableExpr print BinaryExpr VariableExpr y - LiteralExpr Number { 1 } else ExpressionStmt CallExpr VariableExpr print VariableExpr y"];
        perform(src, expected)
    }

    #[test]
    fn if_statements_no_condition() {
        let src = "
            if () print y + 1;
        ";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn if_statements_no_then() {
        let src = "if (x == 2)";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn if_statements_condition_block_not_started() {
        let src = "if";
        let expected = LoxErrorsTypes::Syntax("Expected '(' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn if_statements_condition_block_not_closed() {
        let src = "if (x == 2";
        let expected = LoxErrorsTypes::Syntax("Expected ')' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn elif_statements_condition_block_not_started() {
        let src = "
            if (x == 2) {}
            elif
        ";
        let expected = LoxErrorsTypes::Syntax("Expected '(' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn elif_statements_condition_block_not_closed() {
        let src = "
            if (x == 2){}
            elif (x == 3
        ";
        let expected = LoxErrorsTypes::Syntax("Expected ')' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn else_statements_no_then() {
        let src = "
            if (x == 2) {}
            else
        ";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn elif_statements_no_then() {
        let src = "
            if (x == 2) {}
            elif (x == 2)
        ";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn if_statements_unclosed_block() {
        let src = "if (x == 2) {";
        let expected = LoxErrorsTypes::Syntax("Expected '}' after block".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn for_statement() {
        let src = "for(let i = 0; i < 10; i += 1) {}";
        let expected = vec!["ForStmt (LetStmt i = LiteralExpr Number { 0 };BinaryExpr VariableExpr i < LiteralExpr Number { 10 };CompoundAssignExpr i += LiteralExpr Number { 1 }) BlockStmt {  }"];
        perform(src, expected)
    }

    #[test]
    fn for_statement_simple_statement() {
        let src = "for(let i = 0; i < 10; i += 1) print(i);";
        let expected = vec!["ForStmt (LetStmt i = LiteralExpr Number { 0 };BinaryExpr VariableExpr i < LiteralExpr Number { 10 };CompoundAssignExpr i += LiteralExpr Number { 1 }) ExpressionStmt CallExpr VariableExpr print VariableExpr i"];
        perform(src, expected)
    }

    #[test]
    fn for_statement_nested_block() {
        let src = "for(let i = 0; i < 10; i += 1) { {} }";
        let expected = vec!["ForStmt (LetStmt i = LiteralExpr Number { 0 };BinaryExpr VariableExpr i < LiteralExpr Number { 10 };CompoundAssignExpr i += LiteralExpr Number { 1 }) BlockStmt { BlockStmt {  } }"];
        perform(src, expected)
    }

    #[test]
    fn for_statement_no_var() {
        let src = "for(; i < 10; i += 1) {}";
        let expected = vec!["ForStmt (;BinaryExpr VariableExpr i < LiteralExpr Number { 10 };CompoundAssignExpr i += LiteralExpr Number { 1 }) BlockStmt {  }"];
        perform(src, expected)
    }

    #[test]
    fn for_statement_no_condition() {
        let src = "for(let i = 0;; i += 1) {}";
        let expected = vec!["ForStmt (LetStmt i = LiteralExpr Number { 0 };;CompoundAssignExpr i += LiteralExpr Number { 1 }) BlockStmt {  }"];
        perform(src, expected)
    }

    #[test]
    fn for_statement_no_update() {
        let src = "for(let i = 0; i < 10;) {}";
        let expected = vec!["ForStmt (LetStmt i = LiteralExpr Number { 0 };BinaryExpr VariableExpr i < LiteralExpr Number { 10 };) BlockStmt {  }"];
        perform(src, expected)
    }

    #[test]
    fn for_statement_no_var_condition_update() {
        let src = "for(;;) {}";
        let expected = vec!["ForStmt (;;) BlockStmt {  }"];
        perform(src, expected)
    }

    #[test]
    fn for_statement_continue() {
        let src = "for(;;) { continue; }";
        let expected = vec!["ForStmt (;;) BlockStmt { ContinueStmt }"];
        perform(src, expected)
    }

    #[test]
    fn for_statement_break() {
        let src = "for(;;) { break; }";
        let expected = vec!["ForStmt (;;) BlockStmt { BreakStmt }"];
        perform(src, expected)
    }

    #[test]
    fn for_statement_no_condition_block() {
        let src = "for";
        let expected = LoxErrorsTypes::Syntax("Expected '(' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn for_statement_unclosed_condition() {
        let src = "for (;;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn for_statement_unclosed_condition_second() {
        let src = "for (;; i += 1";
        let expected = LoxErrorsTypes::Syntax("Expected ')' after for clauses".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn for_statement_no_block() {
        let src = "for (;;)";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn for_statement_unclosed_block() {
        let src = "for (;;) {";
        let expected = LoxErrorsTypes::Syntax("Expected '}' after block".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn while_statement() {
        let src = "while (i < 10) {}";
        let expected = vec![
            "WhileStmt (BinaryExpr VariableExpr i < LiteralExpr Number { 10 }) BlockStmt {  }",
        ];
        perform(src, expected)
    }

    #[test]
    fn while_statement_continue() {
        let src = "while (i < 10) {
            continue;
        }";
        let expected = vec!["WhileStmt (BinaryExpr VariableExpr i < LiteralExpr Number { 10 }) BlockStmt { ContinueStmt }"];
        perform(src, expected)
    }

    #[test]
    fn while_statement_break() {
        let src = "while (i < 10) {
            break;
        }";
        let expected = vec!["WhileStmt (BinaryExpr VariableExpr i < LiteralExpr Number { 10 }) BlockStmt { BreakStmt }"];
        perform(src, expected)
    }

    #[test]
    fn while_statement_simple_statement() {
        let src = "while (i < 10) print(i);";
        let expected = vec!["WhileStmt (BinaryExpr VariableExpr i < LiteralExpr Number { 10 }) ExpressionStmt CallExpr VariableExpr print VariableExpr i"];
        perform(src, expected)
    }

    #[test]
    fn while_statement_nested_block() {
        let src = "while (i < 10) { {} }";
        let expected = vec!["WhileStmt (BinaryExpr VariableExpr i < LiteralExpr Number { 10 }) BlockStmt { BlockStmt {  } }"];
        perform(src, expected)
    }

    #[test]
    fn while_statement_complex_condition() {
        let src = "while ((x - 1) * y > 12) {}";
        let expected = vec!["WhileStmt (BinaryExpr BinaryExpr GroupingExpr (BinaryExpr VariableExpr x - LiteralExpr Number { 1 }) * VariableExpr y > LiteralExpr Number { 12 }) BlockStmt {  }"];
        perform(src, expected)
    }

    #[test]
    fn while_statement_no_condition_block() {
        let src = "while ";
        let expected = LoxErrorsTypes::Syntax("Expected '(' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn while_statement_no_block() {
        let src = "while (x < 10)";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn while_statement_unclosed_condition() {
        let src = "while (x < 10";
        let expected = LoxErrorsTypes::Syntax("Expected ')' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn while_statement_unclosed_block() {
        let src = "while (x < 10) {";
        let expected = LoxErrorsTypes::Syntax("Expected '}' after block".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn fn_statement() {
        let src = "fn test() {}";
        let expected = vec!["FunctionStmt test() {  }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_params() {
        let src = "fn test(a, b, c) {}";
        let expected = vec!["FunctionStmt test(a, b, c) {  }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_body() {
        let src = "fn test() {
            let a = 1;
        }";
        let expected = vec!["FunctionStmt test() { LetStmt a = LiteralExpr Number { 1 } }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_nested_block() {
        let src = "fn test() {
            {

            }
        }";
        let expected = vec!["FunctionStmt test() { BlockStmt {  } }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_return_statement() {
        let src = "fn test(a, b) {
            return a + b;
        }";
        let expected = vec![
            "FunctionStmt test(a, b) { ReturnStmt BinaryExpr VariableExpr a + VariableExpr b }",
        ];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_variable() {
        let src = "fn test(a, b) {
            let result = a + b;
            return result;
        }";
        let expected = vec!["FunctionStmt test(a, b) { LetStmt result = BinaryExpr VariableExpr a + VariableExpr b ReturnStmt VariableExpr result }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_print() {
        let src = "fn test(a, b) {
            print(a + b);
        }";
        let expected = vec![
            "FunctionStmt test(a, b) { ExpressionStmt CallExpr VariableExpr print BinaryExpr VariableExpr a + VariableExpr b }",
        ];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_nested_fn() {
        let src = "fn test(a, b) {
            fn nested() {
                return a;
            }
        }";
        let expected =
            vec!["FunctionStmt test(a, b) { FunctionStmt nested() { ReturnStmt VariableExpr a } }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_return_lambda() {
        let src = "fn test(a, b) {
            return lm() {
                print(a + b);
            };
        }";
        let expected = vec!["FunctionStmt test(a, b) { ReturnStmt LambdaExpr lm() { ExpressionStmt CallExpr VariableExpr print BinaryExpr VariableExpr a + VariableExpr b } }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_recurse() {
        let src = "fn test(a, b) {
            test(1, 2);
        }";
        let expected = vec!["FunctionStmt test(a, b) { ExpressionStmt CallExpr VariableExpr test LiteralExpr Number { 1 } LiteralExpr Number { 2 } }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_return_call() {
        let src = "fn test(a, b) {
            return test(1, 2);
        }";
        let expected = vec!["FunctionStmt test(a, b) { ReturnStmt CallExpr VariableExpr test LiteralExpr Number { 1 } LiteralExpr Number { 2 } }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_return_from_if() {
        let src = "fn test(a, b) {
            if (a > b) return a;
            elif (a < b) return b;
            else return -1;
        }";
        let expected = vec!["FunctionStmt test(a, b) { IfStmt (BinaryExpr VariableExpr a > VariableExpr b) ReturnStmt VariableExpr a else IfStmt (BinaryExpr VariableExpr a < VariableExpr b) ReturnStmt VariableExpr b else ReturnStmt UnaryExpr - LiteralExpr Number { 1 } }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_return_from_while() {
        let src = "fn test(a, b) {
            while (a < 10) {
                return a;
            }
        }";
        let expected = vec!["FunctionStmt test(a, b) { WhileStmt (BinaryExpr VariableExpr a < LiteralExpr Number { 10 }) BlockStmt { ReturnStmt VariableExpr a } }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_return_from_for() {
        let src = "fn test(a, b) {
            for (let i = 0; i < 10; i += 1) {
                return a;
            }
        }";
        let expected = vec!["FunctionStmt test(a, b) { ForStmt (LetStmt i = LiteralExpr Number { 0 };BinaryExpr VariableExpr i < LiteralExpr Number { 10 };CompoundAssignExpr i += LiteralExpr Number { 1 }) BlockStmt { ReturnStmt VariableExpr a } }"];
        perform(src, expected)
    }

    #[test]
    fn fn_statement_no_name() {
        let src = "fn";
        let expected = LoxErrorsTypes::Syntax("Expected function name after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn fn_statement_no_param_block() {
        let src = "fn test";
        let expected = LoxErrorsTypes::Syntax("Expected '(' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn fn_statement_unclosed_params_block() {
        let src = "fn test(";
        let expected = LoxErrorsTypes::Syntax("Expected parameter identifier".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn fn_statement_unclosed_params_block_2() {
        let src = "fn test(a";
        let expected = LoxErrorsTypes::Syntax("Expected ')' after parameters".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn fn_statement_trailing_comma() {
        let src = "fn test(a,) {}";
        let expected = LoxErrorsTypes::Syntax("Expected parameter identifier".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn fn_statement_no_body() {
        let src = "fn test(a)";
        let expected = LoxErrorsTypes::Syntax("Expected '{' before function body".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn fn_statement_unclosed_body() {
        let src = "fn test(a) {";
        let expected = LoxErrorsTypes::Syntax("Expected '}' after block".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn array_expr() {
        let src = "[1, 2, 3];";
        let expected = vec!["ExpressionStmt ArrayExpr [LiteralExpr Number { 1 }, LiteralExpr Number { 2 }, LiteralExpr Number { 3 }]"];
        perform(src, expected)
    }

    #[test]
    fn array_expr_multi_type() {
        let src = "[1, \"str\", true, none];";
        let expected = vec!["ExpressionStmt ArrayExpr [LiteralExpr Number { 1 }, LiteralExpr String { \"str\" }, LiteralExpr true, LiteralExpr none]"];
        perform(src, expected)
    }

    #[test]
    fn array_expr_nested() {
        let src = "[
            [1, 2, 3],
            [true, false, none],
            [\"Str1\", \"Str2\", \"Str3\"]
        ];";
        let expected = vec!["ExpressionStmt ArrayExpr [ArrayExpr [LiteralExpr Number { 1 }, LiteralExpr Number { 2 }, LiteralExpr Number { 3 }], ArrayExpr [LiteralExpr true, LiteralExpr false, LiteralExpr none], ArrayExpr [LiteralExpr String { \"Str1\" }, LiteralExpr String { \"Str2\" }, LiteralExpr String { \"Str3\" }]]"];
        perform(src, expected)
    }

    #[test]
    fn array_expr_expression_as_elements() {
        let src = "[a, 1 + 2, a > 2, lm(){ print(123); }];";
        let expected = vec!["ExpressionStmt ArrayExpr [VariableExpr a, BinaryExpr LiteralExpr Number { 1 } + LiteralExpr Number { 2 }, BinaryExpr VariableExpr a > LiteralExpr Number { 2 }, LambdaExpr lm() { ExpressionStmt CallExpr VariableExpr print LiteralExpr Number { 123 } }]"];
        perform(src, expected)
    }

    #[test]
    fn array_expr_trailing_comma() {
        let src = "[1, 2,];";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn array_expr_unclosed_array() {
        let src = "[1, 2;";
        let expected = LoxErrorsTypes::Syntax("Expected ']' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn array_expr_no_semicolon() {
        let src = "[1, 2]";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn index_expr() {
        let src = "a[0];";
        let expected = vec!["ExpressionStmt IndexExpr VariableExpr a LiteralExpr Number { 0 }"];
        perform(src, expected)
    }

    #[test]
    fn index_expr_double_index() {
        let src = "a[0][0];";
        let expected = vec!["ExpressionStmt IndexExpr IndexExpr VariableExpr a LiteralExpr Number { 0 } LiteralExpr Number { 0 }"];
        perform(src, expected)
    }

    #[test]
    fn index_expr_call() {
        let src = "a()[0];";
        let expected =
            vec!["ExpressionStmt IndexExpr CallExpr VariableExpr a LiteralExpr Number { 0 }"];
        perform(src, expected)
    }

    #[test]
    fn index_expr_arithmetic() {
        let src = "a[1 + 2];";
        let expected = vec!["ExpressionStmt IndexExpr VariableExpr a BinaryExpr LiteralExpr Number { 1 } + LiteralExpr Number { 2 }"];
        perform(src, expected)
    }

    #[test]
    fn index_expr_arithmetic_call() {
        let src = "a[b() - 1];";
        let expected = vec!["ExpressionStmt IndexExpr VariableExpr a BinaryExpr CallExpr VariableExpr b - LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn index_expr_no_expr() {
        let src = "a[;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn index_expr_no_expr_unclosed() {
        let src = "a[;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn index_expr_unclosed() {
        let src = "a[0;";
        let expected = LoxErrorsTypes::Syntax("Expected ']' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn update_expr_prefix() {
        let src = "++a;";
        let expected = vec!["ExpressionStmt UpdateExpr ++ VariableExpr a"];
        perform(src, expected)
    }

    #[test]
    fn update_expr_postfix() {
        let src = "a--;";
        let expected = vec!["ExpressionStmt UpdateExpr VariableExpr a --"];
        perform(src, expected)
    }

    #[test]
    fn update_expr_no_semicolon() {
        let src = "++a";
        let expected = LoxErrorsTypes::Syntax("Expected ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn get_expr() {
        let src = "A.b;";
        let expected = vec!["ExpressionStmt GetExpr VariableExpr A -> b"];
        perform(src, expected)
    }

    #[test]
    fn get_expr_call() {
        let src = "A.b();";
        let expected = vec!["ExpressionStmt CallExpr GetExpr VariableExpr A -> b"];
        perform(src, expected)
    }

    #[test]
    fn get_expr_chaining() {
        let src = "A.b.c.d();";
        let expected =
            vec!["ExpressionStmt CallExpr GetExpr GetExpr GetExpr VariableExpr A -> b -> c -> d"];
        perform(src, expected)
    }

    #[test]
    fn get_expr_no_rhs() {
        let src = "A.;";
        let expected = LoxErrorsTypes::Syntax("Expected property name after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn set_expr() {
        let src = "a.b = 1;";
        let expected =
            vec!["ExpressionStmt SetExpr VariableExpr a -> b = LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn set_expr_chaining() {
        let src = "a.b.c.d = 1;";
        let expected = vec!["ExpressionStmt SetExpr GetExpr GetExpr VariableExpr a -> b -> c -> d = LiteralExpr Number { 1 }"];
        perform(src, expected)
    }

    #[test]
    fn set_expr_call() {
        let src = "a.b() = 1;";
        let expected = LoxErrorsTypes::Parse("Invalid assignment target".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn set_expr_no_value() {
        let src = "a.b() = ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn set_expr_no_rhs() {
        let src = "a. = 1;";
        let expected = LoxErrorsTypes::Syntax("Expected property name after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn class_decl() {
        let src = "class Point {}";
        let expected = vec!["ClassStmt Point {  }"];
        perform(src, expected)
    }

    #[test]
    fn class_decl_methods() {
        let src = "class Point { init() {} }";
        let expected = vec!["ClassStmt Point { FunctionStmt init() {  } }"];
        perform(src, expected)
    }

    #[test]
    fn class_decl_methods_this() {
        let src = "class Point { init(x) { this.x = x; } }";
        let expected = vec!["ClassStmt Point { FunctionStmt init(x) { ExpressionStmt SetExpr ThisExpr -> x = VariableExpr x } }"];
        perform(src, expected)
    }

    #[test]
    fn class_decl_no_name() {
        let src = "class;";
        let expected = LoxErrorsTypes::Syntax("Expected identifier for class".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn class_decl_no_brace() {
        let src = "class Point;";
        let expected = LoxErrorsTypes::Syntax("Expected '{' before class body".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn class_decl_unclosed_body() {
        let src = "class Point {;";
        let expected = LoxErrorsTypes::Syntax("Unexpected token ';' after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn class_decl_method_no_parens() {
        let src = "class Point { init;";
        let expected = LoxErrorsTypes::Syntax("Expected '}' after class body".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn class_decl_method_unclosed_paren_no_params() {
        let src = "class Point { init(;";
        let expected = LoxErrorsTypes::Syntax("Expected parameter identifier".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn class_decl_method_unclosed_paren() {
        let src = "class Point { init(a;";
        let expected = LoxErrorsTypes::Syntax("Expected ')' after parameters".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn class_decl_method_no_brace() {
        let src = "class Point { init(a) ;";
        let expected = LoxErrorsTypes::Syntax("Expected '{' before method body".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn class_decl_method_unclosed_body() {
        let src = "class Point { init(a) { ;";
        let expected = LoxErrorsTypes::Syntax("Expected expression after".to_string());
        perform_err(src, expected)
    }

    #[test]
    fn class_decl_method_unclosed_body_statement() {
        let src = "class Point { init(a) { let a = 0;";
        let expected = LoxErrorsTypes::Syntax("Expected '}' after block".to_string());
        perform_err(src, expected)
    }
}
