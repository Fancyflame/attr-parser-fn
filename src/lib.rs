use std::{collections::HashSet, marker::PhantomData};

use args::ParseRequiredArgs;
use meta::ParseMeta;
use opt_args::ParseOptionalArgs;
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use rest_args::ParseRestArgs;
use syn::{
    buffer::Cursor,
    parse::{ParseStream, Parser},
    Attribute, Error, Meta, Result, Token,
};

pub mod args;
pub mod find_attr;
pub mod meta;
pub mod opt_args;
pub mod rest_args;

pub trait ParseAttrTrait: Sized {
    type Output;
    fn parse(self, input: ParseStream) -> Result<Self::Output>;

    fn parse_attr(self, input: &Attribute) -> Result<Self::Output> {
        (|input: ParseStream| self.parse(input)).parse2(match &input.meta {
            Meta::Path(_) => TokenStream::new(),
            Meta::List(list) => list.tokens.clone(),
            Meta::NameValue(meta) => {
                return Err(Error::new_spanned(
                    meta,
                    "expect `path` or `list(...)`, found `key = value`",
                ))
            }
        })
    }

    fn parse_concat_attrs<'r, I>(self, input: I) -> Result<Self::Output>
    where
        I: Iterator<Item = &'r Attribute>,
    {
        let parser = |input: ParseStream| self.parse(input);
        let mut concatenated = TokenStream::new();

        for attr in input {
            let tokens = attr.meta.require_list()?.tokens.clone();
            if tokens.is_empty() {
                continue;
            }

            let mut trail_comma = false;
            concatenated.extend(tokens.into_iter().map(|token| {
                trail_comma = matches!(&token, TokenTree::Punct(p) if p.as_char() == ',');
                token
            }));

            if !trail_comma {
                <Token![,]>::default().to_tokens(&mut concatenated);
            }
        }

        parser.parse2(concatenated)
    }
}

pub struct Marker<T>(PhantomData<T>);

impl<ReqArgs, OptArgs, RestArgs, Meta> ParseAttrTrait
    for ParseArgs<Marker<ReqArgs>, Marker<OptArgs>, Marker<RestArgs>, Meta>
where
    ReqArgs: ParseRequiredArgs,
    OptArgs: ParseOptionalArgs,
    RestArgs: ParseRestArgs,
    Meta: ParseMeta,
{
    type Output = ParseArgs<ReqArgs::Output, OptArgs::Output, RestArgs, Meta::Output>;

    fn parse(mut self, input: ParseStream) -> Result<Self::Output> {
        Ok(ParseArgs {
            args: ReqArgs::parse(input)?,
            opt_args: OptArgs::parse(input)?,
            rest_args: RestArgs::parse(input)?,
            meta: {
                let mut specified_paths = HashSet::new();
                let cursor = input.cursor();
                syn::meta::parser(|nested| {
                    let id = nested.path.require_ident()?.to_string();

                    if specified_paths.contains(&id) {
                        return Err(Error::new_spanned(
                            nested.path,
                            format!("path `{id}` has been specified",),
                        ));
                    }

                    if !self.meta.parse(&nested)? {
                        return Err(Error::new_spanned(
                            nested.path,
                            format!("attribute `{id}` is not expected, or the calling form is not compliant"),
                        ));
                    }

                    specified_paths.insert(id);
                    Ok(())
                })
                .parse2(cursor.token_stream())?;

                // set input buffer to empty
                input.step(|_| Ok(((), Cursor::empty()))).unwrap();

                self.meta.finish()?
            },
        })
    }
}

#[derive(Debug)]
pub struct ParseArgs<ReqArgs, OptArgs, RestArgs, Meta> {
    pub args: ReqArgs,
    pub opt_args: OptArgs,
    pub rest_args: RestArgs,
    pub meta: Meta,
}

impl ParseArgs<Marker<()>, Marker<()>, Marker<()>, ()> {
    pub fn new() -> Self {
        ParseArgs {
            args: marker(),
            opt_args: marker(),
            rest_args: marker(),
            meta: (),
        }
    }
}

impl<ReqArgs, OptArgs, RestArgs, Meta> ParseArgs<ReqArgs, OptArgs, RestArgs, Meta> {
    pub fn args<T: ParseRequiredArgs>(self) -> ParseArgs<Marker<T>, OptArgs, RestArgs, Meta> {
        ParseArgs {
            args: marker(),
            opt_args: self.opt_args,
            rest_args: self.rest_args,
            meta: self.meta,
        }
    }

    pub fn opt_args<T: ParseOptionalArgs>(self) -> ParseArgs<ReqArgs, Marker<T>, RestArgs, Meta> {
        ParseArgs {
            args: self.args,
            opt_args: marker(),
            rest_args: self.rest_args,
            meta: self.meta,
        }
    }

    pub fn rest_args<T: ParseRestArgs>(self) -> ParseArgs<ReqArgs, OptArgs, Marker<T>, Meta> {
        ParseArgs {
            args: self.args,
            opt_args: self.opt_args,
            rest_args: marker(),
            meta: self.meta,
        }
    }

    pub fn meta<T: ParseMeta>(self, meta: T) -> ParseArgs<ReqArgs, OptArgs, RestArgs, T> {
        ParseArgs {
            args: self.args,
            opt_args: self.opt_args,
            rest_args: self.rest_args,
            meta,
        }
    }
}

fn with_comma(input: ParseStream) -> Result<()> {
    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }

    Ok(())
}

fn marker<T>() -> Marker<T> {
    Marker(PhantomData)
}
