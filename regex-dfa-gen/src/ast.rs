//! Abstract Syntax Tree for Regex.
//! 
//! get AstNode by:
//! 
//! ```
//! use regex_dfa_gen::ast::AstNode;
//! let ast : AstNode = r"12".parse::<AstNode>().unwrap();
//! ```
//! 

use std::str::FromStr;
use crate::set::*;
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AstNode {
    Char(CharRange),
    Options(Vec<AstNode>),
    Multiple(Box<AstNode>),
    EmptyOr(Box<AstNode>),
    MultipleNonGreedy(Box<AstNode>),
    Concat(Vec<AstNode>),
    // NamedConcat(Vec<AstNode>, String),
}
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("missing expression at {0}")]
    MissingExpresion(usize),
    #[error("missing first expr at {0}")]
    MissingFirstExpr(usize),
    #[error("'^' is unusable at {0}")]
    ExceptNotUsable(usize),
    #[error("unmatched char '{1}' at {0}")]
    UnmatchedChar(usize, char),
    #[error("unexpect end at {0}")]
    UnexpectedEnd(usize),
    #[error("unexpected char '{1}' at {0}")]
    UnexpectedChar(usize, char),
    #[error("found an empty string")]
    EmptyString,
}

// impl std::error::Error for Error {}

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Error::MissingExpresion(pos) => { write!(f, , pos) }
//             Error::MissingFirstExpr(pos) => { write!(f, "missing first expr at {}", pos) }
//             Error::ExceptNotUsable(pos) => { write!(f, "'^' is unusable at {}", pos) }
//             Error::UnmatchedChar(pos, ch) => { write!(f, "unmatched char '{}' at {}", ch, pos) }
//             Error::UnexpectedEnd(pos) => { write!(f, "unexpect end at {}", pos) }
//             Error::UnexpectedChar(pos, ch) => { write!(f, "unexpected char '{}' at {}", ch, pos) }
//             Error::EmptyString => { write!(f, "found an empty string") }
//         }
//     }
// }

pub type Result<T> = std::result::Result<T,Error>;
pub trait CharStream: Iterator<Item=char> {}


/// LL1 Parser for Regex
/// 
/// ```c
/// Tree -> Option '|' ... '|' Option
/// Option -> Element ... Element
/// Element -> '(' Tree ')' | char | [char*] | '^'Element | Element'*'
/// ```
struct Parser<Iter : CharStream> {
    first : char,
    iter : Iter,
    pos: usize,
}


impl<Iter : CharStream> Parser<Iter> {
    /// create a new ll1 parser.
    pub fn new(mut iter: Iter) -> Result<Self> {
        Ok(Self {
            first: iter.next().ok_or(Error::EmptyString)?,
            iter,
            pos: 0,
        })
    }

    fn next(&mut self) -> char {
        self.first = self.iter.next().unwrap_or('\0');
        self.pos += 1;
        self.first
    }

    fn next_matches(&mut self, c : char) {
        trace!("{} == {}", self.first, c);
        self.first = self.iter.next().unwrap_or('\0');
        self.pos += 1;
    }
    /// read a tree in parser.(`Tree -> Option '|' ... '|' Option`)
    /// 
    /// `inside` mark whether it's inside a parentheses.
    pub fn parse_tree(&mut self, inside : bool) -> Result<AstNode> {
        trace!("parse_tree first:{}", self.first);
        let mut ret = Vec::<AstNode>::new();
        loop{
            let op = self.parse_option()?;
            ret.push(op);

            // FOLLOW(Tree) = '(' '\0'
            if self.first == ')' || self.first == '\0' {
                if self.first == ')' && !inside {
                    return Err(Error::UnmatchedChar(self.pos, ')'));
                }
                break;
            }

            self.next_matches('|');
        }

        if ret.len() == 0 {
            return Err(Error::MissingExpresion(self.pos));
        }

        trace!("parse_tree {:?}", ret);
        if ret.len() == 1 {
            Ok(ret.pop().unwrap())
        } else {
            Ok(AstNode::Options(ret))
        }
    }

    /// read an option in parser.(`Option -> Element ... Element`)
    pub fn parse_option(&mut self) -> Result<AstNode> {

        let mut ret = Vec::<AstNode>::new();
        loop {
            let element = self.parse_element()?;
            ret.push(element);
            // FOLLOW(Option) = '|' '\0'
            if self.first == '|' || self.first == '\0' || self.first == ')' {
                break;
            }

            // no self.next()
        }

        if ret.len() == 0 {
            return Err(Error::MissingExpresion(self.pos));
        }

        trace!("parse_option {:?}", ret);
        if ret.len() == 1 {
            Ok(ret.pop().unwrap())
        } else  {
            Ok(AstNode::Concat(ret))
        }
    }

    /// read an elemnt in parser.(`Element -> '(' Tree ')' | char | [char*] | '^'Element | Element'*'`)
    pub fn parse_element(&mut self) -> Result<AstNode> {

        let mut is_except = false;
        if self.first == '^'{
            is_except = true;
            self.next_matches('^');
        }

        let mut ret = match self.first {
            '(' => {
                if is_except {
                    return Err(Error::ExceptNotUsable(self.pos));
                }
                self.next_matches('('); // parse_tree known nothings about this '(' ')'
                let ret = self.parse_tree(true)?;
                self.next_matches(')');
                ret
            },
            '[' => {
                // parse_charset know about these '[' ']'
                let ret = self.parse_charset(is_except)?;
                ret
            },
            '.' => {
                if is_except {
                    return Err(Error::ExceptNotUsable(self.pos));
                }
                self.next_matches('.');
                AstNode::Char(CHAR_MIN..CHAR_MAX)
            }
            '\0' => { return Err(Error::UnexpectedEnd(self.pos));}
            ')' | ']' | '|' | '*' | '+' | '?'  => {
                return Err(Error::UnexpectedChar(self.pos, self.first));
            }
            '\\' => {
                let c = self.next();
                self.next();
                let c = match c {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    _ => c
                };
                AstNode::Char(c..add1(c))
            }
            c => {
                self.next();
                AstNode::Char(c..add1(c))
            }
        };

        if self.first == '*' {
            self.next_matches('*');
            if self.first == '?' {
                self.next_matches('?');
                ret = AstNode::MultipleNonGreedy(Box::new(ret));
            }else {
                ret = AstNode::Multiple(Box::new(ret));
            }
        }
        if self.first == '+' {
            self.next_matches('+');
            ret = AstNode::Concat(vec![
                ret.clone(),
                AstNode::Multiple(Box::new(ret))
            ])
        }
        if self.first == '?' {
            self.next_matches('?');
            ret = AstNode::EmptyOr(Box::new(ret));
        }
        trace!("parse_element {:?}", ret);
        Ok(ret)
    }

    fn parse_charset(&mut self, is_except: bool) -> Result<AstNode> {
        self.next_matches('[');
        let mut ret = Vec::<CharRange>::new();
        loop {
            match self.first {
                ']' => { 
                    self.next_matches(']');
                    break;
                },
                '-' => {
                    if let Some(range) = ret.pop() {
                        if add1(range.start) == range.end {
                            self.next();
                            ret.push(range.start..add1(self.first));
                        } else {
                            return Err(Error::MissingFirstExpr(self.pos));
                        }
                    } else {
                        return Err(Error::MissingFirstExpr(self.pos));
                    }
                }
                c => { ret.push(c..add1(c)) },
            }
            self.next();
        }
        trace!("parse_charset {:?}", ret);
        
        if ret.len() == 0{
            return Err(Error::MissingExpresion(self.pos));
        }

        if is_except {
            if ret.len() == 1 {
                let s = ret.pop().unwrap();
                return Ok(AstNode::Options(vec![
                    AstNode::Char(
                        CHAR_MIN..s.start
                    ),
                    AstNode::Char(
                        s.end..CHAR_MAX
                    ),
                ]))
            }
            panic!("not implemented!");
        }

        if ret.len() == 1 {
            Ok(AstNode::Char(ret.pop().unwrap()))
        } else {
            Ok(AstNode::Options(
                ret.iter().map(|c| AstNode::Char(c.clone())).collect()
            ))
        }
    }
}

impl CharStream for core::str::Chars<'_> {}

impl FromStr for AstNode {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let iter = s.chars();
        Parser::new(iter)?.parse_tree(false)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use std::assert;
    use AstNode::*;

    fn charnode(c : char) -> AstNode { Char(c..add1(c)) }
    fn charrange(c : char, d : char) -> AstNode {
        Char(c..add1(d))
    }

    fn multi(n : AstNode) -> AstNode { Multiple(Box::new(n)) }
    fn multi_non_greedy(n : AstNode) -> AstNode { MultipleNonGreedy(Box::new(n)) }



    #[test]
    fn basics() {
        let ast : AstNode = r"12".parse::<AstNode>().unwrap();
        assert_eq!(
            ast, Concat(vec![
                charnode('1'),
                charnode('2'),
            ])
        );

        let ast : AstNode = r"1|2".parse::<AstNode>().unwrap();
        assert_eq!(
            ast , Options(vec![
                charnode('1'),
                charnode('2'),
            ])
        );


        let ast : AstNode = r"1|2*3(5|4)*".parse::<AstNode>().unwrap();
        assert_eq!(
            ast , Options(vec![
                charnode('1'),
                Concat(vec![
                    multi(charnode('2')),
                    charnode('3'),
                    multi(
                        Options(vec![
                            charnode('5'),
                            charnode('4'),
                        ])
                    ),
                ])
            ])
        );

        let ast = r"1([1-9][1-9])*?".parse::<AstNode>().unwrap();
        assert_eq!(
            ast , Concat(vec![
                charnode('1'),
                multi_non_greedy(
                    Concat(vec![
                        charrange('1', '9'),
                        charrange('1', '9'),
                    ])
                )
            ])
        );

        let ast2 = r"1(([1-9]([1-9])))*?".parse::<AstNode>().unwrap();
        assert!(ast2 == ast);
    }
}