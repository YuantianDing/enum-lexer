
use crate::ast;
use regex_dfa_gen::{ 
    dfa::{ Dfa, DfaState},
    nfa::{ NfaBuilder},
    ast::{ AstNode, Error as RegexError},
};
use thiserror::Error;

use proc_macro2::{Span, TokenStream};
use quote::{ quote, format_ident};
use std::{ops::Range};
use syn::{LitInt, LitChar};

#[derive(Debug, Error)]
pub enum Error {
    #[error("regex parse error {0} at {1}")]
    RegexError(RegexError, String)
}

type Result<T> = std::result::Result<T, Error>;

fn uses() -> TokenStream {
    quote! {
        use super::*;
        use std::ops::Deref;
        use enum_lexer::Span;
        use enum_lexer::Cursor;
        use enum_lexer::SpanError;
        use enum_lexer::GroupError;

    }
}

fn type_definition(lexer: &ast::EnumLexer) -> TokenStream {
    let error_type = &lexer.error_type;
    let attrs = &lexer.attrs;

    let variants = 
        lexer.variants.iter()
        .map(|v| v.variant())
        .filter(|v| v.ident.to_string() != "COMMENTS");
    
    quote! {
        pub #error_type

        #(#attrs)*
        pub struct Token {
            pub inner: TokenInner,
            pub span: Span,
        }

        #(#attrs)*
        pub enum TokenInner {
            #( #variants ,)*
        }
        use TokenInner::*;

        // #[derive(Debug, Clone, Eq, PartialEq)]
        enum StateNext {
            Next(usize),
            Final(usize),
            End
        }

        struct ProtoToken{
            end_num: usize,
            text: Option<String>,
            span: Span,
        }

        pub type Result<T> = std::result::Result<T, LexError>;
        
        // #[derive(Debug, Clone)]
        pub struct TokenIterator<'a> {
            cursor: Cursor<'a>,
            // state: usize,
        }

        impl Deref for Token {
            type Target = TokenInner;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }
    }
}

fn get_dfa(lexer: ast::EnumLexer, test: bool) -> Result<(Vec<ast::LexerMap>, Dfa)> {
    let vec: Vec<_> = lexer.variants.into_iter().flat_map(|v| v.regex_maps()).collect();

    let asts: Result<Vec<AstNode>> = vec.iter().map(|m| 
        m.regex.value().parse::<AstNode>()
            .map_err(|e| Error::RegexError(e, m.regex.value()))
    ).collect();
    let asts = asts?;

    let mut nfabuilder = NfaBuilder::new();

    let nfa_nodes: Vec<_> = asts.iter().enumerate()
        .map(|(i, a)| {
            let ret = nfabuilder.from_ast(a);
            nfabuilder.set_end(&ret, i);
            ret
        }).collect();
    
    let nfa_node = nfabuilder.options(nfa_nodes);
    let nfa = nfabuilder.to_nfa(nfa_node);

    let dfa = Dfa::from_nfa(&nfa).opt();
    
    if test {
        let mut f = std::fs::File::create("dfa.dot").unwrap();
        dfa.render_to(&mut f).expect("msg");
    }
    
    
    Ok((vec, dfa))
}

fn dfa_arc(range: Range<char>, state: usize, is_greedy: bool, end_num: Option<usize>) -> TokenStream {
    let start = LitChar::new(range.start, Span::call_site());
    let end = LitChar::new(range.end, Span::call_site());

    if !is_greedy && end_num.is_some() {
        let end_num = end_num.unwrap();
        quote! {
            Some(ch @ #start..#end) => {
                Ok(StateNext::Final(#end_num))
            }
        }
    } else {
        quote! {
            Some(ch @ #start..#end) => {
                self.cursor.next();
                Ok(StateNext::Next(#state))
            }
        }
    }

}

fn dfa_state(i: usize, state: &DfaState) -> TokenStream {
    let fn_ident = format_ident!("dfa_state_{}", i);
    let DfaState{table, end_num} = state;

    let streams: _ = table.iter().map(|(range, i, is_greedy)| {
        dfa_arc(range.clone(), *i, *is_greedy, end_num.clone())
    });

    let others = if let Some(end_num) = end_num {
        let end_num = *end_num;
        quote!{ Ok(StateNext::Final( #end_num )) }
    } else { 
        quote!{
            let (_, span) = self.cursor.get_token();
            Err(SpanError(span).into())
        }
    };

    let end = if let Some(end_num) = end_num {
        let end_num = *end_num;
        quote!{ Ok(StateNext::Final( #end_num )) }
    } else { 
        quote!{ Ok(StateNext::End) }
    };

    quote! {
        #[inline(always)]
        fn #fn_ident (&mut self) -> Result<StateNext> {
            match self.cursor.peek().copied() {
                #( #streams )*
                None => { #end }
                _ => { #others }
            }
        }
    }
}

fn to_lit_int(i : usize) -> LitInt {
    let temp = format!("{}", i);
    LitInt::new(temp.as_str(), Span::call_site())
}


fn state_machine(maps: &Vec<ast::LexerMap>, dfa: &Dfa) -> TokenStream {
    let len = dfa.states.len();
    let states_num: Vec<_> = (0..len)
        .map(|i| to_lit_int(i))
        .collect();

    let funcs: Vec<_> = (0..len)
        .map(|i| format_ident!("dfa_state_{}", i))
        .collect();
    

    let state_arcs = quote! {
        #(#states_num => self.#funcs(),)*
    };

    let handlers: _ =  maps.iter().enumerate().map(|(i,m)| {
        let expr = m.expr.clone();
        let i = to_lit_int(i);
        quote! { #i => #expr, }
    });

    let states: _ = dfa.states.iter().enumerate().map(|(i, s)|{
        dfa_state(i, s)
    });

    quote! {
        impl<'a> TokenIterator<'a> {
            #( #states )*

            #[inline(always)]
            fn next_proto(&mut self) -> Option<Result<ProtoToken>> {
                let mut cur_state = 0;
                self.cursor.leap_until(|c| c != ' ' && c != '\n' && c != '\r' && c != '\t');
                let end_num = loop {
                    let result = match cur_state {
                        #state_arcs
                        _ => { panic!("Unexpected"); }
                    };
                    match result {
                        Ok(StateNext::Next(state)) => { cur_state = state; }
                        Ok(StateNext::Final(end_num)) => { break end_num; }
                        Ok(StateNext::End) => { return None; }
                        Err(e) => { return Some(Err(e)); }
                    };
                };
                let (text, span) = self.cursor.get_token();
                let text = Some(text);
                Some(Ok(ProtoToken{end_num, text, span}))
            }

            pub fn next_until(&mut self, num: usize) -> Result<Vec<Token>> {
                let mut vec = Vec::new();
                loop {
                    let proto = self.next_proto();
                    let (span, inner) = match proto {
                        Some(proto) => {
                            let mut proto = proto?;
                            if proto.end_num == num {
                                return Ok(vec);
                            }
                            let inner = proto.handlers(self);
                            (proto.span, inner)
                        }
                        None => { return Err(GroupError().into()); }
                    };
                    let inner = match inner? {
                        Some(inner) => inner,
                        None => { continue; }
                    };
                    vec.push(Token {inner, span});
                }
            }
        }
        impl ProtoToken {
            #[inline(always)]
            fn same_type(&self, other: &ProtoToken) -> bool {
                self.end_num == other.end_num
            }
            #[inline(always)]
            fn handlers(&mut self, iterator: &mut TokenIterator) -> Result<Option<TokenInner>> {
                
                let num = self.end_num;
                let mut read_group = {
                    || { iterator.next_until(num + 1)}
                };
                let text = self.text.take().unwrap();
                let inner = match self.end_num {
                    #( #handlers )*
                    _ => { panic!("Unexpected"); }
                };
                Ok(Some(inner))
            }
        }

        impl<'a> Iterator for TokenIterator<'a> {
            type Item = Result<Token>;
            fn next(&mut self) -> Option<Result<Token>> {
                use TokenInner::*;
                let proto = self.next_proto();
                let (span, inner) = match proto {
                    Some(Ok(mut proto)) => {
                        let inner = proto.handlers(self);
                        (proto.span, inner)
                    }
                    Some(Err(e)) => { return Some(Err(e)); },
                    None => { return None; }
                };
                let inner = match inner {
                    Ok(Some(inner)) => inner,
                    Ok(None) => { return self.next(); }
                    Err(e) => { return Some(Err(e)); }
                };
                Some(Ok(Token {inner, span}))
            }
        }
    }
}

// thread_local! {
//     static LEXERMAP: RefCell<Vec<ast::LexerMap>> = RefCell::new(Vec::new());
// }

pub fn generate(lexer: ast::EnumLexer, test: bool) -> Result<TokenStream> {
    let ident = lexer.ident.clone();
    let uses = uses();
    let type_definition = type_definition(&lexer);
    let (maps, dfa) = get_dfa(lexer, test)?;
    let state_machine = state_machine(&maps, &dfa);
    
    // LEXERMAP.with(|lm| {
    //     lm.replace(maps)
    // });
    Ok(quote! {
        #[allow(non_snake_case)]
        mod #ident {
            #uses

            #type_definition

            #state_machine

            pub fn parse_str<'a>(src: &'a str) -> Result<TokenIterator<'a>> {
                let cursor = Cursor::new_file("<string>", src);
                Ok(TokenIterator{ cursor })
            }

            pub fn parse_str_with_name<'a>(name: &str, src: &'a str) -> Result<TokenIterator<'a>> {
                let cursor = Cursor::new_file(name, src);
                Ok(TokenIterator{ cursor })
            }
        }
    })
}

// pub fn get_dfa_num(regex: &String) -> Option<usize> {
//     LEXERMAP.with(|lm| {
//         let lm = lm.borrow();
//         lm.iter().position(|lm|{
//             &lm.regex.value() == regex
//         })
//     })
// }