use std::fmt::Write;

use syn::{meta::ParseNestedMeta, Result};

use crate::{Marker, ParseArgs};

use super::{list, List, ParseMeta, ParseMetaUnnamed};

pub trait ParseMetaExt: ParseMeta + Sized {
    fn optional(self) -> Optional<Self> {
        Optional(self)
    }

    fn map<F, R>(self, map: F) -> Map<Self, F>
    where
        F: FnOnce(Self::Output) -> R,
    {
        Map { parser: self, map }
    }

    fn value<U>(self, value: U) -> impl ParseMeta<Output = U> {
        self.map(move |_| value)
    }
}

impl<T> ParseMetaExt for T where T: ParseMeta + Sized {}

pub struct Optional<T>(T);

impl<T> ParseMeta for Optional<T>
where
    T: ParseMeta,
{
    type Output = Option<T::Output>;

    fn conflict_alternative_arm(&self, f: &mut dyn Write) -> std::fmt::Result {
        self.0.conflict_alternative_arm(f)
    }

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool> {
        self.0.parse(nested)
    }

    fn finish(self) -> Result<Self::Output> {
        let opt = if self.0.ok_to_finish() {
            Some(self.0.finish()?)
        } else {
            None
        };

        Ok(opt)
    }

    fn ok_to_finish(&self) -> bool {
        true
    }
}

pub struct Map<T, F> {
    parser: T,
    map: F,
}

impl<T, F, R> ParseMeta for Map<T, F>
where
    T: ParseMeta,
    F: FnOnce(T::Output) -> R,
{
    type Output = R;

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool> {
        self.parser.parse(nested)
    }

    fn conflict_alternative_arm(&self, f: &mut dyn Write) -> std::fmt::Result {
        self.parser.conflict_alternative_arm(f)
    }

    fn finish(self) -> Result<Self::Output> {
        self.parser.finish().map(self.map)
    }

    fn ok_to_finish(&self) -> bool {
        self.parser.ok_to_finish()
    }
}

pub fn meta_list<P>(p: P) -> MetaList<P>
where
    P: ParseMeta,
{
    MetaList(list(ParseArgs::new().meta(p)))
}

pub struct MetaList<P>(List<ParseArgs<Marker<()>, Marker<()>, Marker<()>, P>>)
where
    P: ParseMeta;

impl<P> ParseMetaUnnamed for MetaList<P>
where
    P: ParseMeta,
{
    type Output = P::Output;

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool> {
        self.0.parse(nested)
    }

    fn ok_to_finish(&self) -> bool {
        self.0.ok_to_finish()
    }

    fn finish(self) -> Option<Self::Output> {
        self.0.finish().map(|x| x.meta)
    }
}
