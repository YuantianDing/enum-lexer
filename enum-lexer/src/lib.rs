//! # Enum Lexer
//! 
//! A proc_macro lexer generator. using `enum`-like syntax.
//! 
//! ## Write a lexer
//! 
//! ```no_run
//! #![feature(exclusive_range_pattern)]
//!
//! use enum_lexer::enum_lexer;
//!
//! enum_lexer! {
//!     #[derive(Debug, Eq, PartialEq)]
//!     enum lexer {
//!         Ident(String): {
//!             r"[A-Za-z_][A-Za-z_0-9]*" => Ident(text),
//!         }
//!         LitInt(usize): {
//!             r"[0-9][0-9]*" =>
//!                 LitInt(text.parse::<usize>()?), // default error type is Box<dyn Error>
//!         }
//!         Op(char): {
//!             r"\+" => Op('+'),
//!             r"\-" => Op('-'),
//!         }
//!         Def: r"def",
//!         Let: r"let",
//!         Group(Vec<Token>, char) : {
//!             r"\(" => {
//!                 Group(read_group()?, '(')       // construct a token tree within '(', ')'.
//!             }
//!             r"\)" => { panic!("error") }
//!         }
//!         COMMENTS: {                             // COMMENTS will be ignored
//!             r"//.*?\n" => !,
//!             r"/\*.*?\*/" => !,
//!         }
//!     }
//! }
//! ```
//! 
//! This will generate struct and enum like:
//! 
//! ```ignore
//! mod lexer {
//!      #[derive(Debug, Eq, PartialEq)]
//!      pub struct Token {
//!          pub inner: TokenInner,
//!          pub span: Span,
//!      }
//!      
//!      #[derive(Debug, Eq, PartialEq)]
//!      pub enum TokenInner {
//!          Ident(String),
//!          LitInt(usize),
//!          Op(char),
//!          Def,
//!          Let,
//!          Group(Vec<Token>, char),
//!      }
//!      pub struct TokenIterator{...}
//!      pub type LexError = Box<&dyn Error>;
//!      pub fn parse_str(src: &str) -> Result<TokenIterator>;
//! }
//! ```
//! ## Usage
//! 
//! ```ignore
//! let vec: lexer::Result<Vec<_>> =
//!     lexer::parse_str(r#"
//!         let a = 10 + (1 + 2) // alpha
//!     "#).unwrap().collect();
//! 
//! println!("{:?}", vec);
//! ```
//! 
//! ## Customizing Error Types
//! 
//! ```ignore
//! enum_lexer! {
//!     type LexError = MyError;
//!     enum lexer {
//!         LitStr: "\".*?\""
//!     }
//! }
//! ```
//! 

mod cursor;

pub use enum_lexer_macro::enum_lexer;

pub use enum_lexer_macro::enum_lexer_test;

pub use cursor::*;

use std::{ fmt };

#[derive(Debug, Clone)]
pub struct SpanError(pub Span);

impl std::error::Error for SpanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lexer error at {:?}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct GroupError();

impl std::error::Error for GroupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for GroupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "brace not match")
    }
}