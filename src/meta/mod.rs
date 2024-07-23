use std::fmt::Write;

use impl_variadics::impl_variadics;
use proc_macro2::{Span, TokenStream};
use syn::{
    meta::ParseNestedMeta,
    parenthesized,
    parse::{Parse, ParseStream, Parser},
    token::Paren,
    Error, LitStr, Result, Token,
};

use crate::ParseAttrTrait;

pub use self::{
    conflicts::{conflicts, ConflictGroup},
    utils::{meta_list, Map, MetaList, Optional, ParseMetaExt},
};

mod conflicts;
mod utils;

pub trait ParseMeta {
    type Output;

    fn conflict_alternative_arm(&self, f: &mut dyn Write) -> std::fmt::Result;
    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool>;
    fn finish(self) -> Result<Self::Output>;
    fn ok_to_finish(&self) -> bool;
}

pub trait ParseMetaUnnamed {
    type Output;

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool>;
    fn finish(self) -> Option<Self::Output>;
    fn ok_to_finish(&self) -> bool;
}

impl<T> ParseMeta for (&str, T)
where
    T: ParseMetaUnnamed,
{
    type Output = T::Output;

    fn conflict_alternative_arm(&self, f: &mut dyn Write) -> std::fmt::Result {
        write!(f, "`{}`", self.0)
    }

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool> {
        if nested.path.is_ident(self.0) {
            self.1.parse(nested)
        } else {
            Ok(false)
        }
    }

    fn finish(self) -> Result<Self::Output> {
        self.1.finish().ok_or_else(|| {
            Error::new(
                Span::call_site(),
                format!("attribute path `{}` must be specified", self.0),
            )
        })
    }

    fn ok_to_finish(&self) -> bool {
        self.1.ok_to_finish()
    }
}

pub fn path_only() -> PathOnly {
    PathOnly { assigned: false }
}

pub struct PathOnly {
    assigned: bool,
}

impl ParseMetaUnnamed for PathOnly {
    type Output = bool;

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool> {
        if nested.input.peek(Token![,]) || nested.input.is_empty() {
            self.assigned = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn finish(self) -> Option<Self::Output> {
        Some(self.assigned)
    }

    fn ok_to_finish(&self) -> bool {
        true
    }
}

pub fn key_value<T>() -> KeyValue<T>
where
    T: Parse,
{
    KeyValue { value: None }
}

pub struct KeyValue<T> {
    value: Option<T>,
}

impl<T> ParseMetaUnnamed for KeyValue<T>
where
    T: Parse,
{
    type Output = T;

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool> {
        if nested.input.peek(Token![=]) {
            self.value = Some(nested.value()?.parse::<T>()?);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn finish(self) -> Option<Self::Output> {
        self.value
    }

    fn ok_to_finish(&self) -> bool {
        self.value.is_some()
    }
}

pub fn key_str<T>() -> KeyStr<T>
where
    T: Parse,
{
    KeyStr { value: None }
}

pub struct KeyStr<T> {
    value: Option<T>,
}

impl<T> ParseMetaUnnamed for KeyStr<T>
where
    T: Parse,
{
    type Output = T;

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool> {
        if nested.input.peek(Token![=]) {
            let litstr: LitStr = nested.value()?.parse()?;
            self.value = Some(litstr.parse()?);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn finish(self) -> Option<Self::Output> {
        self.value
    }

    fn ok_to_finish(&self) -> bool {
        self.value.is_some()
    }
}

pub fn list<P>(parser: P) -> List<P>
where
    P: ParseAttrTrait,
{
    List(ListInner::Unassigned(parser))
}

enum ListInner<P>
where
    P: ParseAttrTrait,
{
    Unassigned(P),
    Assigned(P::Output),
    Intermediate,
}

pub struct List<P>(ListInner<P>)
where
    P: ParseAttrTrait;

impl<P> ParseMetaUnnamed for List<P>
where
    P: ParseAttrTrait,
{
    type Output = P::Output;

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool> {
        if !nested.input.peek(Paren) {
            return Ok(false);
        }

        let inner = &mut self.0;

        let content;
        parenthesized!(content in nested.input);

        let ListInner::Unassigned(parser) = std::mem::replace(inner, ListInner::Intermediate)
        else {
            unreachable!("cannot assign a list twice");
        };

        *inner = ListInner::Assigned(parser.parse(&content)?);
        Ok(true)
    }

    fn finish(self) -> Option<Self::Output> {
        match self.0 {
            ListInner::Assigned(output) => Some(output),
            ListInner::Intermediate => unreachable!("this list is not correctly assigned"),
            ListInner::Unassigned(parser) => {
                let new_parser = |input: ParseStream| parser.parse(input);
                new_parser.parse2(TokenStream::new()).ok()
            }
        }
    }

    fn ok_to_finish(&self) -> bool {
        true
    }
}

impl_variadics! {
    ..21 "T*" => {
        impl<#(#T0,)*> ParseMeta for (#(#T0,)*)
        where
            #(#T0: ParseMeta,)*
        {
            type Output = (#(#T0::Output,)*);

            fn conflict_alternative_arm(&self, _f: &mut dyn Write) -> std::fmt::Result {
                let mut _comma = "";

                #(
                    write!(_f, "{_comma}")?;
                    self.#index.conflict_alternative_arm(_f)?;
                    _comma = ", ";
                )*

                Ok(())
            }

            fn parse(&mut self, _nested: &ParseNestedMeta) -> Result<bool> {
                Ok(false #(|| self.#index.parse(_nested)?)*)
            }

            fn finish(self) -> Result<Self::Output> {
                Ok((
                    #(self.#index.finish()?,)*
                ))
            }

            fn ok_to_finish(&self) -> bool {
                true #(&& self.#index.ok_to_finish())*
            }
        }
    }
}
