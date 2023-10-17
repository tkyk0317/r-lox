use std::fmt::{Display, Formatter, Result};

// Token定義
#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier(String),
    String(String),
    Number(f64),
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    token: TokenType,
    lexeme: Option<String>,
    num: usize,
    line: usize,
}
impl Token {
    pub fn new(token: TokenType, lexeme: Option<String>, num: usize, line: usize) -> Self {
        Token {
            token,
            lexeme,
            num,
            line,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "type: {:?} lexeme: {:?} lexeme: {:?}",
            self.token, self.lexeme, self.lexeme
        )
    }
}
