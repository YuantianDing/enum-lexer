#![feature(exclusive_range_pattern)]

use enum_lexer::{
    enum_lexer
};



enum_lexer! {
    #[derive(Debug, Eq, PartialEq)]
    enum lexer {
        Ident(String) : {
            r"[A-Za-z_][A-Za-z_0-9]*" => Ident(text),
        }
        LitStr(String) : {
            "\".*?\"" => LitStr(text),
        }
        LitInt(usize) : {
            r"[0-9][0-9]*" => LitInt(text.parse::<usize>()?),
        } 
        Def: r"def",
        Let: r"let",
        Op(char) : {
            r"\+" => Op('+'),
            r"\-" => Op('-'),
            r"=" => Op('='),
            r"!" => Op('!'),
        }
        Group(Vec<TokenInner>, char) : {
            r"\(" => {
                Group(
                    read_group()?.into_iter().map(|t| t.inner).collect(),
                    '('
                )
            }
            r"\)" => { panic!("error") }
        }
        COMMENTS: {
            r"//.*?\n" => !,
            r"/\*.*?\*/" => !,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::assert_eq;
    use lexer::TokenInner::*;
    
    #[test]
    fn lexer_test() {
        let vec: lexer::Result<Vec<_>> = lexer::parse_str(
            r#"
            let a = "asdf" + (1 + 2) // alpha
            /* asdf asdf */
            "#
            ).unwrap()
            .map(|result| result.map(|t| t.inner))
            .collect();
        let vec = vec.unwrap();
        assert_eq!{
            vec,
            vec![Let, Ident("a".into()), Op('='), LitStr("\"asdf\"".into()), Op('+'),
            Group(vec![
                LitInt(1), Op('+'), LitInt(2)
            ], '(')]
        }
    }
}