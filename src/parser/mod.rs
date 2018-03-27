use std::ops::Deref;

use num_bigint::BigInt;

use util::{Direction, Sink, Source};

use scanner::{KeyWord, Token};

//All of these enums make up our AST.

//  <stmt> ::=
//    "var" <var_ident> ":" <type> [ ":=" <expr> ]
//  | <var_ident> ":=" <expr>
//  | "for" <var_ident> "in" <expr> ".." <expr> "do" <stmts> "end" "for"
//  | "read" <var_ident>
//  | "print" <expr>
//  | "assert" "(" <expr> ")"
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Declaration(String, Type, Option<Expression>),
    Assignment(String, Expression),
    For(String, Expression, Expression, Vec<Statement>),
    Read(String),
    Print(Expression),
    Assert(Expression),
}

// grammar has been changed.
// Original:
// <expr> ::= <opnd> <op> <opnd>
//         | [ <unary_op> ] <opnd>
// New:
// <expr> ::= <opnd> <op> <opnd>
//         |  <unary_op> <opnd>
//         |  <opnd>

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Binary(Operand, BinaryOperator, Operand),
    Unary(UnaryOperator, Operand),
    Singleton(Operand),
}

// <opnd> ::=
//   <int>
// | <string>
// | <var_ident>
// | "(" expr ")"
#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    Int(BigInt),
    StringLiteral(String),
    Bool,
    Expr(Box<Expression>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    LessThan,
    Equals,
    And,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnaryOperator {
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    Str,
    Bool,
}

pub struct Parser<'a, O>
where
    O: Sink<Statement> + 'a,
{
    buffer: Vec<Token>,
    for_buffer: Vec<(String, Expression, Expression, Vec<Statement>)>,
    for_range_pointer: usize,
    statements: &'a mut O,
}

pub fn parse<I, O>(tokens: &mut I, statements: &mut O)
where
    I: Source<Token>,
    O: Sink<Statement>,
{
    let mut parser = Parser::new(statements);
    let mut state = State(Parser::normal_parse);
    while let Some(t) = tokens.take() {
        state = state(&mut parser, t);
    }
}

//  <prog> ::= <stmts>
//  <stmts> ::= <stmt> ";" ( <stmt> ";" )*
//  <stmt> ::= "var" <var_ident> ":" <type> [ ":=" <expr> ]
//  | <var_ident> ":=" <expr>
//  | "for" <var_ident> "in" <expr> ".." <expr> "do"
//  <stmts> "end" "for"
//  | "read" <var_ident>
//  | "print" <expr>
//  | "assert" "(" <expr> ")"
//  <expr> ::= <opnd> <op> <opnd>
//  | [ <unary_op> ] <opnd>
//  <opnd> ::= <int>
//  | <string>
//  | <var_ident>
//  | "(" expr ")"
//  <type> ::= "int" | "string" | "bool"
//  <var_ident> ::= <ident>
//  <reserved keyword> ::=
//  "var" | "for" | "end" | "in" | "do" | "read" |
//  "print" | "int" | "string" | "bool" | "assert"
// I tried the design pattern described here
// https://dev.to/mindflavor/lets-build-zork-using-rust-1opm
impl<'a, O> Parser<'a, O>
where
    O: Sink<Statement>,
{
    fn new(statements: &'a mut O) -> Self {
        Parser {
            buffer: Vec::new(),
            for_buffer: Vec::new(),
            for_range_pointer: 0,
            statements,
        }
    }

    fn normal_parse(&mut self, t: Token) -> State<'a, O> {
        match t {
            Token::Identifier(_) => {
                self.buffer.push(t);
                State(Self::assignment_parse)
            }
            Token::KeyWord(keyword) => match keyword {
                KeyWord::Var => State(Self::variable_definition_parse),
                KeyWord::For => State(Self::for_loop_parse),
                KeyWord::Read => State(Self::read_parse),
                KeyWord::Print => State(Self::print_parse),
                KeyWord::Assert => State(Self::assert_parse),
                KeyWord::End => State(Self::expect_end_for),
                _ => panic!("a statement cannot start with the keyword {:#?}", keyword),
            },
            //empty statements are allowed. They are skiped.
            Token::Semicolon => State(Self::normal_parse),

            _ => panic!("unexpected token: {:#?}", t),
        }
    }

    // "var" <var_ident> ":" <type> [ ":=" <expr> ]
    fn variable_definition_parse(&mut self, t: Token) -> State<'a, O> {
        match self.buffer.len() {
            0 => match t {
                Token::Identifier(_) => self.buffer.push(t),
                _ => panic!("Expected an identifier but found {:#?} instead", t),
            },
            1 => match t {
                Token::Colon => self.buffer.push(t),
                _ => panic!("Expected a colon but found {:#?} instead", t),
            },
            2 => match t {
                Token::KeyWord(KeyWord::String) | Token::KeyWord(KeyWord::Int) => self.buffer.push(t),
                _ => panic!("Expected a type signature but found {:#?} instead", t),
            },
            _ => {
                
            },
        }
        State(Self::variable_definition_parse) 
    }

    fn assignment_parse(&mut self, t: Token) -> State<'a, O> {
        //let len = self.buffer.len();
        if self.buffer.len() == 1 {
            match t {
                Token::Assignment => self.buffer.push(t),
                _ => panic!("expected a := but found {:#?} instead", t),
            }
            State(Self::assignment_parse)
        } else {
            match t {
                Token::Semicolon => {
                    let statement = match &self.buffer[0] {
                        &Token::Identifier(ref identifier) => {
                            Statement::Assignment(
                                    identifier.clone(),
                                    Self::parse_expression(&self.buffer[2..])
                            )
                        },
                        _ => unreachable!(
                            "the first token of the buffer during assignment parsing was something other than an identifier"
                        ),
                    };
                    self.handle_statement(statement);
                    State(Self::normal_parse)
                }
                Token::Bracket(_)
                | Token::Operator(_)
                | Token::Identifier(_)
                | Token::Number(_)
                | Token::StringLiteral(_) => {
                    self.buffer.push(t);
                    State(Self::assignment_parse)
                }
                _ => panic!("unexpected Token {:#?} read during", t),
            }
        }
    }

    // "for" <var_ident> "in" <expr> ".." <expr> "do" <stmts> "end" "for"
    fn for_loop_parse(&mut self, t: Token) -> State<'a, O> {
        match self.buffer.len() {
            0 => match t {
                Token::Identifier(_) => self.buffer.push(t),
                _ => panic!("Expected an identifier, found {:#?}", t),
            },
            1 => match t {
                Token::KeyWord(KeyWord::In) => self.buffer.push(t),
                _ => panic!("Expected keyword 'in', found {:#?}"),
            },
            _ => match t {
                Token::KeyWord(KeyWord::Do) => {
                    if self.for_range_pointer < 3 {
                        panic!("incorrect for loop range");
                    }
                    let identifier = match self.buffer[0] {
                        Token::Identifier(ref i) => i.clone(),
                        _ => unreachable!("the buffer did not have an identifier as the first element when parsing a for loop"),
                    };
                    self.for_buffer.push((
                        identifier,
                        Self::parse_expression(&self.buffer[2..self.for_range_pointer]),
                        Self::parse_expression(
                            &self.buffer[(self.for_range_pointer + 1)..self.buffer.len()],
                        ),
                        Vec::new(),
                    ));
                    self.for_range_pointer = 0;
                    self.buffer.clear();
                    return State(Self::normal_parse);
                }
                Token::Range => {
                    if self.for_range_pointer == 0 {
                        self.for_range_pointer = self.buffer.len();
                        self.buffer.push(t);
                    } else {
                        panic!("found more than one range during for loop parsing");
                    }
                }
                Token::Bracket(_)
                | Token::Operator(_)
                | Token::Identifier(_)
                | Token::Number(_)
                | Token::StringLiteral(_) => {
                    self.buffer.push(t);
                }
                _ => {
                    panic!("error parsing a for loop: {:#?} is not a valid token in an expression")
                }
            },
        }
        State(Self::for_loop_parse)
    }

    fn expect_end_for(&mut self, t: Token) -> State<'a, O> {
        match t {
            Token::KeyWord(KeyWord::For) => {
                let (identifier, from, to, statements) = self.for_buffer
                    .pop()
                    .expect("encountered an end for but no for loops were initialized.");

                let for_statement = Statement::For(identifier, from, to, statements);

                self.handle_statement(for_statement);
            }
            _ => panic!("Expected end after for, found {:#?} instead", t),
        };
        State(Self::expect_semicolon)
    }

    // "read" <var_ident>
    fn read_parse(&mut self, t: Token) -> State<'a, O> {
        match t {
            Token::Identifier(i) => self.handle_statement(Statement::Read(i)),
            _ => panic!("expected an identifier after a read"),
        };
        State(Self::normal_parse)
    }

    // "print" <expr>
    fn print_parse(&mut self, t: Token) -> State<'a, O> {
        State(Self::print_parse)
    }

    fn assert_parse(&mut self, t: Token) -> State<'a, O> {
        State(Self::assert_parse)
    }

    fn parse_expression(tokens: &[Token]) -> Expression {
        Expression::Singleton(Operand::Int(1.into()))
    }

    fn handle_statement(&mut self, statement: Statement) {
        if self.for_buffer.is_empty() {
            self.statements.put(statement);
        } else {
            let len = self.for_buffer.len()-1;
            self.for_buffer[len].3.push(statement);
        }
        self.buffer.clear();
    }

    fn expect_semicolon(&mut self, t: Token) -> State<'a, O> {
        match t {
            Token::Semicolon => State(Self::normal_parse),
            _ => panic!("expected a semicolon, found {:#?} instead", t),
        }
    } 
}

struct State<'a, O>(fn(&mut Parser<'a, O>, Token) -> State<'a, O>)
where
    O: Sink<Statement> + 'a;
impl<'a, O> Deref for State<'a, O>
where
    O: Sink<Statement>,
{
    type Target = fn(&mut Parser<'a, O>, Token) -> State<'a, O>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
