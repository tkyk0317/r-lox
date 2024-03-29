//! EBNF
//!
//! program     -> declaration* EOF ;
//! declaration -> varDecl
//!              | funDecl
//!              | statement ;
//! varDecl     -> "var" IDENTIFIER ( "=" expression )? ";" ;
//! funDecl     -> "fun" function ;
//! function    -> IDENTIFIER "(" parameters? ") block ;
//! statement   -> exprStmt
//!              | ifStmt
//!              | printStmt
//!              | returnStmt
//!              | whileStmt
//!              | forStmt
//!              | block ;
//! forStmt     -> "for" "(" ( varDecl | exprStmt | ";")
//!                expression> ";"
//!                expression? ")" statement ;
//! whileStmt   -> "while" "(" expression ")" statement ;
//! ifStmt      -> "if" "(" expression ")" statement
//!                ( "else" statement )? ;
//! returnStmt  -> "return" expression?  ";"
//! block       -> "" declaration* "" ;
//! exprStmt    -> expression ";" ;
//! printStmt   -> "print" expression ";" ;
//! expression  -> assignment ;
//! assignment  -> IDENTIFIER "=" assignment
//!             | logic_or
//! logic_or    -> logic_and ( "or" logic_and )* ;
//! logic_and   -> equality ( "and" equality )* ;
//! equality    -> comparison ( ("!=" | "==") comparison ) *;
//! comparison  -> term ( (">" | ">=" | "<" | "<=" ) term ) *;
//! term        -> factor ( ( "-" | "+" ) factor ) * ;
//! factor      -> unary ( ( "/" | "*" ) unary ) * ;
//! unary       -> ( "!" | "-" ) unary | call ;
//! call        -> primary ( "(" arguments? ")" )* ;
//! arguments   -> expression ( "," expression )* ;
//! primary     -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;
use crate::token::{Token, TokenType};
use std::error;
use std::fmt;
use std::vec::Vec;

/// 解析エラー
pub enum ParseError {
    CouldNotReadToken,
    NotFoundToken(String),
    NotFoundAstType(String),
    NotSupportToken(String),
}
impl ParseError {
    fn print(&self) -> String {
        match self {
            Self::CouldNotReadToken => "Could not read token".to_string(),
            Self::NotFoundToken(token) => format!("Could not found {:?} token", token),
            Self::NotFoundAstType(ast) => format!("Could not found {:?} ast type", ast),
            Self::NotSupportToken(token) => format!("Could not support {:?} token", token),
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.print())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.print())
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

type ParseResult = Result<AstType, ParseError>;

#[derive(PartialEq, Clone, Debug)]
pub enum AstType {
    // varDecl
    Var(String, Box<AstType>),
    Fun(String, Vec<AstType>, Box<AstType>), // 関数名、引数列、ブロック

    // statement
    Print(Box<AstType>),
    Block(Vec<AstType>),
    While(Box<AstType>, Box<AstType>),            // 条件、ブロック
    If(Box<AstType>, Box<AstType>, Box<AstType>), // 条件、IFブロック、ELSEブロック
    Return(Box<AstType>),

    // Assignment
    Assign(String, Box<AstType>),

    // Equality
    BangEqual(Box<AstType>, Box<AstType>),
    EqualEqual(Box<AstType>, Box<AstType>),
    And(Box<AstType>, Box<AstType>),
    Or(Box<AstType>, Box<AstType>),

    // Comparison
    Greater(Box<AstType>, Box<AstType>),
    GreaterEqual(Box<AstType>, Box<AstType>),
    Less(Box<AstType>, Box<AstType>),
    LessEqual(Box<AstType>, Box<AstType>),

    // Term
    Minus(Box<AstType>, Box<AstType>),
    Plus(Box<AstType>, Box<AstType>),

    // Factor
    Div(Box<AstType>, Box<AstType>),
    Mul(Box<AstType>, Box<AstType>),

    // Unary
    Bang(Box<AstType>),
    UnaryMinus(Box<AstType>),
    Call(String, Vec<AstType>), // 関数名、引数

    // primary
    Grouping(Box<AstType>),

    // 終端記号
    Number(f64),
    String(String),
    True,
    False,
    Nil,
    Identifier(String),
}

pub struct Parser<'a> {
    read_pos: usize,
    tokens: &'a Vec<Token>,
}
impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Parser {
            read_pos: 0,
            tokens,
        }
    }

    /// program parse
    ///
    /// # Returns
    /// * Vec<AstType> - パース結果
    pub fn program(&mut self) -> Vec<AstType> {
        let mut result = vec![];
        loop {
            self.declaration().map_or_else(
                |_| {
                    // 文の区切りまでSKIPし、再度パースを行う
                    self.back();
                    self.synchronize();
                },
                |parse_result| {
                    result.push(parse_result);
                },
            );

            if self.end() {
                break;
            }
        }

        result
    }

    /// declaration parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn declaration(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::Var => self.var_declaration(),
                    TokenType::Fun => self.fun_declaration(),
                    _ => {
                        self.back();
                        self.statement()
                    }
                }
            })
    }

    /// fun declaration parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn fun_declaration(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::Identifier(i) => {
                        let identifier = i.clone();
                        self.consume(Some(TokenType::LeftParen))?;
                        let args = self.fun_parameters()?;
                        self.consume(Some(TokenType::RightParen))?;
                        self.consume(Some(TokenType::LeftBrace))?;
                        let body = self.block_statement()?;
                        Ok(AstType::Fun(identifier.to_string(), args, Box::new(body)))
                    }
                    _ => Err(ParseError::NotFoundToken(String::from("Identifier"))),
                }
            })
    }

    /// function parameter parse
    ///
    /// # Returns
    /// * Result<Vec<AstType>, ParseError> - パース結果
    fn fun_parameters(&mut self) -> Result<Vec<AstType>, ParseError> {
        let mut arguments = vec![];

        loop {
            if let Some(token) = self.token() {
                match token.token_type() {
                    TokenType::RightParen => {
                        self.back();
                        break;
                    }
                    TokenType::Comma => continue,
                    _ => {
                        self.back();
                        arguments.push(self.fun_one_parameter()?);
                    }
                }
            } else {
                return Err(ParseError::CouldNotReadToken);
            }

            // 引数の数は255までしか解釈しない
            if arguments.len() >= 255 {
                println!("Can not have more than 255 arguments");
                break;
            }
        }

        Ok(arguments)
    }

    /// create function parameter
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn fun_one_parameter(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::Identifier(_i) => {
                        self.back();
                        self.primary()
                    }
                    _ => Err(ParseError::NotFoundToken(String::from("Identifier"))),
                }
            })
    }

    /// var declaration parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn var_declaration(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::Identifier(i) => self.declaration_identifier(i.clone()),
                    _ => Err(ParseError::NotFoundToken(String::from("Identifier"))),
                }
            })
    }

    /// create identifier declaration
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn declaration_identifier(&mut self, identifier: String) -> ParseResult {
        self.token().map_or(
            Err(ParseError::NotFoundToken(String::from("Identifier"))),
            |token| {
                match token.token_type() {
                    TokenType::Equal => {
                        let expr = self.expression()?;
                        self.consume(Some(TokenType::SemiColon))?;
                        Ok(AstType::Var(identifier, Box::new(expr)))
                    }
                    // 初期化されていない変数は、nilで初期化
                    TokenType::SemiColon => Ok(AstType::Var(identifier, Box::new(AstType::Nil))),
                    _ => Err(ParseError::NotFoundToken(String::from("SemiColon"))),
                }
            },
        )
    }

    /// statement parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn statement(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::Print => self.print_statement(),
                    TokenType::If => self.if_statement(),
                    TokenType::While => self.while_statement(),
                    TokenType::For => self.for_statement(),
                    TokenType::Return => self.return_statement(),
                    TokenType::LeftBrace => self.block_statement(),
                    _ => {
                        self.back();
                        self.expression_stmt()
                    }
                }
            })
    }

    /// return statement parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn return_statement(&mut self) -> ParseResult {
        let expr = if let Some(token) = self.token() {
            match token.token_type() {
                TokenType::SemiColon => {
                    self.back();
                    AstType::Nil
                }
                _ => {
                    self.back();
                    self.expression()?
                }
            }
        } else {
            return Err(ParseError::CouldNotReadToken);
        };

        self.consume(Some(TokenType::SemiColon))?;

        Ok(AstType::Return(Box::new(expr)))
    }

    /// while statement parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn while_statement(&mut self) -> ParseResult {
        self.consume(Some(TokenType::LeftParen))?;
        let condition = self.expression()?;
        self.consume(Some(TokenType::RightParen))?;
        let stmt = self.statement()?;

        Ok(AstType::While(Box::new(condition), Box::new(stmt)))
    }

    /// for statement parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn for_statement(&mut self) -> ParseResult {
        self.consume(Some(TokenType::LeftParen))?;
        let initialize = self.for_initialize()?;
        let condition = self.for_condition()?;
        self.consume(Some(TokenType::SemiColon))?;
        let increment = self.for_increment()?;
        self.consume(Some(TokenType::RightParen))?;
        let stmt = self.statement()?;

        Ok(AstType::Block(vec![
            initialize,
            AstType::While(
                Box::new(condition),
                Box::new(AstType::Block(vec![stmt, increment])),
            ),
        ]))
    }

    /// for initialize parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn for_initialize(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::SemiColon => Ok(AstType::Nil),
                    TokenType::Var => self.var_declaration(),
                    _ => {
                        self.back();
                        self.expression_stmt()
                    }
                }
            })
    }

    /// for condition parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn for_condition(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::SemiColon => {
                        self.back();
                        Ok(AstType::True)
                    }
                    _ => {
                        self.back();
                        self.expression()
                    }
                }
            })
    }

    /// for increment parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn for_increment(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::RightParen => {
                        self.back();
                        Ok(AstType::Nil)
                    }
                    _ => {
                        self.back();
                        self.expression()
                    }
                }
            })
    }

    /// if statement parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn if_statement(&mut self) -> ParseResult {
        self.consume(Some(TokenType::LeftParen))?;
        let condition = self.expression()?;
        self.consume(Some(TokenType::RightParen))?;
        let if_stmt = self.statement()?;

        let mut else_stmt = AstType::Nil;
        if let Some(token) = self.token() {
            match token.token_type() {
                TokenType::Else => {
                    else_stmt = self.statement()?;
                }
                _ => {
                    self.back();
                }
            }
        }

        Ok(AstType::If(
            Box::new(condition),
            Box::new(if_stmt),
            Box::new(else_stmt),
        ))
    }

    /// print statement parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn print_statement(&mut self) -> ParseResult {
        let expr = self.expression()?;
        self.consume(Some(TokenType::SemiColon))?;

        Ok(AstType::Print(Box::new(expr)))
    }

    /// block statement
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn block_statement(&mut self) -> ParseResult {
        let mut ast = vec![];
        loop {
            if let Some(token) = self.token() {
                match token.token_type() {
                    TokenType::RightBrace => {
                        self.back();
                        break;
                    }
                    _ => {
                        self.back();
                        ast.push(self.declaration()?);
                    }
                }
            } else {
                return Err(ParseError::CouldNotReadToken);
            }
        }
        self.consume(Some(TokenType::RightBrace))?;

        Ok(AstType::Block(ast))
    }

    /// exprStmt parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn expression_stmt(&mut self) -> ParseResult {
        let expr = self.expression()?;
        self.consume(Some(TokenType::SemiColon))?;

        Ok(expr)
    }

    /// expression parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn expression(&mut self) -> ParseResult {
        self.assignment()
    }

    /// assignment parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn assignment(&mut self) -> ParseResult {
        let expr = self.or_parse()?;

        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::Equal => match expr {
                        AstType::Identifier(i) => {
                            let right_expr = self.assignment()?;
                            Ok(AstType::Assign(i, Box::new(right_expr)))
                        }
                        _ => Err(ParseError::NotFoundAstType(String::from("Identifier"))),
                    },
                    _ => {
                        self.back();
                        Ok(expr)
                    }
                }
            })
    }

    /// or parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn or_parse(&mut self) -> ParseResult {
        let mut expr = self.and_parse()?;

        loop {
            if let Some(token) = self.token() {
                match token.token_type() {
                    TokenType::Or => {
                        let right = self.and_parse()?;
                        expr = AstType::Or(Box::new(expr), Box::new(right));
                    }
                    _ => {
                        self.back();
                        break;
                    }
                };
            } else {
                return Err(ParseError::CouldNotReadToken);
            }
        }

        Ok(expr)
    }

    /// and parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn and_parse(&mut self) -> ParseResult {
        let mut expr = self.equality()?;

        loop {
            if let Some(token) = self.token() {
                match token.token_type() {
                    TokenType::And => {
                        let right = self.equality()?;
                        expr = AstType::And(Box::new(expr), Box::new(right));
                    }
                    _ => {
                        self.back();
                        break;
                    }
                };
            } else {
                return Err(ParseError::CouldNotReadToken);
            }
        }

        Ok(expr)
    }

    /// equality parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn equality(&mut self) -> ParseResult {
        let mut comp = self.comparison()?;
        loop {
            let token = self.token();
            if let Some(token) = token {
                match token.token_type() {
                    TokenType::BangEqual => {
                        let right = self.comparison()?;
                        comp = AstType::BangEqual(Box::new(comp), Box::new(right))
                    }
                    TokenType::EqualEqual => {
                        let right = self.comparison()?;
                        comp = AstType::EqualEqual(Box::new(comp), Box::new(right))
                    }
                    _ => {
                        self.back();
                        break;
                    }
                };
            } else {
                break;
            }
        }

        Ok(comp)
    }

    /// comparison parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn comparison(&mut self) -> ParseResult {
        let mut term = self.term()?;
        loop {
            let token = self.token();
            if let Some(token) = token {
                match token.token_type() {
                    TokenType::Greater => {
                        let right = self.term()?;
                        term = AstType::Greater(Box::new(term), Box::new(right))
                    }
                    TokenType::GreaterEqual => {
                        let right = self.term()?;
                        term = AstType::GreaterEqual(Box::new(term), Box::new(right))
                    }
                    TokenType::Less => {
                        let right = self.term()?;
                        term = AstType::Less(Box::new(term), Box::new(right))
                    }
                    TokenType::LessEqual => {
                        let right = self.term()?;
                        term = AstType::LessEqual(Box::new(term), Box::new(right))
                    }
                    _ => {
                        self.back();
                        break;
                    }
                };
            } else {
                break;
            }
        }
        Ok(term)
    }

    /// term parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn term(&mut self) -> ParseResult {
        let mut factor = self.factor()?;
        loop {
            let token = self.token();
            if let Some(token) = token {
                match token.token_type() {
                    TokenType::Minus => {
                        let right = self.factor()?;
                        factor = AstType::Minus(Box::new(factor), Box::new(right))
                    }
                    TokenType::Plus => {
                        let right = self.factor()?;
                        factor = AstType::Plus(Box::new(factor), Box::new(right))
                    }
                    _ => {
                        self.back();
                        break;
                    }
                };
            } else {
                break;
            }
        }

        Ok(factor)
    }

    /// factory parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn factor(&mut self) -> ParseResult {
        let mut unary = self.unary()?;
        loop {
            let token = self.token();
            if let Some(token) = token {
                match token.token_type() {
                    TokenType::Slash => {
                        let right = self.unary()?;
                        unary = AstType::Div(Box::new(unary), Box::new(right))
                    }
                    TokenType::Star => {
                        let right = self.unary()?;
                        unary = AstType::Mul(Box::new(unary), Box::new(right))
                    }
                    _ => {
                        self.back();
                        break;
                    }
                };
            } else {
                break;
            }
        }

        Ok(unary)
    }

    /// unary parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn unary(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::Bang => {
                        let unary = self.unary()?;
                        Ok(AstType::Bang(Box::new(unary)))
                    }
                    TokenType::Minus => {
                        let unary = self.unary()?;
                        Ok(AstType::UnaryMinus(Box::new(unary)))
                    }
                    _ => {
                        self.back();
                        self.call()
                    }
                }
            })
    }

    /// call parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn call(&mut self) -> ParseResult {
        let expr = self.primary()?;

        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::LeftParen => {
                        let arguments = self.arguments()?;
                        match expr {
                            AstType::Identifier(i) => Ok(AstType::Call(i, arguments)),
                            _ => Err(ParseError::NotFoundAstType(String::from("Identifier"))),
                        }
                    }
                    _ => {
                        self.back();
                        Ok(expr)
                    }
                }
            })
    }

    /// arguments parse
    ///
    /// # Returns
    /// * Result<Vec<AstType>, ParseError> - パース結果
    fn arguments(&mut self) -> Result<Vec<AstType>, ParseError> {
        let mut arguments = vec![];

        loop {
            if let Some(token) = self.token() {
                match token.token_type() {
                    TokenType::RightParen => {
                        self.back();
                        break;
                    }
                    TokenType::Comma => continue,
                    _ => {
                        self.back();
                        let arg = self.expression();
                        if let Ok(arg) = arg {
                            arguments.push(arg);
                        }
                    }
                }
            } else {
                return Err(ParseError::CouldNotReadToken);
            }

            // 引数の数は255までしか解釈しない
            if arguments.len() >= 255 {
                println!("Can not have more than 255 arguments");
                break;
            }
        }

        self.consume(Some(TokenType::RightParen))?;

        Ok(arguments)
    }

    /// primary parse
    ///
    /// # Returns
    /// * ParseResult - パース結果
    fn primary(&mut self) -> ParseResult {
        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                match token.token_type() {
                    TokenType::Number(n) => Ok(AstType::Number(*n)),
                    TokenType::String(s) => Ok(AstType::String(s.clone())),
                    TokenType::True => Ok(AstType::True),
                    TokenType::False => Ok(AstType::False),
                    TokenType::Nil => Ok(AstType::Nil),
                    TokenType::LeftParen => {
                        let expr = self.expression()?;
                        self.consume(Some(TokenType::RightParen))?;
                        Ok(AstType::Grouping(Box::new(expr)))
                    }
                    TokenType::Identifier(i) => Ok(AstType::Identifier(i.to_string())),
                    _ => Err(ParseError::NotSupportToken(format!("{:?}", token))),
                }
            })
    }

    /// リードポインターデクリメント
    fn back(&mut self) {
        self.read_pos -= 1;
    }

    /// token取得
    ///
    /// # Returns
    /// * Option<Token> - Token
    fn token(&mut self) -> Option<Token> {
        if self.end() {
            None
        } else {
            self.read_pos += 1;
            Some(self.tokens[self.read_pos - 1].clone())
        }
    }

    /// トークンを消費
    ///
    /// # Arguments
    /// * `expect_token` - Option型。次に期待するTokenがある場合に、指定する
    ///
    /// # Return
    /// * Result<Token, ParseError> - パース結果
    fn consume(&mut self, expect_token: Option<TokenType>) -> Result<Token, ParseError> {
        if self.end() {
            return Err(ParseError::CouldNotReadToken);
        }

        self.token()
            .map_or(Err(ParseError::CouldNotReadToken), |token| {
                expect_token.map_or(Ok(token.clone()), |expect_token| {
                    if expect_token != *token.token_type() {
                        Err(ParseError::NotFoundToken(format!("{:?}", expect_token)))
                    } else {
                        Ok(token.clone())
                    }
                })
            })
    }

    /// 文の区切りまでSKIPし、同期を取る。エラーが発生した際に、使用する
    fn synchronize(&mut self) {
        loop {
            if self.end() {
                break;
            }

            // 文の区切りであろうTokenまでSKIP
            let token = self.token();
            if let Some(token) = token {
                match token.token_type() {
                    TokenType::SemiColon => break,
                    TokenType::Class
                    | TokenType::For
                    | TokenType::Fun
                    | TokenType::If
                    | TokenType::Print
                    | TokenType::Var
                    | TokenType::Return
                    | TokenType::While => {
                        self.back();
                        break;
                    }
                    _ => continue,
                }
            }
        }
    }

    /// トークン終了判定
    ///
    /// # Return
    /// * bool - true: トークン終了 false: トークン未終了
    fn end(&self) -> bool {
        self.read_pos >= self.tokens.len()
            || *self.tokens[self.read_pos].token_type() == TokenType::Eof
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn 終端記号_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::Number(1.0), parser.program()[0]);

        let tokens = vec![
            Token::new(TokenType::String(String::from("test")), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::String(String::from("test")), parser.program()[0]);

        let tokens = vec![
            Token::new(TokenType::True, None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::True, parser.program()[0]);

        let tokens = vec![
            Token::new(TokenType::False, None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::False, parser.program()[0]);

        let tokens = vec![
            Token::new(TokenType::Nil, None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::Nil, parser.program()[0]);
    }

    #[test]
    fn グルーピング_parse() {
        let tokens = vec![
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Grouping(Box::new(AstType::Number(1.0))),
            parser.program()[0]
        );
    }

    #[test]
    fn unary_parse() {
        let tokens = vec![
            Token::new(TokenType::Bang, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Bang(Box::new(AstType::Number(1.0))),
            parser.program()[0]
        );
        let tokens = vec![
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::UnaryMinus(Box::new(AstType::Number(1.0))),
            parser.program()[0]
        );
    }

    #[test]
    fn factor_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Slash, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Div(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Star, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Mul(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Star, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Slash, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Div(
                Box::new(AstType::Mul(
                    Box::new(AstType::Number(2.0)),
                    Box::new(AstType::Number(3.0))
                )),
                Box::new(AstType::Number(1.0)),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn term_parse() {
        let tokens = vec![
            Token::new(TokenType::String(String::from("a")), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::String(String::from("b")), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Plus(
                Box::new(AstType::String(String::from("a"))),
                Box::new(AstType::String(String::from("b")))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Plus(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Minus(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Minus(
                Box::new(AstType::Plus(
                    Box::new(AstType::Number(2.0)),
                    Box::new(AstType::Number(3.0))
                )),
                Box::new(AstType::Number(1.0)),
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(10.0), None, 0, 0),
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Minus(
                Box::new(AstType::Minus(
                    Box::new(AstType::Number(10.0)),
                    Box::new(AstType::Number(3.0))
                )),
                Box::new(AstType::Number(1.0)),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn 四則演算混合_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Star, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Plus(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Mul(
                    Box::new(AstType::Number(3.0)),
                    Box::new(AstType::Number(1.0)),
                ))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Slash, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Minus(
                Box::new(AstType::Div(
                    Box::new(AstType::Number(2.0)),
                    Box::new(AstType::Number(3.0)),
                )),
                Box::new(AstType::Number(1.0)),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn comparison_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Greater, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Greater(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::GreaterEqual, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::GreaterEqual(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Less, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Less(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::LessEqual, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::LessEqual(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::Greater, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Star, None, 0, 0),
            Token::new(TokenType::Number(4.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Greater(
                Box::new(AstType::Plus(
                    Box::new(AstType::Number(2.0)),
                    Box::new(AstType::Number(1.0))
                )),
                Box::new(AstType::Mul(
                    Box::new(AstType::Number(3.0)),
                    Box::new(AstType::Number(4.0))
                )),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn equality_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::EqualEqual, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::EqualEqual(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::BangEqual, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::BangEqual(
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(1.0))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::EqualEqual, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Star, None, 0, 0),
            Token::new(TokenType::Number(4.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::EqualEqual(
                Box::new(AstType::Plus(
                    Box::new(AstType::Number(2.0)),
                    Box::new(AstType::Number(1.0))
                )),
                Box::new(AstType::Mul(
                    Box::new(AstType::Number(3.0)),
                    Box::new(AstType::Number(4.0))
                )),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn synchronize() {
        let tokens = vec![
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::Number(8.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);

        // 不完全な文法部分がSKIPされていること
        assert_eq!(AstType::Number(8.0), parser.program()[0]);
    }

    #[test]
    fn 複数行_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 1, 0),
            Token::new(TokenType::Number(2.0), None, 2, 0),
            Token::new(TokenType::SemiColon, None, 1, 0),
        ];
        let mut parser = Parser::new(&tokens);
        let result = parser.program();
        assert_eq!(AstType::Number(1.0), result[0]);
        assert_eq!(AstType::Number(2.0), result[1]);
    }

    #[test]
    fn 文末にセミコロンがない_parse() {
        let tokens = vec![Token::new(TokenType::Number(1.0), None, 0, 0)];
        let mut parser = Parser::new(&tokens);
        let result = parser.program();
        assert_eq!(0, result.len());
    }

    #[test]
    fn print_parse() {
        let tokens = vec![
            Token::new(TokenType::Print, None, 0, 0),
            Token::new(TokenType::String(String::from("test")), None, 1, 0),
            Token::new(TokenType::SemiColon, None, 2, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Print(Box::new(AstType::String(String::from("test")))),
            parser.program()[0]
        );
    }

    #[test]
    fn identifier_parse() {
        let tokens = vec![
            Token::new(TokenType::Var, None, 0, 0),
            Token::new(TokenType::Identifier(String::from("test")), None, 0, 0),
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Var(String::from("test"), Box::new(AstType::Number(2.0))),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Var, None, 0, 0),
            Token::new(TokenType::Identifier(String::from("test")), None, 0, 0),
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::String("Hello".to_owned()), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Var(
                String::from("test"),
                Box::new(AstType::String("Hello".to_owned()))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Var, None, 0, 0),
            Token::new(TokenType::Identifier(String::from("test")), None, 0, 0),
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Var(
                String::from("test"),
                Box::new(AstType::Plus(
                    Box::new(AstType::Number(2.0)),
                    Box::new(AstType::Number(3.0)),
                ))
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Var, None, 0, 0),
            Token::new(TokenType::Identifier(String::from("a")), None, 0, 0),
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::Print, None, 0, 0),
            Token::new(TokenType::Identifier("a".to_string()), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        let result = parser.program();
        assert_eq!(
            AstType::Var(String::from("a"), Box::new(AstType::Number(2.0))),
            result[0]
        );
        assert_eq!(
            AstType::Print(Box::new(AstType::Identifier("a".to_string()))),
            result[1]
        );
    }

    #[test]
    fn assign_parse() {
        let tokens = vec![
            Token::new(TokenType::Identifier("test".to_string()), None, 0, 0),
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Assign(String::from("test"), Box::new(AstType::Number(1.0))),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Identifier("test".to_string()), None, 0, 0),
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Assign(
                String::from("test"),
                Box::new(AstType::Plus(
                    Box::new(AstType::Number(1.0)),
                    Box::new(AstType::Number(2.0))
                ))
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn block_parse() {
        let tokens = vec![
            Token::new(TokenType::LeftBrace, None, 0, 0),
            Token::new(TokenType::Identifier("test".to_string()), None, 0, 0),
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::RightBrace, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Block(vec![AstType::Assign(
                String::from("test"),
                Box::new(AstType::Number(1.0))
            ),]),
            parser.program()[0]
        );
    }

    #[test]
    fn if_parse() {
        let tokens = vec![
            Token::new(TokenType::If, None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::Else, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::If(
                Box::new(AstType::Number(1.0)),
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Number(3.0)),
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::If, None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::If(
                Box::new(AstType::Number(1.0)),
                Box::new(AstType::Number(2.0)),
                Box::new(AstType::Nil),
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::If, None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::Less, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::If(
                Box::new(AstType::Less(
                    Box::new(AstType::Number(1.0)),
                    Box::new(AstType::Number(2.0)),
                )),
                Box::new(AstType::Number(3.0)),
                Box::new(AstType::Nil),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn or_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::Or, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Or(
                Box::new(AstType::Number(1.0)),
                Box::new(AstType::Number(2.0)),
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::Or, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Or, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Or(
                Box::new(AstType::Or(
                    Box::new(AstType::Number(1.0)),
                    Box::new(AstType::Number(2.0)),
                )),
                Box::new(AstType::Number(3.0)),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn and_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::And, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::And(
                Box::new(AstType::Number(1.0)),
                Box::new(AstType::Number(2.0)),
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::And, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::And, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::And(
                Box::new(AstType::And(
                    Box::new(AstType::Number(1.0)),
                    Box::new(AstType::Number(2.0)),
                )),
                Box::new(AstType::Number(3.0)),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn or_and_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::Or, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::And, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Or(
                Box::new(AstType::Number(1.0)),
                Box::new(AstType::And(
                    Box::new(AstType::Number(2.0)),
                    Box::new(AstType::Number(3.0)),
                )),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn while_parse() {
        let tokens = vec![
            Token::new(TokenType::While, None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::While(
                Box::new(AstType::Number(1.0)),
                Box::new(AstType::Number(2.0)),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn for_parse() {
        let tokens = vec![
            Token::new(TokenType::For, None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Var, None, 0, 0),
            Token::new(TokenType::Identifier("a".to_string()), None, 0, 0),
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::Identifier("a".to_string()), None, 0, 0),
            Token::new(TokenType::Less, None, 0, 0),
            Token::new(TokenType::Number(10.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::Identifier("a".to_string()), None, 0, 0),
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::Identifier("a".to_string()), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::Print, None, 0, 0),
            Token::new(TokenType::String("Hello".to_string()), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Block(vec![
                AstType::Var("a".to_string(), Box::new(AstType::Number(1.0))),
                AstType::While(
                    Box::new(AstType::Less(
                        Box::new(AstType::Identifier("a".to_string())),
                        Box::new(AstType::Number(10.0))
                    )),
                    Box::new(AstType::Block(vec![
                        AstType::Print(Box::new(AstType::String("Hello".to_string()))),
                        AstType::Assign(
                            "a".to_string(),
                            Box::new(AstType::Plus(
                                Box::new(AstType::Identifier("a".to_string())),
                                Box::new(AstType::Number(1.0))
                            ))
                        )
                    ]))
                )
            ]),
            parser.program()[0]
        );
    }

    #[test]
    fn 関数コール_parse() {
        let tokens = vec![
            Token::new(TokenType::Identifier("test_func".to_string()), None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::Comma, None, 0, 0),
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Call(
                "test_func".to_string(),
                vec![AstType::Number(1.0), AstType::Number(2.0)]
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Identifier("test_func".to_string()), None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Call("test_func".to_string(), vec![],),
            parser.program()[0]
        );
    }

    #[test]
    fn 関数定義_parse() {
        let tokens = vec![
            Token::new(TokenType::Fun, None, 0, 0),
            Token::new(TokenType::Identifier("test_func".to_string()), None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::LeftBrace, None, 0, 0),
            Token::new(TokenType::RightBrace, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Fun(
                "test_func".to_string(),
                vec![],
                Box::new(AstType::Block(vec![])),
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Fun, None, 0, 0),
            Token::new(TokenType::Identifier("test_func".to_string()), None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::LeftBrace, None, 0, 0),
            Token::new(TokenType::Print, None, 0, 0),
            Token::new(TokenType::String(String::from("test")), None, 1, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::RightBrace, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Fun(
                "test_func".to_string(),
                vec![],
                Box::new(AstType::Block(vec![AstType::Print(Box::new(
                    AstType::String(String::from("test"))
                )),])),
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Fun, None, 0, 0),
            Token::new(TokenType::Identifier("test_func".to_string()), None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Identifier("a".to_string()), None, 0, 0),
            Token::new(TokenType::Comma, None, 0, 0),
            Token::new(TokenType::Identifier("b".to_string()), None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::LeftBrace, None, 0, 0),
            Token::new(TokenType::RightBrace, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Fun(
                "test_func".to_string(),
                vec![
                    AstType::Identifier("a".to_string()),
                    AstType::Identifier("b".to_string()),
                ],
                Box::new(AstType::Block(vec![])),
            ),
            parser.program()[0]
        );
    }

    #[test]
    fn return_parse() {
        let tokens = vec![
            Token::new(TokenType::Fun, None, 0, 0),
            Token::new(TokenType::Identifier("test_func".to_string()), None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::LeftBrace, None, 0, 0),
            Token::new(TokenType::Return, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::RightBrace, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Fun(
                "test_func".to_string(),
                vec![],
                Box::new(AstType::Block(vec![AstType::Return(Box::new(
                    AstType::Number(1.0)
                ))])),
            ),
            parser.program()[0]
        );

        let tokens = vec![
            Token::new(TokenType::Fun, None, 0, 0),
            Token::new(TokenType::Identifier("test_func".to_string()), None, 0, 0),
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
            Token::new(TokenType::LeftBrace, None, 0, 0),
            Token::new(TokenType::Return, None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::RightBrace, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Fun(
                "test_func".to_string(),
                vec![],
                Box::new(AstType::Block(vec![AstType::Return(Box::new(
                    AstType::Nil
                ))])),
            ),
            parser.program()[0]
        );
    }
}
