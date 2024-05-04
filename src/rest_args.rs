use syn::{
    parse::{Parse, ParseStream},
    token::Token,
    Result,
};

use crate::with_comma;

pub trait ParseRestArgs: Sized {
    fn parse(input: ParseStream) -> Result<Self>;
}

impl ParseRestArgs for () {
    fn parse(_: ParseStream) -> Result<Self> {
        Ok(())
    }
}

impl<P> ParseRestArgs for Vec<P>
where
    P: Parse + Token,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let mut vec = vec![];
        while let Some(v) = input.parse::<Option<P>>()? {
            vec.push(v);
            with_comma(input)?;
        }
        Ok(vec)
    }
}
