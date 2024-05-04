use impl_variadics::impl_variadics;
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

use crate::with_comma;

pub trait ParseRequiredArgs {
    type Output;
    fn parse(input: ParseStream) -> Result<Self::Output>;
}

impl_variadics! {
    ..21 "T*" => {
        impl<#(#T0,)*> ParseRequiredArgs for (#(#T0,)*)
        where
            #(#T0: Parse,)*
        {
            type Output = (#(#T0,)*);

            fn parse(_input: ParseStream) -> Result<Self::Output> {
                let r = (#({
                    let x = _input.parse()?;
                    with_comma(_input)?;
                    x
                },)*);

                Ok(r)
            }
        }
    }
}
