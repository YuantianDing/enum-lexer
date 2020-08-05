mod lexer {
    use super::*;
    use enum_lexer::Cursor;
    use enum_lexer::Span;
    use enum_lexer::SpanError;
    use std::ops::Deref;
    type LexError = Box<dyn std::error::Error>;
    pub struct Token {
        pub inner: TokenInner,
        pub span: Span,
    }
    pub enum TokenInner {
        Ident(String),
        LitStr(String),
        LitInt(usize),
    }
    enum StateNext {
        Next(usize),
        Final(usize),
    }
    type Result<T> = std::result::Result<T, LexError>;
    pub struct TokenIterator<'a> {
        cursor: Cursor<'a>,
    }
    impl Deref for Token {
        type Target = TokenInner;
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<'a> TokenIterator<'a> {
        #[inline(always)]
        fn dfa_state_0(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{41}'..'\u{5b}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(1usize))
                }
                Some(ch @ '\u{61}'..'\u{7b}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(6usize))
                }
                Some(ch @ '\u{5f}'..'\u{60}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(7usize))
                }
                Some(ch @ '\u{22}'..'\u{23}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(8usize))
                }
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(11usize))
                }
                _ => {
                    let (_, span) = self.cursor.get_token();
                    Err(SpanError(span).into())
                }
            }
        }
        #[inline(always)]
        fn dfa_state_1(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{41}'..'\u{5b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{61}'..'\u{7b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{5f}'..'\u{60}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                _ => Ok(StateNext::Final(0usize)),
            }
        }
        #[inline(always)]
        fn dfa_state_2(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{41}'..'\u{5b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{61}'..'\u{7b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{5f}'..'\u{60}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                _ => Ok(StateNext::Final(0usize)),
            }
        }
        #[inline(always)]
        fn dfa_state_3(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{41}'..'\u{5b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{61}'..'\u{7b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{5f}'..'\u{60}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                _ => Ok(StateNext::Final(0usize)),
            }
        }
        #[inline(always)]
        fn dfa_state_4(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{41}'..'\u{5b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{61}'..'\u{7b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{5f}'..'\u{60}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                _ => Ok(StateNext::Final(0usize)),
            }
        }
        #[inline(always)]
        fn dfa_state_5(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{41}'..'\u{5b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{61}'..'\u{7b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{5f}'..'\u{60}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                _ => Ok(StateNext::Final(0usize)),
            }
        }
        #[inline(always)]
        fn dfa_state_6(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{41}'..'\u{5b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{61}'..'\u{7b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{5f}'..'\u{60}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                _ => Ok(StateNext::Final(0usize)),
            }
        }
        #[inline(always)]
        fn dfa_state_7(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{41}'..'\u{5b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{61}'..'\u{7b}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{5f}'..'\u{60}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(0usize))
                }
                _ => Ok(StateNext::Final(0usize)),
            }
        }
        #[inline(always)]
        fn dfa_state_8(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{0}'..'\u{22}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(9usize))
                }
                Some(ch @ '\u{22}'..'\u{23}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(10usize))
                }
                Some(ch @ '\u{23}'..'\u{7f}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(9usize))
                }
                _ => {
                    let (_, span) = self.cursor.get_token();
                    Err(SpanError(span).into())
                }
            }
        }
        #[inline(always)]
        fn dfa_state_9(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{0}'..'\u{22}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(9usize))
                }
                Some(ch @ '\u{22}'..'\u{23}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(10usize))
                }
                Some(ch @ '\u{23}'..'\u{7f}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(9usize))
                }
                _ => {
                    let (_, span) = self.cursor.get_token();
                    Err(SpanError(span).into())
                }
            }
        }
        #[inline(always)]
        fn dfa_state_10(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{0}'..'\u{22}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(1usize))
                }
                Some(ch @ '\u{22}'..'\u{23}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(1usize))
                }
                Some(ch @ '\u{23}'..'\u{7f}') => {
                    self.cursor.next();
                    Ok(StateNext::Final(1usize))
                }
                _ => Ok(StateNext::Final(1usize)),
            }
        }
        #[inline(always)]
        fn dfa_state_11(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(12usize))
                }
                _ => Ok(StateNext::Final(2usize)),
            }
        }
        #[inline(always)]
        fn dfa_state_12(&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                Some(ch @ '\u{30}'..'\u{3a}') => {
                    self.cursor.next();
                    Ok(StateNext::Next(12usize))
                }
                _ => Ok(StateNext::Final(2usize)),
            }
        }
    }
    #[inline(always)]
    fn handlers(end_num: usize, text: String, span: Span) -> Result<TokenInner> {
        let inner = match end_num {
            0 => Ident(text),
            1 => LitStr(text),
            2 => LitInt(text.parse::<usize>()?),
            _ => {
                panic!("Unexpected");
            }
        };
        Ok(inner)
    }
    impl<'a> Iterator for TokenIterator<'a> {
        type Item = Result<Token>;
        fn next(&mut self) -> Option<Result<Token>> {
            use TokenInner::*;
            let mut cur_state = 0;
            self.cursor
                .leap_until(|c| c != ' ' && c != '\n' && c != '\r' && c != '\t');
            let end_num = loop {
                let result = match cur_state {
                    0 => self.dfa_state_0(),
                    1 => self.dfa_state_1(),
                    2 => self.dfa_state_2(),
                    3 => self.dfa_state_3(),
                    4 => self.dfa_state_4(),
                    5 => self.dfa_state_5(),
                    6 => self.dfa_state_6(),
                    7 => self.dfa_state_7(),
                    8 => self.dfa_state_8(),
                    9 => self.dfa_state_9(),
                    10 => self.dfa_state_10(),
                    11 => self.dfa_state_11(),
                    12 => self.dfa_state_12(),
                    _ => {
                        panic!("Unexpected");
                    }
                };
                match result {
                    Ok(StateNext::Next(state)) => {
                        cur_state = state;
                    }
                    Ok(StateNext::Final(end_num)) => {
                        break cur_state;
                    }
                    Err(e) => {
                        return Some(Err(e));
                    }
                };
            };
            let (text, span) = self.cursor.get_token();
            let inner = match handlers(end_num, text, span) {
                Ok(inner) => inner,
                Err(e) => Some(Err(e)),
            };
            Some(Ok(Token { inner, span }))
        }
    }
    pub fn parse_str<'a>(src: &'a str) -> Result<TokenIterator<'a>> {
        let cursor = Cursor::new_file("<string>", src);
        Ok(TokenIterator { cursor })
    }
    pub fn parse_str_with_name<'a>(name: &str, src: &'a str) -> Result<TokenIterator<'a>> {
        let cursor = Cursor::new_file(name, src);
        Ok(TokenIterator { cursor })
    }
}
