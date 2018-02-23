//     mini-pl compiler.
//     Copyright (C) 2018  Victor Bankowski

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::char::from_u32;

use num_bigint::BigInt;

use util::Direction;
use util::Direction::*;

/// All the different tokens mini-pl has.
#[derive(Clone)]
pub enum Token {
    Bracket(Direction),
    Identifier(String),
    StringLiteral(String),
    Number(BigInt),
    Semicolon,
    Colon,
    Assignment,
    Operator(Operator),
    KeyWord(KeyWord),
    Range,
}

/// All the different operators mini-pl has.
#[derive(Clone)]
enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    LessThan,
    Equals,
    And,
    Not,
}

///All the keywords mini-pl has.
#[derive(Clone)]
enum KeyWord {
    Var,
    For,
    End,
    In,
    Do,
    Read,
    Print,
    Int,
    String,
    Bool,
    Assert,
}
/// ScanModes can be thought as parts of an finite automaton that handle recognizing different token types.
enum ScanMode {
    Normal,
    StringLiteral,
    Number,
    PossibleComment,
    LineComment,
    BlockComment,
    Other,
    Escape,
}

/// Scanner is essentially a finite state automaton that takes in a source code as a string and
pub struct Scanner {
    /// Tokens that have been parsed.
    tokens: Vec<Token>,
    /// Current state of scanning. It used to choose the approriate function to scan for a token.
    scan_mode: ScanMode,
    /// a String used to store previously scanned characters that are needed in the next token.
    buffer: String,
    /// a String used to store characters related to escape sequences (in strings).
    escape_buffer: String,

    block_comment_counter: usize,
}

impl Scanner {
    /// Creates a new Scanner.
    pub fn new() -> Self {
        Scanner {
            tokens: Vec::new(),
            scan_mode: ScanMode::Normal,
            buffer: String::new(),
            escape_buffer: String::new(),
            block_comment_counter: 0,
        }
    }
    /// Goes trough the whole source string character by character and produces a vector of tokens.
    pub fn scan(&mut self, source: &str) -> Vec<Token> {
        use self::ScanMode::*;
        // Foreach through the source string and choose the approriate handling function for the current character
        // according to what state(´ScanMode´) the scanner is currently in.
        for c in source.chars() {
            match self.scan_mode {
                Normal => self.normal_scan(c),
                StringLiteral => self.string_scan(c),
                Number => self.number_scan(c),
                PossibleComment => self.check_for_comment(c),
                LineComment => self.line_comment_handling(c),
                BlockComment => self.block_comment_handling(c),
                Other => self.identifier_and_keyword_scan(c),
                Escape => self.escape_scan(c),
            }
        }
        self.tokens.clone()
    }

    fn normal_scan(&mut self, c: char) {
        self.tokens.push(match c {
            // With these characters we return the corresponding Token from the match to be pushed into the token stream.
            '(' => Token::Bracket(Left),
            ')' => Token::Bracket(Right),
            ';' => Token::Semicolon,
            ':' => Token::Colon,
            '+' => Token::Operator(Operator::Plus),
            '-' => Token::Operator(Operator::Minus),
            '*' => Token::Operator(Operator::Multiply),
            '<' => Token::Operator(Operator::LessThan),
            '=' => Token::Operator(Operator::Equals),
            '&' => Token::Operator(Operator::And),
            '!' => Token::Operator(Operator::Not),

            // In the case of these characters, we don't want to insert a token into our token stream.
            // Instead we choose the approriate scanning mode, possibly push the current character into our buffer
            // for later use and then do an early return from the function in order to proceed to the next character.
            '"' => {
                self.scan_mode = ScanMode::StringLiteral;
                return;
            }
            '/' => {
                self.scan_mode = ScanMode::PossibleComment;
                return;
            }
            '0'...'9' => {
                self.buffer.push(c);
                self.scan_mode = ScanMode::Number;
                return;
            }
            ' ' | '\n' | '\t' | '\r' => return,
            _ => {
                self.buffer.push(c);
                self.scan_mode = ScanMode::Other;
                return;
            }
        });
    }

    fn string_scan(&mut self, c: char) {
        match c {
            // Escapes need to be handled in their own mode since they are transformed into
            // their corresponding character and then inserted into the string we're reading.
            '\\' => {
                self.scan_mode = ScanMode::Escape;
            }

            // The string literal has ended. We create a token out of the string we've built
            // into our buffer and then return to normal scanning mode.
            '"' => {
                self.tokens.push(Token::StringLiteral(self.buffer.clone()));
                self.buffer.clear();
                self.scan_mode = ScanMode::Normal;
            }
            //TODO: It is possible that I need to add problematic characters here. 
            //      For example, currently multiline string literals are allowed.

            //The character does not have a special meaning and is just added to the string we're building.
            _ => self.buffer.push(c),
        }
    }

    fn escape_scan(&mut self, c: char) {
        if self.escape_buffer.is_empty() {
            //match the escape to the actual character and store it in a variable.
            let escaped_char = match c {
                'a' => '\x07',
                'b' => '\x08',
                'f' => '\x0C',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'v' => '\x0B',
                '\\' | '\'' | '"' | '?' => c,
                '0'...'8' | 'x' | 'U' | 'u' => {
                    self.escape_buffer.push(c);
                    //return because in the case these characters we want to gather more characters in order to parse the escape correctly.
                    return;
                }
                _ => panic!("Escape \\{} not supported", c),
            };
            //the escape has been handled. push the character into the string we're forming and return back to normal string scanning.
            self.buffer.push(escaped_char);
            self.escape_buffer.clear();
            self.scan_mode = ScanMode::StringLiteral;
        } else {
            //we have found an escape sequence that's larger than one character long.
            match self.escape_buffer.chars().next().unwrap() {
                //hexadecimal escape handling
                'x' => match c {
                    '0'...'9' | 'a'...'f' | 'A'...'F' => {
                        //
                        if self.escape_buffer.len() > 2 {
                            panic!("hex escape sequences longer than a byte(two digits) are not supported.");
                        }
                        self.escape_buffer.push(c);
                    }
                    _ => {
                        if self.escape_buffer.len() < 2 {
                            panic!("Atleast one hexadecimal digit is needed after \\x");
                        }
                        let chr = u8::from_str_radix(&self.escape_buffer[1..], 16)
                            .expect("error occured when parsing the hex escape into a char");
                        self.buffer.push(chr as char);
                        self.escape_buffer.clear();
                        if c == '"' {
                            self.tokens.push(Token::StringLiteral(self.buffer.clone()));
                            self.buffer.clear();
                            self.scan_mode = ScanMode::Normal;
                        } else {
                            self.scan_mode = ScanMode::StringLiteral;
                        }
                    }
                },
                // Unicode escapes.
                // \U is a 4 byte unicode escape sequence and is represented as an 8 digit hexadecimal number.
                // \u is a 2 byte unicode escape sequence and is represented as an 4 digit hexadecimal number.
                u @ 'U' | u @ 'u' => {
                    match c {
                        '0'...'9' | 'a'...'f' | 'A'...'F' => self.escape_buffer.push(c),
                        _ => {
                            if u == 'U' {
                                panic!("{} is not a valid hex digit. \\U requires eight hex digits after it.",c);
                            } else {
                                panic!("{} is not a valid hex digit. \\u requires four hex digits after it.", c);
                            }
                        }
                    }
                    let max_buffer_len = if u == 'U' { 8 } else { 4 } + 1;
                    if self.escape_buffer.len() == max_buffer_len {
                        let error_string = format!(
                            "error occured when parsing unicode escape sequence {}",
                            self.escape_buffer
                        );
                        let unicode_error =
                            format!("{} is an invalid unicode codepoint.", self.escape_buffer);
                        let chr = from_u32(
                            u32::from_str_radix(&self.escape_buffer[1..], 16).expect(&error_string),
                        ).expect(&unicode_error);
                        self.buffer.push(chr as char)
                    }
                }

                _ => unreachable!("Unsupported multichar escape sequence."),
            }
        }
    }

    fn number_scan(&mut self, c: char) {
        match c {
            '0'...'9' => self.buffer.push(c),
            _ => {
                self.tokens.push(
                    self.buffer
                        .parse()
                        .map(Token::Number)
                        .unwrap_or_else(|e| panic!("Number parsing failed: {}", e)),
                );
                self.buffer.clear();
                self.scan_mode = ScanMode::Normal;
                self.normal_scan(c);
            }
        }
    }

    fn eval_buffer(&mut self) {
        self.tokens.push(match &*self.buffer {
            "var" => Token::KeyWord(KeyWord::Var),
            "end" => Token::KeyWord(KeyWord::End),
            "for" => Token::KeyWord(KeyWord::For),
            "in" => Token::KeyWord(KeyWord::In),
            "do" => Token::KeyWord(KeyWord::Do),
            "read" => Token::KeyWord(KeyWord::Read),
            "print" => Token::KeyWord(KeyWord::Print),
            "int" => Token::KeyWord(KeyWord::Int),
            "string" => Token::KeyWord(KeyWord::String),
            "bool" => Token::KeyWord(KeyWord::Bool),
            "assert" => Token::KeyWord(KeyWord::Assert),
            _ => Token::Identifier(self.buffer.clone()),
        });
    }

    fn identifier_and_keyword_scan(&mut self, c: char) {
        if c.is_alphanumeric() || c == '_' {
            self.buffer.push(c);
        } else {
            self.eval_buffer();
            self.scan_mode = ScanMode::Normal;
            self.normal_scan(c);
        }
    }

    fn check_for_comment(&mut self, c: char) {
        match c {
            '/' => self.scan_mode = ScanMode::LineComment,
            '*' => {
                self.block_comment_counter += 1;
                self.scan_mode = ScanMode::BlockComment;
            }
            _ => {
                self.tokens.push(Token::Operator(Operator::Divide));
                self.scan_mode = ScanMode::Normal;
            }
        }
    }

    fn block_comment_handling(&mut self, c: char) {
        if c == '*' || c == '/' {
            let b = self.buffer.pop();
            if b != Some(c) {
                self.buffer.extend(b);
                self.buffer.push(c);
            }
            if self.buffer == "/*" {
                self.block_comment_counter += 1;
                self.buffer.clear();
            } else if self.buffer == "*/" {
                self.block_comment_counter -= 1;
                self.buffer.clear();
                if self.block_comment_counter == 0 {
                    self.scan_mode = ScanMode::Normal;
                }
            }
        } else {
            self.buffer.clear();
        }
    }

    fn line_comment_handling(&mut self, c: char) {
        if c == '\n' {
            self.scan_mode = ScanMode::Normal;
        }
    }
}
