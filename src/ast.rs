//! EBNF
//!
//! expression -> equality
//! equality   -> comparison ( ("!=" | "==") comparison ) *;
//! comparison -> term ( (">" | ">=" | "<" | "<=" ) term ) *;
//! term       -> factor ( ( "-" | "+" ) factor ) * ;
//! factor     -> unary ( ( "/" | "*" ) unary ) * ;
//! unary      -> ( "!" | "-" ) unary
//!            | primary ;
//! primary    -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
use crate::token::{Token, TokenType};

type Operation = String;
type ParseResult = Result<AstType, String>;

#[derive(PartialEq, Debug)]
pub enum AstType {
    Equality(Box<AstType>, Operation, Box<AstType>),
    Comparison(Box<AstType>, Operation, Box<AstType>),
    Term(Box<AstType>, Operation, Box<AstType>),
    Factor(Box<AstType>, Operation, Box<AstType>),
    Unary(Operation, Box<AstType>),

    Grouping(Box<AstType>),

    // 終端記号
    Number(f64),
    String(String),
    True,
    False,
    Nil,
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

    /// expression parse
    ///
    /// # Returns
    /// * AstType - パース結果
    pub fn expression(&mut self) -> AstType {
        let result = self.equality();
        if let Ok(result) = result {
            result
        } else {
            // 文の区切りまでSKIPし、再度パースを行う
            self.back();
            self.synchronize();
            self.expression()
        }
    }

    /// equality parse
    ///
    /// # Returns
    /// * Result<AstType, ()> - パース結果
    fn equality(&mut self) -> ParseResult {
        let mut comp = self.comparison()?;
        loop {
            let token = self.token();
            if let Some(token) = token {
                match token.token_type() {
                    TokenType::BangEqual => {
                        let right = self.comparison()?;
                        comp =
                            AstType::Equality(Box::new(comp), String::from("!="), Box::new(right))
                    }
                    TokenType::EqualEqual => {
                        let right = self.comparison()?;
                        comp =
                            AstType::Equality(Box::new(comp), String::from("=="), Box::new(right))
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
    /// * Result<AstType, ()> - パース結果
    fn comparison(&mut self) -> ParseResult {
        let mut term = self.term()?;
        loop {
            let token = self.token();
            if let Some(token) = token {
                match token.token_type() {
                    TokenType::Greater => {
                        let right = self.term()?;
                        term =
                            AstType::Comparison(Box::new(term), String::from(">"), Box::new(right))
                    }
                    TokenType::GreaterEqual => {
                        let right = self.term()?;
                        term =
                            AstType::Comparison(Box::new(term), String::from(">="), Box::new(right))
                    }
                    TokenType::Less => {
                        let right = self.term()?;
                        term =
                            AstType::Comparison(Box::new(term), String::from("<"), Box::new(right))
                    }
                    TokenType::LessEqual => {
                        let right = self.term()?;
                        term =
                            AstType::Comparison(Box::new(term), String::from("<="), Box::new(right))
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
    /// * Result<AstType, ()> - パース結果
    fn term(&mut self) -> ParseResult {
        let mut factor = self.factor()?;
        loop {
            let token = self.token();
            if let Some(token) = token {
                match token.token_type() {
                    TokenType::Minus => {
                        let right = self.factor()?;
                        factor = AstType::Term(Box::new(factor), String::from("-"), Box::new(right))
                    }
                    TokenType::Plus => {
                        let right = self.factor()?;
                        factor = AstType::Term(Box::new(factor), String::from("+"), Box::new(right))
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
    /// * Result<AstType, ()> - パース結果
    fn factor(&mut self) -> ParseResult {
        let mut unary = self.unary()?;
        loop {
            let token = self.token();
            if let Some(token) = token {
                match token.token_type() {
                    TokenType::Slash => {
                        let right = self.unary()?;
                        unary = AstType::Factor(Box::new(unary), String::from("/"), Box::new(right))
                    }
                    TokenType::Star => {
                        let right = self.unary()?;
                        unary = AstType::Factor(Box::new(unary), String::from("*"), Box::new(right))
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
    /// * Result<AstType, ()> - パース結果
    fn unary(&mut self) -> ParseResult {
        let token = self.token();
        if let Some(token) = token {
            match token.token_type() {
                TokenType::Bang => {
                    let unary = self.unary()?;
                    Ok(AstType::Unary(String::from("!"), Box::new(unary)))
                }
                TokenType::Minus => {
                    let unary = self.unary()?;
                    Ok(AstType::Unary(String::from("-"), Box::new(unary)))
                }
                _ => {
                    self.back();
                    self.primary()
                }
            }
        } else {
            Err(String::from("Could not read token"))
        }
    }

    /// primary parse
    ///
    /// # Returns
    /// * Result<AstType, ()> - パース結果
    fn primary(&mut self) -> ParseResult {
        let token = self.token();
        if let Some(token) = token {
            match token.token_type() {
                TokenType::Number(n) => Ok(AstType::Number(*n)),
                TokenType::String(s) => Ok(AstType::String(s.clone())),
                TokenType::True => Ok(AstType::True),
                TokenType::False => Ok(AstType::False),
                TokenType::Nil => Ok(AstType::Nil),
                TokenType::LeftParen => {
                    let expr = self.expression();
                    self.consume(Some(TokenType::RightParen))?;
                    Ok(AstType::Grouping(Box::new(expr)))
                }
                _ => Err(format!("Not Support Token: {:?}", token)),
            }
        } else {
            Err(String::from("Could not read token"))
        }
    }

    /// リードポインターデクリメント
    fn back(&mut self) {
        self.read_pos -= 1;
    }

    /// token取得
    ///
    /// # Returns
    /// * Option<&Token> - Token
    fn token(&mut self) -> Option<&Token> {
        if self.end() {
            None
        } else {
            self.read_pos += 1;
            Some(&self.tokens[self.read_pos - 1])
        }
    }

    /// トークンを消費
    ///
    /// # Arguments
    /// * `expect_token` - Option型。次に期待するTokenがある場合に、指定する
    fn consume(&mut self, expect_token: Option<TokenType>) -> Result<(), String> {
        let token = self.token().expect("Could not read token").token_type();
        if let Some(expect_token) = expect_token {
            if expect_token != *token {
                return Err(format!("Could not found token {:?}", expect_token));
            }
        }

        Ok(())
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
        let tokens = vec![Token::new(TokenType::Number(1.0), None, 0, 0)];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::Number(1.0), parser.expression());

        let tokens = vec![Token::new(
            TokenType::String(String::from("test")),
            None,
            0,
            0,
        )];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::String(String::from("test")), parser.expression());

        let tokens = vec![Token::new(TokenType::True, None, 0, 0)];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::True, parser.expression());

        let tokens = vec![Token::new(TokenType::False, None, 0, 0)];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::False, parser.expression());

        let tokens = vec![Token::new(TokenType::Nil, None, 0, 0)];
        let mut parser = Parser::new(&tokens);
        assert_eq!(AstType::Nil, parser.expression());
    }

    #[test]
    fn グルーピング_parse() {
        let tokens = vec![
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::RightParen, None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Grouping(Box::new(AstType::Number(1.0))),
            parser.expression()
        );
    }

    #[test]
    fn unary_parse() {
        let tokens = vec![
            Token::new(TokenType::Bang, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Unary(String::from("!"), Box::new(AstType::Number(1.0))),
            parser.expression()
        );
        let tokens = vec![
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Unary(String::from("-"), Box::new(AstType::Number(1.0))),
            parser.expression()
        );
    }

    #[test]
    fn factor_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Slash, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Factor(
                Box::new(AstType::Number(2.0)),
                String::from("/"),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Star, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Factor(
                Box::new(AstType::Number(2.0)),
                String::from("*"),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Star, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Slash, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Factor(
                Box::new(AstType::Factor(
                    Box::new(AstType::Number(2.0)),
                    String::from("*"),
                    Box::new(AstType::Number(3.0))
                )),
                String::from("/"),
                Box::new(AstType::Number(1.0)),
            ),
            parser.expression()
        );
    }

    #[test]
    fn term_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Term(
                Box::new(AstType::Number(2.0)),
                String::from("+"),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Term(
                Box::new(AstType::Number(2.0)),
                String::from("-"),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Term(
                Box::new(AstType::Term(
                    Box::new(AstType::Number(2.0)),
                    String::from("+"),
                    Box::new(AstType::Number(3.0))
                )),
                String::from("-"),
                Box::new(AstType::Number(1.0)),
            ),
            parser.expression()
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
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Term(
                Box::new(AstType::Number(2.0)),
                String::from("+"),
                Box::new(AstType::Factor(
                    Box::new(AstType::Number(3.0)),
                    String::from("*"),
                    Box::new(AstType::Number(1.0)),
                ))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Slash, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Minus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Term(
                Box::new(AstType::Factor(
                    Box::new(AstType::Number(2.0)),
                    String::from("/"),
                    Box::new(AstType::Number(3.0)),
                )),
                String::from("-"),
                Box::new(AstType::Number(1.0)),
            ),
            parser.expression()
        );
    }

    #[test]
    fn comparison_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Greater, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Comparison(
                Box::new(AstType::Number(2.0)),
                String::from(">"),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::GreaterEqual, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Comparison(
                Box::new(AstType::Number(2.0)),
                String::from(">="),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Less, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Comparison(
                Box::new(AstType::Number(2.0)),
                String::from("<"),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::LessEqual, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Comparison(
                Box::new(AstType::Number(2.0)),
                String::from("<="),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::Greater, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Star, None, 0, 0),
            Token::new(TokenType::Number(4.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Comparison(
                Box::new(AstType::Term(
                    Box::new(AstType::Number(2.0)),
                    String::from("+"),
                    Box::new(AstType::Number(1.0))
                )),
                String::from(">"),
                Box::new(AstType::Factor(
                    Box::new(AstType::Number(3.0)),
                    String::from("*"),
                    Box::new(AstType::Number(4.0))
                )),
            ),
            parser.expression()
        );
    }

    #[test]
    fn equality_parse() {
        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::EqualEqual, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Equality(
                Box::new(AstType::Number(2.0)),
                String::from("=="),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::BangEqual, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Equality(
                Box::new(AstType::Number(2.0)),
                String::from("!="),
                Box::new(AstType::Number(1.0))
            ),
            parser.expression()
        );

        let tokens = vec![
            Token::new(TokenType::Number(2.0), None, 0, 0),
            Token::new(TokenType::Plus, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::EqualEqual, None, 0, 0),
            Token::new(TokenType::Number(3.0), None, 0, 0),
            Token::new(TokenType::Star, None, 0, 0),
            Token::new(TokenType::Number(4.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);
        assert_eq!(
            AstType::Equality(
                Box::new(AstType::Term(
                    Box::new(AstType::Number(2.0)),
                    String::from("+"),
                    Box::new(AstType::Number(1.0))
                )),
                String::from("=="),
                Box::new(AstType::Factor(
                    Box::new(AstType::Number(3.0)),
                    String::from("*"),
                    Box::new(AstType::Number(4.0))
                )),
            ),
            parser.expression()
        );
    }

    #[test]
    fn synchronize() {
        let tokens = vec![
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::SemiColon, None, 0, 0),
            Token::new(TokenType::Number(8.0), None, 0, 0),
        ];
        let mut parser = Parser::new(&tokens);

        // 不完全な文法部分がSKIPされていること
        assert_eq!(AstType::Number(8.0), parser.expression());
    }
}
