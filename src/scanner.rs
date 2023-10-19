use crate::token::{Token, TokenType};
use std::collections::HashMap;
use std::vec::Vec;

#[derive(Debug)]
pub struct Scanner<'a> {
    contents: &'a String,
    keywords: HashMap<String, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(contents: &'a String) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert(String::from("and"), TokenType::And);
        keywords.insert(String::from("class"), TokenType::Class);
        keywords.insert(String::from("else"), TokenType::Else);
        keywords.insert(String::from("false"), TokenType::False);
        keywords.insert(String::from("for"), TokenType::For);
        keywords.insert(String::from("fun"), TokenType::Fun);
        keywords.insert(String::from("if"), TokenType::If);
        keywords.insert(String::from("nil"), TokenType::Nil);
        keywords.insert(String::from("or"), TokenType::Or);
        keywords.insert(String::from("print"), TokenType::Print);
        keywords.insert(String::from("return"), TokenType::Return);
        keywords.insert(String::from("super"), TokenType::Super);
        keywords.insert(String::from("this"), TokenType::This);
        keywords.insert(String::from("true"), TokenType::True);
        keywords.insert(String::from("var"), TokenType::Var);
        keywords.insert(String::from("while"), TokenType::While);

        Scanner { contents, keywords }
    }

    /// 文字列スキャン開始
    ///
    /// # Return
    /// * Vec<Token> - TokenのVec
    pub fn scan(&self) -> Vec<Token> {
        let mut cur = 0;
        let mut line = 0;
        let mut tokens: Vec<Token> = vec![];
        let chars = self.contents.chars().collect::<Vec<char>>();
        loop {
            let cur_char = chars[cur];
            cur = match cur_char {
                '\n' | '\r' => {
                    line += 1;
                    cur + 1
                }
                '\t' | ' ' => cur + 1,
                _ => {
                    // コメントをSKIPするので、次の文字まで取得しておく
                    let next_char: Option<char> = if self.end(cur + 1) {
                        None
                    } else {
                        Some(chars[cur + 1])
                    };

                    if let Some(next_char) = next_char {
                        // コメントのSKIP
                        if next_char == '/' {
                            let read_num = self.skip_line(&chars[cur..]);
                            cur + read_num
                        } else {
                            let (t, read_num) = self.scan_token(&chars, cur, line);
                            tokens.push(t);
                            cur + read_num
                        }
                    } else {
                        let (t, read_num) = self.scan_token(&chars, cur, line);
                        tokens.push(t);
                        cur + read_num
                    }
                }
            };

            if self.end(cur) {
                break;
            }
        }

        tokens.push(Token::new(TokenType::Eof, None, cur, line));

        tokens
    }

    /// 1行SKIP
    ///
    /// # Arguments
    /// * `s` - 読み取り対象文字列
    ///
    /// # Returns
    /// * usize - 読み取った文字数
    fn skip_line(&self, s: &[char]) -> usize {
        let mut read_num = 0;
        for (i, val) in s.iter().enumerate() {
            if self.end(i) {
                break;
            }
            read_num += 1;
            if *val == '\n' {
                break;
            }
        }

        read_num
    }

    /// 読み込み文字列の終了判定
    ///
    /// # Arguments
    /// * `num` - 読み込んだ文字数
    ///
    /// # Return
    /// * bool - true: 終了 false: 未終了
    fn end(&self, num: usize) -> bool {
        num >= self.contents.chars().collect::<Vec<char>>().len()
    }

    /// TokenTypeのスキャン
    ///
    /// # Arguments
    /// * `s` - スキャンする文字列
    /// * `cur` - 読み取り位置
    ///
    /// # Return
    /// * [Token, usize] - Tokenと読み取り文字数のタプル
    fn scan_token(&self, s: &Vec<char>, cur: usize, line: usize) -> (Token, usize) {
        let c = s[cur];
        let mut read_num = 1;
        let t = match c {
            '"' => {
                // ダブルクォーテーションの次の文字位置からサーチ
                let (token, num) = self.string(cur, &s[(cur + 1)..], line);
                read_num = num + 1;
                token
            }
            '0'..='9' => {
                let (token, num) = self.number(cur, &s[cur..], line);
                read_num = num;
                token
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                // アルファベットもしくはアンダースコアから始まる
                let (token, num) = self.identifier(cur, &s[cur..], line);
                read_num = num;
                token
            }
            '(' => Token::new(TokenType::LeftParen, None, cur, line),
            ')' => Token::new(TokenType::RightParen, None, cur, line),
            '{' => Token::new(TokenType::LeftBrace, None, cur, line),
            '}' => Token::new(TokenType::RightBrace, None, cur, line),
            ',' => Token::new(TokenType::Comma, None, cur, line),
            '.' => Token::new(TokenType::Dot, None, cur, line),
            '-' => Token::new(TokenType::Minus, None, cur, line),
            '+' => Token::new(TokenType::Plus, None, cur, line),
            ';' => Token::new(TokenType::SemiColon, None, cur, line),
            '*' => Token::new(TokenType::Star, None, cur, line),
            '/' => Token::new(TokenType::Slash, None, cur, line),
            '!' => Token::new(
                if self.next_match(s, cur + 1, '=') {
                    read_num += 1;
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                },
                None,
                cur,
                line,
            ),
            '=' => Token::new(
                if self.next_match(s, cur + 1, '=') {
                    read_num += 1;
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                },
                None,
                cur,
                line,
            ),
            '<' => Token::new(
                if self.next_match(s, cur + 1, '=') {
                    read_num += 1;
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                },
                None,
                cur,
                line,
            ),
            '>' => Token::new(
                if self.next_match(s, cur + 1, '=') {
                    read_num += 1;
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                },
                None,
                cur,
                line,
            ),
            _ => panic!("Not Support Token: {:?}", c),
        };

        (t, read_num)
    }

    /// 文字列リテラル取得
    ///
    /// # Arguments
    /// * `s` - 読み取り対象文字列（ダブルクォーテーションの次の文字からの配列）
    /// * `cur` - 文字列の読み取り位置
    /// * `line` - 行数
    ///
    /// # Return
    /// * (Token, usize) - 文字列リテラルに対応するトークンと読み取り文字数のタプル
    fn string(&self, cur: usize, s: &[char], line: usize) -> (Token, usize) {
        // 次のダブルクォーテーションまで
        let mut literal = String::new();
        let mut read_num = 0;
        for (i, val) in s.iter().enumerate() {
            if *val != '"' && !self.end(i) {
                literal.push_str(&val.to_string());
                read_num += 1;
            }
            if *val == '"' {
                read_num += 1;
                break;
            }
        }

        (
            Token::new(TokenType::String(literal.clone()), None, cur, line),
            read_num,
        )
    }

    /// 数値リテラル取得
    ///
    /// # Arguments
    /// * `s` - 読み取り対象文字列（数値リテラルの開始時点からの配列）
    /// * `cur` - 文字列の読み取り位置
    /// * `line` - 行数
    ///
    /// # Return
    /// * (Token, usize) - 数値リテラルに対応するトークンと読み取り文字数のタプル
    fn number(&self, cur: usize, s: &[char], line: usize) -> (Token, usize) {
        let mut literal = String::new();
        let mut read_num = 0;
        for (i, val) in s.iter().enumerate() {
            match *val {
                // 小数点をカバー
                '0'..='9' | '.' if !self.end(i) => {
                    literal.push_str(&val.to_string());
                    read_num += 1;
                }
                _ => break,
            };
        }

        (
            Token::new(
                TokenType::Number(
                    literal
                        .parse::<f64>()
                        .expect("could not parse f64: {:literal?}"),
                ),
                None,
                cur,
                line,
            ),
            read_num,
        )
    }

    /// 識別子、予約語リテラル取得
    ///
    /// # Arguments
    /// * `s` - 読み取り対象文字列（識別子の開始時点からの配列）
    /// * `cur` - 文字列の読み取り位置
    /// * `line` - 行数
    ///
    /// # Return
    /// * (Token, usize) - 識別子に対応するトークンと読み取り文字数のタプル
    fn identifier(&self, cur: usize, s: &[char], line: usize) -> (Token, usize) {
        let mut literal = String::new();
        let mut read_num = 0;
        for (i, val) in s.iter().enumerate() {
            match *val {
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' | '.' if !self.end(i) => {
                    literal.push_str(&val.to_string());
                    read_num += 1;
                }
                _ => break,
            };
        }

        // 予約語チェック
        let keyword = String::from(&literal);
        let token_type = self.keywords.get(&keyword);
        token_type.map_or_else(
            || {
                (
                    Token::new(TokenType::Identifier(literal.clone()), None, cur, line),
                    read_num,
                )
            },
            |token_type| (Token::new(token_type.clone(), None, cur, line), read_num),
        )
    }

    /// 次の文字が期待する文字と一致しているか判定
    ///
    /// # Arguments
    /// * `s` - 読み取り対象文字列
    /// * `cur` - 文字列の読み取り位置
    /// * `expect` - 期待する文字
    ///
    /// # Return
    /// * bool - true: 一致 false: 不一致
    fn next_match(&self, s: &Vec<char>, cur: usize, e: char) -> bool {
        // 文字列読み取り判定
        if s.len() < 2 {
            return false;
        }

        let c = s[cur];
        c == e
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn 記号_scan() {
        let tokens = Scanner::new(&"(".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::Eof, None, 1, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"()".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::LeftParen, None, 0, 0),
            Token::new(TokenType::RightParen, None, 1, 0),
            Token::new(TokenType::Eof, None, 2, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&">".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Greater, None, 0, 0),
            Token::new(TokenType::Eof, None, 1, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"<".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Less, None, 0, 0),
            Token::new(TokenType::Eof, None, 1, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"=".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::Eof, None, 1, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"==".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::EqualEqual, None, 0, 0),
            Token::new(TokenType::Eof, None, 2, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&">=".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::GreaterEqual, None, 0, 0),
            Token::new(TokenType::Eof, None, 2, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"<=".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::LessEqual, None, 0, 0),
            Token::new(TokenType::Eof, None, 2, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"/".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Slash, None, 0, 0),
            Token::new(TokenType::Eof, None, 1, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"// comment\n/".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Slash, None, 11, 0),
            Token::new(TokenType::Eof, None, 12, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"!".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Bang, None, 0, 0),
            Token::new(TokenType::Eof, None, 1, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"!=".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::BangEqual, None, 0, 0),
            Token::new(TokenType::Eof, None, 2, 0),
        ];
        assert_eq!(expect, tokens);
    }

    #[test]
    fn 文字列リテラル_scan() {
        let tokens = Scanner::new(&"\"test\"".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::String("test".to_string()), None, 0, 0),
            Token::new(TokenType::Eof, None, 6, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"<=\"test\"".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::LessEqual, None, 0, 0),
            Token::new(TokenType::String("test".to_string()), None, 2, 0),
            Token::new(TokenType::Eof, None, 8, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"=\"test\"".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Equal, None, 0, 0),
            Token::new(TokenType::String("test".to_string()), None, 1, 0),
            Token::new(TokenType::Eof, None, 7, 0),
        ];
        assert_eq!(expect, tokens);
    }

    #[test]
    fn 数値リテラル_scan() {
        let tokens = Scanner::new(&"123".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Number(123.0), None, 0, 0),
            Token::new(TokenType::Eof, None, 3, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"123.123".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Number(123.123), None, 0, 0),
            Token::new(TokenType::Eof, None, 7, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"1 <= 2".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Number(1.0), None, 0, 0),
            Token::new(TokenType::LessEqual, None, 2, 0),
            Token::new(TokenType::Number(2.0), None, 5, 0),
            Token::new(TokenType::Eof, None, 6, 0),
        ];
        assert_eq!(expect, tokens);
    }

    #[test]
    fn 識別子リテラル_scan() {
        let tokens = Scanner::new(&"a".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Identifier("a".to_string()), None, 0, 0),
            Token::new(TokenType::Eof, None, 1, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"a_b".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Identifier("a_b".to_string()), None, 0, 0),
            Token::new(TokenType::Eof, None, 3, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"_a".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Identifier("_a".to_string()), None, 0, 0),
            Token::new(TokenType::Eof, None, 2, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"or_123".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Identifier("or_123".to_string()), None, 0, 0),
            Token::new(TokenType::Eof, None, 6, 0),
        ];
        assert_eq!(expect, tokens);
    }

    #[test]
    fn 予約語_scan() {
        let tokens = Scanner::new(&"and".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::And, None, 0, 0),
            Token::new(TokenType::Eof, None, 3, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"and123".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Identifier("and123".to_string()), None, 0, 0),
            Token::new(TokenType::Eof, None, 6, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"class".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Class, None, 0, 0),
            Token::new(TokenType::Eof, None, 5, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"else".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Else, None, 0, 0),
            Token::new(TokenType::Eof, None, 4, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"false".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::False, None, 0, 0),
            Token::new(TokenType::Eof, None, 5, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"for".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::For, None, 0, 0),
            Token::new(TokenType::Eof, None, 3, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"fun".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Fun, None, 0, 0),
            Token::new(TokenType::Eof, None, 3, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"if".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::If, None, 0, 0),
            Token::new(TokenType::Eof, None, 2, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"nil".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Nil, None, 0, 0),
            Token::new(TokenType::Eof, None, 3, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"or".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Or, None, 0, 0),
            Token::new(TokenType::Eof, None, 2, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"print".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Print, None, 0, 0),
            Token::new(TokenType::Eof, None, 5, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"super".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Super, None, 0, 0),
            Token::new(TokenType::Eof, None, 5, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"this".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::This, None, 0, 0),
            Token::new(TokenType::Eof, None, 4, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"true".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::True, None, 0, 0),
            Token::new(TokenType::Eof, None, 4, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"var".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Var, None, 0, 0),
            Token::new(TokenType::Eof, None, 3, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"var a = 1;".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::Var, None, 0, 0),
            Token::new(TokenType::Identifier("a".to_string()), None, 4, 0),
            Token::new(TokenType::Equal, None, 6, 0),
            Token::new(TokenType::Number(1.0), None, 8, 0),
            Token::new(TokenType::SemiColon, None, 9, 0),
            Token::new(TokenType::Eof, None, 10, 0),
        ];
        assert_eq!(expect, tokens);

        let tokens = Scanner::new(&"while".to_string()).scan();
        let expect = vec![
            Token::new(TokenType::While, None, 0, 0),
            Token::new(TokenType::Eof, None, 5, 0),
        ];
        assert_eq!(expect, tokens);
    }
}
