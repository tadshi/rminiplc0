use std::fs::File;
use std::io::{self, BufRead};
use std::ops::Deref;

use crate::error::{CompilationError, ErrorCode};

pub struct Tokenizer<'a> {
    filename: &'a str,
    initialized: bool,
    lines_buffer: Vec<String>,
    ptr: (usize, usize),
    state: DFAState,
}

impl<'a> Tokenizer<'a> {
    pub fn next_token(&mut self) -> Result<Token, CompilationError> {
        if !self.initialized {
            self.read_all();
        }
        self.next_token_unchecked()
    }
    fn next_token_unchecked(&mut self) -> Result<Token, CompilationError> {
        let mut current = DFAState::InitialState;
        let mut ss = String::new();
        let mut pos = (0, 0);
        loop {
            let current_char = self.next_char();
            match current {
                DFAState::InitialState => {
                    let ch = current_char.ok_or(CompilationError::new(0, 0, ErrorCode::ErrEOF))?;
                    let mut invalid = false;
                    if ch.is_whitespace() {
                        current = DFAState::InitialState;
                    } else if ch.is_ascii_graphic() {
                        invalid = true;
                    } else if ch.is_digit(10) {
                        current = DFAState::UnsignedIntegerState;
                    } else {
                        current = match ch {
                            '=' => DFAState::EqualSignState,
                            '-' => DFAState::MinusSignState,
                            '+' => DFAState::PlusSignState,
                            '*' => DFAState::MultiplicationSignState,
                            '/' => DFAState::DivisionSignState,
                            '(' => DFAState::LeftbracketState,
                            ')' => DFAState::RightbracketState,
                            ':' => DFAState::SemicolonState,
                            _ => {
                                invalid = true;
                                DFAState::InitialState
                            }
                        }
                    }
                    if !matches!(current, DFAState::InitialState) {
                        pos = self.previous_pos();
                    }
                    if invalid {
                        self.unread_last();
                        return Result::Err(CompilationError::new_packed(
                            pos,
                            ErrorCode::ErrInvalidInput,
                        ));
                    }
                    if !matches!(current, DFAState::InitialState) {
                        ss.push(ch);
                    }
                }
                DFAState::UnsignedIntegerState => {
                    if current_char.map_or(true, |ch| !ch.is_digit(10)) {
                        if current_char.is_some() {
                            self.unread_last();
                        }
                        return Ok(Token::Integer(
                            TokenType::UnsignedInteger,
                            ss.parse().map_err(|_| {
                                CompilationError::new_packed(
                                    self.ptr,
                                    ErrorCode::ErrInvalidIdentifier,
                                )
                            })?,
                            pos,
                            self.ptr,
                        ));
                    } else {
                        ss.push(current_char.unwrap());
                    }
                }
                DFAState::IdentifierState => {
                    if current_char.map_or(true, |ch| !ch.is_ascii_alphanumeric()) {
                        if current_char.is_some() {
                            self.unread_last();
                        }
                        return Ok(Token::Str(check_keyword(&ss), ss, pos, self.ptr));
                    } else {
                        ss.push(current_char.unwrap());
                    }
                }

                DFAState::PlusSignState => {
                    return Ok(Token::from_sign(TokenType::PlusSign, pos, self.ptr)?)
                }

                DFAState::MinusSignState => {
                    return Ok(Token::from_sign(TokenType::MinusSign, pos, self.ptr)?)
                }

                DFAState::MultiplicationSignState => {
                    return Ok(Token::from_sign(
                        TokenType::MultiplicationSign,
                        pos,
                        self.ptr,
                    )?)
                }

                DFAState::DivisionSignState => {
                    return Ok(Token::from_sign(TokenType::DivisionSign, pos, self.ptr)?)
                }

                DFAState::EqualSignState => {
                    return Ok(Token::from_sign(TokenType::EqualSign, pos, self.ptr)?)
                }

                DFAState::LeftbracketState => {
                    return Ok(Token::from_sign(TokenType::LeftBracket, pos, self.ptr)?)
                }

                DFAState::RightbracketState => {
                    return Ok(Token::from_sign(TokenType::RightBracket, pos, self.ptr)?)
                }

                DFAState::SemicolonState => {
                    return Ok(Token::from_sign(TokenType::Semicolon, pos, self.ptr)?)
                }
            }
        }
    }

    fn next_char(&mut self) -> Option<char> {
        if self.is_EOF() {
            return None;
        }
        let result: char = self.lines_buffer[self.ptr.0].as_bytes()[self.ptr.1].into();
        self.ptr = self.next_pos();
        Some(result)
    }

    fn previous_pos(&self) -> (usize, usize) {
        if self.ptr == (0, 0) {
            panic!("Unread from beginning!");
        }
        if self.ptr.1 == 0 {
            (self.ptr.0 - 1, self.lines_buffer[self.ptr.0 - 1].len() - 1)
        } else {
            (self.ptr.0, self.ptr.1 - 1)
        }
    }

    fn next_pos(&self) -> (usize, usize) {
        if self.ptr.0 >= self.lines_buffer.len() {
            panic!("Advance after EOF!");
        }
        if self.ptr.1 == self.lines_buffer.len() - 1 {
            return (self.ptr.0 + 1, 0);
        } else {
            return (self.ptr.0, self.ptr.1 + 1);
        }
    }

    fn read_all(&mut self) {
        if self.initialized {
            return;
        }
        let file = File::open(self.filename).expect("cannot find target file.");
        self.lines_buffer = io::BufReader::new(file)
            .lines()
            .map(|rl| rl.unwrap())
            .collect();
    }

    fn is_EOF(&self) -> bool {
        return self.ptr.0 >= self.lines_buffer.len();
    }

    fn unread_last(&mut self) {
        self.ptr = self.previous_pos();
    }
}

pub fn tokenize() {}

fn check_keyword(identifier: &str) -> TokenType {
    match identifier {
        "begin" => TokenType::Begin,
        "end" => TokenType::End,
        "const" => TokenType::Const,
        "var" => TokenType::Var,
        "print" => TokenType::Print,
        _ => TokenType::Identifier,
    }
}
enum TokenType {
    NullToken,
    UnsignedInteger,
    Identifier,
    Begin,
    End,
    Var,
    Const,
    Print,
    PlusSign,
    MinusSign,
    MultiplicationSign,
    DivisionSign,
    EqualSign,
    Semicolon,
    LeftBracket,
    RightBracket,
}

impl TokenType {
    pub fn to_string(&self) -> Result<String, CompilationError> {
        Ok(String::from(match self.deref() {
            TokenType::EqualSign => "=",
            TokenType::PlusSign => "+",
            TokenType::MinusSign => "-",
            TokenType::MultiplicationSign => "*",
            TokenType::DivisionSign => "/",
            TokenType::LeftBracket => "(",
            TokenType::RightBracket => ")",
            TokenType::Semicolon => ":",
            _ => return Err(CompilationError::new(0, 0, ErrorCode::ErrInvalidIdentifier)),
        }))
    }
}

enum Token {
    Integer(TokenType, u64, (usize, usize), (usize, usize)),
    Str(TokenType, String, (usize, usize), (usize, usize)),
}

impl Token {
    pub fn from_sign(
        token_type: TokenType,
        start: (usize, usize),
        end: (usize, usize),
    ) -> Result<Token, CompilationError> {
        let sign_string = token_type.to_string()?;
        Ok(Token::Str(token_type, sign_string, start, end))
    }
}

enum DFAState {
    InitialState,
    UnsignedIntegerState,
    PlusSignState,
    MinusSignState,
    DivisionSignState,
    MultiplicationSignState,
    IdentifierState,
    EqualSignState,
    SemicolonState,
    LeftbracketState,
    RightbracketState,
}
