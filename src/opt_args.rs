use impl_variadics::impl_variadics;
use syn::{
    parse::{Parse, ParseStream},
    token::Token,
    Result,
};

use crate::with_comma;

pub trait ParseOptionalArgs {
    type Output;
    fn parse(input: ParseStream) -> Result<Self::Output>;
}

impl_variadics! {
    ..21 "T*" => {
        impl<#(#T0,)*> ParseOptionalArgs for (#(#T0,)*)
        where
            #(#T0: Parse + Token,)*
        {
            type Output = (#(Option<#T0>,)*);

            fn parse(_input: ParseStream) -> Result<Self::Output> {
                #[allow(unused_mut)]
                let mut output: Self::Output = (#(None,)*);

                #(
                    match _input.parse::<Option<#T0>>()? {
                        v @ Some(_) => {
                            output.#index = v;
                            with_comma(_input)?;
                        },
                        None => return Ok(output),
                    }
                )*

                Ok(output)
            }
        }
    }
}
