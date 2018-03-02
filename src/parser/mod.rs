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

//grammar has been changed.
//Original:
// <expr> ::= <opnd> <op> <opnd>
//         | [ <unary_op> ] <opnd>
//New:
// <expr> ::= <opnd> <op> <opnd>
//         |  <unary_op> <opnd>
//         |  <opnd>

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Binary(Operand, BinaryOperator, Operand),
    Unary(UnaryOperator, Operand),
    Singleton(Operand),
}

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
                _ => panic!("a statement cannot start with the keyword {:#?}", keyword),
            },
            Token::Semicolon => State(Self::normal_parse),

            _ => panic!("unexpected token: {:#?}", t),
        }
    }

    fn variable_definition_parse(&mut self, t: Token) -> State<'a, O> {
        
        State(Self::variable_definition_parse)
    }

    fn assignment_parse(&mut self, t: Token) -> State<'a, O> {
        if self.buffer.len() == 1 {
            match t {
                Token::Assignment => self.buffer.push(t),
                _ => panic!("expected a := but found {:#?} instead", t),
            }
            State(Self::assignment_parse)
        } else {
            match t {
                Token::Semicolon => {
                    match &self.buffer[0] {
                        &Token::Identifier(ref identifier) => {
                            self.statements.put(Statement::Assignment(identifier.clone(), Self::parse_expression(&self.buffer[2..])));
                        }
                        _ => unreachable!("the first token of the buffer during assignment parsing was something other than an identifier"),
                    }
                    State(Self::normal_parse)
                },
                Token::Bracket(_) | Token::Operator(_) | Token::Identifier(_) | Token::KeyWord(KeyWord::Int) | Token::KeyWord(KeyWord::String) => {
                    self.buffer.push(t);
                    State(Self::assignment_parse)
                },
                _ => panic!("unexpected Token {:#?} read during", t),
            }
        }
    }

    fn for_loop_parse(&mut self, t: Token) -> State<'a, O> {
        State(Self::for_loop_parse)
    }

    fn read_parse(&mut self, t: Token) -> State<'a, O> {
        State(Self::read_parse)
    }

    fn print_parse(&mut self, t: Token) -> State<'a, O> {
        State(Self::print_parse)
    }

    fn assert_parse(&mut self, t: Token) -> State<'a, O> {
        State(Self::assert_parse)
    }

    fn parse_expression(tokens: &[Token]) -> Expression {
        Expression::Singleton(Operand::Int(1.into()))
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