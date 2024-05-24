#![feature(proc_macro_span)]

use std::fmt::Write;

use proc_macro::{Span, TokenStream as TokenStream1};
use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::{quote, quote_spanned};

struct Source {
    modified: String,
    column: usize,
    initial_line: Option<usize>,
    line: usize,
}

impl Source {
    fn add_whitespace(
        &mut self,
        span: Span,
        line: usize,
        column: usize,
    ) -> Result<(), TokenStream> {
        let initial_line = *self.initial_line.get_or_insert(line);
        let line = line.checked_sub(initial_line);
        let line = line
            .ok_or_else(|| quote_spanned!(span.into() => compile_error!{"Invalid line number"}))?;
        if line > self.line {
            while line > self.line {
                self.modified.push('\n');
                self.line += 1;
            }

            for _ in 0..column {
                self.modified.push(' ');
            }

            self.column = column;
        } else if line == self.line {
            while column > self.column {
                self.modified.push(' ');
                self.column += 1;
            }
        }

        Ok(())
    }

    fn add(&mut self, input: TokenStream) -> Result<(), TokenStream> {
        let mut tokens = input.into_iter();

        while let Some(token) = tokens.next() {
            let span = token.span().unwrap();
            let col = span.column();
            let line = span.line();

            dbg!(&token);
            dbg!(col, line);
            self.add_whitespace(span, span.line(), span.column())?;
            match &token {
                TokenTree::Group(x) => {
                    let (start, end) = match x.delimiter() {
                        Delimiter::Parenthesis => ("(", ")"),
                        Delimiter::Brace => ("{{", "}}"),
                        Delimiter::Bracket => ("[", "]"),
                        Delimiter::None => ("", ""),
                    };
                    self.modified.push_str(start);
                    self.column += start.len();
                    self.add(x.stream())?;
                    let end_span = token.span().unwrap().end();
                    self.add_whitespace(
                        span,
                        end_span.line(),
                        end_span.column().saturating_sub(end.len()),
                    )?;
                    self.modified.push_str(end);
                    self.column += end.len();
                }
                TokenTree::Punct(x) => {
                    // If we find a template parameter, like 'workgroup_size_y, then replace
                    // with {workgroup_size_y}
                    if x.as_char() == '\'' && x.spacing() == Spacing::Joint {
                        if let Some(TokenTree::Ident(name)) = tokens.next() {
                            let name_str = name.to_string();
                            self.modified.push('{');
                            self.modified.push_str(&name_str);
                            self.modified.push('}');
                            self.column += name_str.len();
                        } else {
                            return Err(quote! {
                                compile_error!("Expected a templated parameter after single quote.");
                            });
                        };
                    } else {
                        self.modified.push(x.as_char());
                    }
                    self.column += 1;
                }
                TokenTree::Ident(x) => {
                    write!(&mut self.modified, "{}", x).unwrap();
                    let end_span = token.span().unwrap().end();
                    self.column = end_span.column();
                }
                _ => {
                    self.modified.push_str(&token.to_string());
                    let end_span = token.span().unwrap().end();
                    self.column = end_span.column();
                }
            }
        }

        Ok(())
    }
}

fn inline_wgsl(input: TokenStream) -> Result<TokenStream, TokenStream> {
    let mut source = Source {
        modified: "".to_string(),
        column: 0,
        initial_line: None,
        line: 0,
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
