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

use crate::lexer::{
    token::{
        Token,
        Keyword,
    },
};
use crate::parser::{
    ast::*,
};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0
        }
    }

    fn next(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            self.pos += 1;
            self.tokens.get(self.pos - 1)
        } else {
            None
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn parse() {

    }

    pub fn parse_expr() {

    }

    pub fn parse_prim() {
        
    }

}
