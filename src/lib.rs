use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::quote;

struct Source {
    modified: String,
}

impl Source {
    fn add(&mut self, input: TokenStream) -> Result<(), TokenStream> {
        let mut tokens = input.into_iter();

        while let Some(token) = tokens.next() {
            match &token {
                TokenTree::Group(x) => {
                    let (start, end) = match x.delimiter() {
                        Delimiter::Parenthesis => ("(", ")"),
                        Delimiter::Brace => ("{{", "}}"),
                        Delimiter::Bracket => ("[", "]"),
                        Delimiter::None => ("", ""),
                    };
                    self.modified.push_str(start);
                    self.add(x.stream())?;
                    self.modified.push_str(end);
                }
                TokenTree::Punct(x) => {
                    // If we find a template parameter, like 'workgroup_size_y, then replace
                    // with {workgroup_size_y}
                    if x.as_char() == '\'' && x.spacing() == Spacing::Joint {
                        if let Some(TokenTree::Ident(name)) = tokens.next() {
                            self.modified.push('{');
                            self.modified.push_str(&name.to_string());
                            self.modified.push('}');
                        } else {
                            return Err(quote! {
                                compile_error!("Expected a templated parameter after single quote.");
                            });
                        };
                    } else {
                        self.modified.push_str(&x.to_string());
                    }
                }
                TokenTree::Ident(v) => {
                    let v_str = v.to_string();
                    self.modified.push_str(&v.to_string());

                    match v_str.as_str() {
                        "var" | "let" | "fn" => {
                            self.modified.push(' ');
                        }
                        _ => {}
                    }
                }
                _ => {
                    self.modified.push_str(&token.to_string());
                }
            }
        }

        Ok(())
    }
}

fn inline_wgsl(input: TokenStream) -> Result<TokenStream, TokenStream> {
    let mut source = Source {
        modified: "".to_string(),
    };
    let _ = source.add(input);
    let modified = source.modified;

    let quoted = quote! {
        format!(#modified)
    };
    Ok(quoted)
}

#[doc(hidden)]
#[proc_macro]
pub fn wgsl(input: TokenStream1) -> TokenStream1 {
    TokenStream1::from(match inline_wgsl(TokenStream::from(input)) {
        Ok(tokens) => tokens,
        Err(tokens) => tokens,
    })
}
