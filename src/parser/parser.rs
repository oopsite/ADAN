/*
MIT License

Copyright (c) 2025 Cappucina

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

// The ADAN Parser
// Authored by: @oopsite
// Dated Nov 4, 2025

#[allow(unused_variables)]

use crate::lexer::token::*;
use crate::parser::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn match_keyword(&mut self, kw: Keyword) -> bool {
        if matches!(self.peek(), Some(Token::Keyword(k)) if *k == kw) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn match_symbol(&mut self, sym: Symbols) -> bool {
        if matches!(self.peek(), Some(Token::Symbols(s)) if *s == sym) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_symbol(&mut self, sym: Symbols) -> Result<(), String> {
        if self.match_symbol(sym) {
            Ok(())
        } else {
            Err(format!("Expected symbol {:?}", sym))
        }
    }

    fn expect_keyword(&mut self, kw: Keyword) -> Result<(), String> {
        if self.match_keyword(kw) {
            Ok(())
        } else {
            Err(format!("Expected keyword {:?}", kw))
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Ident(id)) => Ok(id.clone()),
            other => Err(format!("Expected identifier, got {:?}", other)),
        }
    }

    // ------------------------
    // Top-level parsing
    // ------------------------
    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = vec![];
        while self.peek().is_some() {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    // ------------------------
    // Statements
    // ------------------------
    fn parse_statement(&mut self) -> Result<Statement, String> {
        if self.match_keyword(Keyword::Local) || self.match_keyword(Keyword::Global) {
            return self.parse_var_decl();
        }
        if self.match_symbol(Symbols::LCurlyBracket) {
            return Ok(Statement::Block(self.parse_block()?));
        }
        let expr = self.parse_expr()?;
        self.expect_keyword(Keyword::SemiColon)?;
        Ok(Statement::Expression(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = vec![];

        while !self.match_symbol(Symbols::RCurlyBracket) {
            if self.peek().is_none() {
                return Err("Unterminated block".into());
            }
            stmts.push(self.parse_statement()?);
        }

        Ok(stmts)
    }

    fn parse_var_decl(&mut self) -> Result<Statement, String> {
        let name = self.expect_ident()?;
        let initializer = if self.match_keyword(Keyword::Assign) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect_keyword(Keyword::SemiColon)?;
        Ok(Statement::VarDecl { name, initializer })
    }

    // ------------------------
    // Expressions
    // ------------------------
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let expr = self.parse_factor()?;

        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Symbols(Symbols::Period) => {
                    break;
                }
                Token::Keyword(Keyword::Colon) => {
                    break;
                }
                _ => break,
            };

            _ = op;
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        self.parse_unary()
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        self.parse_prim()
    }

    fn parse_prim(&mut self) -> Result<Expr, String> {
        // Take one token from the stream.
        let tok = match self.next() {
            Some(t) => t.clone(),
            None => return Err("Unexpected EOF while parsing primary expression".into()),
        };

        match tok {
            Token::Number(num_str) => {
                let val = num_str.parse::<f64>().unwrap_or(0.0);
                Ok(Expr::Literal(Literal::Number(val)))
            }
            Token::Ident(name) => {
                let next_tok = self.peek().cloned();

                match next_tok {
                    Some(Token::Symbols(Symbols::LParen)) => {
                        self.next();

                        let mut args = Vec::new();
                        let next_after_paren = self.peek().cloned();
                        match next_after_paren {
                            Some(Token::Symbols(Symbols::RParen)) => {
                                self.next();
                            }
                            _ => {
                                let arg = self.parse_expr()?;
                                args.push(arg);
                                match self.next() {
                                    Some(Token::Symbols(Symbols::RParen)) => {}
                                    other => {
                                        return Err(format!(
                                            "Expected ')' after argument list, found {:?}",
                                            other
                                        ));
                                    }
                                }
                            }
                        }

                        Ok(Expr::FCall { callee: String::from(name), args })
                    }

                    _ => Ok(Expr::Variable(String::from(name))),
                }
            }
            Token::Symbols(Symbols::LParen) => {
                let expr = self.parse_expr()?;
                match self.next() {
                    Some(Token::Symbols(Symbols::RParen)) => Ok(expr),
                    other => Err(format!("Expected ')' to close grouping, found {:?}", other)),
                }
            }
            other => Err(format!("Unexpected token in expression: {:?}", other)),
        }
    }
}