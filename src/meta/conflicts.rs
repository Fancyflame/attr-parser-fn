use impl_variadics::impl_variadics;
use proc_macro2::Span;
use syn::{meta::ParseNestedMeta, Error, Result};

use super::ParseMeta;

pub fn conflicts<T>(group: T) -> Conflicts<T>
where
    T: ConflictGroup,
{
    Conflicts {
        parser: group,
        selected: None,
    }
}

pub struct Conflicts<T>
where
    T: ConflictGroup,
{
    parser: T,
    selected: Option<(String, u8)>,
}

pub trait ConflictGroup: Sized {
    type Output;

    fn parse_meta_conflict_alternative_arm(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result;
    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<Option<u8>>;
    fn finish(self, index: u8) -> Result<<Self as ConflictGroup>::Output>;
}

impl_variadics! {
    1..21 "T*" => {
        impl<Out, #(#T0,)*> ConflictGroup for (#(#T0,)*)
        where
            #(#T0: ParseMeta<Output = Out>,)*
        {
            type Output = Out;

            fn parse_meta_conflict_alternative_arm(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
                self.conflict_alternative_arm(f)
            }

            fn parse(&mut self, nested: &ParseNestedMeta) -> Result<Option<u8>> {
                #(if self.#index.parse(nested)? {
                    Ok(Some(#index))
                } else)* {
                    Ok(None)
                }
            }

            fn finish(self, index: u8) -> Result<<Self as ConflictGroup>::Output> {
                match index {
                    #(#index => self.#index.finish(),)*
                    _ => unreachable!("invalid index")
                }
            }
        }
    }
}

impl<T> ParseMeta for Conflicts<T>
where
    T: ConflictGroup,
{
    type Output = T::Output;

    fn conflict_alternative_arm(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(f, "(conflict group: ")?;
        self.parser.parse_meta_conflict_alternative_arm(f)?;
        write!(f, ")")
    }

    fn parse(&mut self, nested: &ParseNestedMeta) -> Result<bool> {
        match self.parser.parse(nested)? {
            Some(index) => {
                let new_name = nested.path.require_ident()?.to_string();
                match &self.selected {
                    Some((name, _)) => Err(Error::new_spanned(
                        &nested.path,
                        format!("attribute `{new_name}` is conflicts with `{name}`"),
                    )),

                    None => {
                        self.selected = Some((new_name, index));
                        Ok(true)
                    }
                }
            }

            None => Ok(false),
        }
    }

    fn finish(self) -> Result<Self::Output> {
        match self.selected {
            Some((_, index)) => self.parser.finish(index),
            None => Err(Error::new(Span::call_site(), {
                let mut msg = "one of following attributes must be provided: ".to_string();
                self.parser
                    .parse_meta_conflict_alternative_arm(&mut msg)
                    .unwrap();
                msg
            })),
        }
    }

    fn ok_to_finish(&self) -> bool {
        self.selected.is_some()
    }
}
