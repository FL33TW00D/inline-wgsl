#![feature(proc_macro_span)]

use std::fmt::Write;

use proc_macro::{Span, TokenStream as TokenStream1};
use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use std::cmp::Ordering;

struct Source {
    modified: String,
    column: usize,
    initial_line: Option<usize>,
    initial_col: Option<usize>,
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
        let line = line
            .checked_sub(initial_line)
            .ok_or_else(|| quote_spanned!(span.into() => compile_error!("Invalid line number")))?;

        let initial_col = *self.initial_col.get_or_insert(column);
        let column = column.checked_sub(initial_col).ok_or_else(
            || quote_spanned!(span.into() => compile_error!("Invalid column number")),
        )?;

        match line.cmp(&self.line) {
            Ordering::Greater => {
                self.modified.push_str(&"\n".repeat(line - self.line));
                self.line = line;
                self.modified.push_str(&" ".repeat(column));
                self.column = column;
            }
            Ordering::Equal => {
                self.modified.push_str(&" ".repeat(column - self.column));
                self.column = column;
            }
            _ => {}
        }

        Ok(())
    }

    fn add(&mut self, input: TokenStream) -> Result<(), TokenStream> {
        let mut tokens = input.into_iter();

        while let Some(token) = tokens.next() {
            let span = token.span().unwrap();
            self.add_whitespace(span, span.line(), span.column())?;

            match &token {
                TokenTree::Group(x) => {
                    let (start, end, sub) = match x.delimiter() {
                        Delimiter::Parenthesis => ("(", ")", 1),
                        Delimiter::Brace => ("{{", "}}", 1),
                        Delimiter::Bracket => ("[", "]", 1),
                        Delimiter::None => ("", "", 0),
                    };
                    self.modified.push_str(start);
                    self.column += start.len();
                    self.add(x.stream())?;
                    let end_span = token.span().unwrap().end();
                    self.add_whitespace(
                        span,
                        end_span.line(),
                        end_span.column().saturating_sub(sub),
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
                            write!(&mut self.modified, "{{{}}}", name_str).unwrap();
                            self.column += name_str.len() + 1;
                        } else {
                            return Err(quote! {
                                compile_error!("Expected a templated parameter after single quote.");
                            });
                        };
                    } else {
                        self.modified.push(x.as_char());
                        self.column += 1;
                    }
                }
                TokenTree::Ident(x) => {
                    write!(&mut self.modified, "{}", x).unwrap();
                    let end_span = token.span().unwrap().end();
                    let end_col = end_span.column() - self.initial_col.unwrap_or(0);
                    self.column = end_col;
                }
                _ => {
                    self.modified.push_str(&token.to_string());
                    let end_span = token.span().unwrap().end();
                    let end_col = end_span.column() - self.initial_col.unwrap_or(0);
                    self.column = end_col;
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
        initial_col: None,
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
