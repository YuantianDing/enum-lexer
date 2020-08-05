use proc_macro2::{Ident};
use syn;
use syn::parse::ParseStream;
use syn::token;
use syn::{Result};
use std::fmt;

#[derive(Clone)]
pub struct EnumLexer {
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub enum_token: token::Enum,
    pub ident: Ident,
    pub brace_token: token::Brace,
    pub variants: Vec<LexerVariant>,
    pub error_type: syn::ItemType,
}


#[derive(Clone)]
pub enum LexerVariant {
    Single{
        variant: syn::Variant,
        colon: token::Colon,
        regex: syn::LitStr,
        comma: token::Comma,
    },
    Multiple{
        variant: syn::Variant,
        colon: token::Colon,
        brace_token: token::Brace,
        entrys: Vec<LexerEntry>,
    }
}
#[derive(Clone)]
pub struct LexerEntry {
    pub regex: syn::LitStr,
    pub fat_arrow_token: token::FatArrow,
    pub body: Option<syn::Expr>,
    pub comma: Option<token::Comma>,
}

impl syn::parse::Parse for EnumLexer {
    fn parse(input: ParseStream) -> Result<Self> {
        let error_type: syn::ItemType;
        if input.peek(syn::Token![type]) {
            error_type = input.parse()?;
        } else {
            error_type = syn::parse_quote!{ type LexError = Box<dyn std::error::Error>; }
        }
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse::<syn::Visibility>()?;
        let enum_token = input.parse::<syn::Token![enum]>()?;
        let ident = input.parse::<Ident>()?;

        let content;
        let brace_token = syn::braced!(content in input);

        let mut variants = Vec::new();
        while !content.is_empty() {
            variants.push(content.parse()?);
        }

        Ok(Self {
            attrs,
            vis,
            enum_token,
            ident,
            brace_token,
            variants,
            error_type
        })
    }
}

pub(crate) struct LexerMap {
    pub(crate) regex: syn::LitStr,
    pub(crate) expr: syn::Expr,
}

impl LexerVariant{
    pub(crate) fn variant(&self) -> &syn::Variant {
        use LexerVariant::*;
        match self {
            Single{variant,..} => variant,
            Multiple{variant,..} => variant
        }
        
    }

    pub(crate) fn regex_maps(self) -> Vec<LexerMap> {
        match self {
            LexerVariant::Single{ variant, regex,..} => {
                if let syn::Fields::Unit = variant.fields {
                    vec![LexerMap {
                        regex,
                        expr: syn::parse_quote!(TokenInner::#variant)
                    }]
                } else { vec![] }
            }
            LexerVariant::Multiple { entrys ,..} => {
                entrys.into_iter().map(|e|{
                    let expr = e.body.unwrap_or(
                        syn::parse_quote!( {return Ok(None);})
                    );
                    LexerMap {
                        regex: e.regex,
                        expr,
                    }
                }).collect()
            }
        }
    }
}

impl syn::parse::Parse for LexerVariant {
    

    fn parse(input: ParseStream) -> Result<Self> {
        let variant = input.parse()?;
        let colon = input.parse()?;
        if input.peek(syn::LitStr) {
            Ok(Self::Single {
                variant,
                colon,
                regex: input.parse()?,
                comma: input.parse()?,
            })
        } else {
            let content;
            let brace_token = syn::braced!(content in input);

            let mut entrys = Vec::new();
            while !content.is_empty() {
                entrys.push(content.parse()?);
            }

            Ok(Self::Multiple {
                variant,
                colon,
                brace_token,
                entrys,
            })
        }
    }
}

impl syn::parse::Parse for LexerEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let requires_comma;
        Ok(Self {
            regex: input.parse()?,
            fat_arrow_token: input.parse()?,
            body: {
                if input.peek(syn::Token![!]) {
                    let _ : syn::Token![!] = input.parse()?;
                    requires_comma = true;
                    None
                } else {
                    let body = input.parse()?;
                    requires_comma = requires_terminator(&body);
                    Some(body)
                }
            },
            comma: {
                if requires_comma && !input.is_empty() {
                    Some(input.parse()?)
                } else {
                    input.parse()?
                }
            },
        })
    }
}

pub fn requires_terminator(expr: &syn::Expr) -> bool {
    use syn::Expr;
    match *expr {
        Expr::Unsafe(..)
        | Expr::Block(..)
        | Expr::If(..)
        | Expr::Match(..)
        | Expr::While(..)
        | Expr::Loop(..)
        | Expr::ForLoop(..)
        | Expr::Async(..)
        | Expr::TryBlock(..) => false,
        _ => true,
    }
}

impl fmt::Debug for LexerEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\t\t{:?} => <expr>", self.regex.value())
    }
}

impl fmt::Debug for LexerVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single{variant, regex, ..}=> {
                write!(f, "\t{}(..): {}", variant.ident, regex.value())
            }
            Self::Multiple{entrys, variant, ..}=> {
                let vec: Vec<_> = entrys.iter().map(|v| format!("{:?}", v)).collect();
                write!(f, "\t{}(..): {{\n{}\n\t}}", variant.ident, vec.join("\n"))
            }
        }
    }
}

impl fmt::Debug for EnumLexer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vec: Vec<_> = self.variants.iter().map(|v| format!("{:?}", v)).collect();
        write!(f, "enum {} {{\n{}\n}}", self.ident.to_string(), vec.join("\n"))
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use std::assert_eq;
    
    #[test]
    fn test0() {
        let ast: EnumLexer = syn::parse_str("enum lexer { Alpha(String): { \"alpha\" => Alpha(token_text) + 1 } }").unwrap();
        assert_eq!{
            format!("{:?}", ast),
            "enum lexer {\n\tAlpha(..): {\n\t\t\"alpha\" => <expr>\n\t}\n}"
        }
        
        let ast: EnumLexer = syn::parse_str(r#"
            enum lexer {
                Alpha: {
                    "alpha" => Alpha,
                    "alpha0" => Alpha,
                }
                Beta(String): {
                    "beta" => !,
                    "beta" => Beta(text),
                }
                Gamma: "gamma",
            }
        "#).unwrap();

        assert_eq!{
            format!("{:?}", ast),
            "enum lexer {\n\tAlpha(..): {\n\t\t\"alpha\" => <expr>\n\t\t\"alpha0\" => <expr>\n\t}\n\tBeta(..): {\n\t\t\"beta\" => <expr>\n\t\t\"beta\" => <expr>\n\t}\n\tGamma(..): gamma\n}"
        }
    }
}