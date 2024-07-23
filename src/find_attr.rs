use std::fmt::Display;

use syn::{Attribute, Error, Ident, Result};

fn path_is<I>(expect: &I) -> impl Fn(&&Attribute) -> bool + '_
where
    Ident: PartialEq<I>,
    I: ?Sized,
{
    move |attr| attr.path().is_ident(expect)
}

pub fn all<'a: 'p, 'p, I>(
    attrs: &'a [Attribute],
    expect_path: &'p I,
) -> impl Iterator<Item = &'a Attribute> + 'p
where
    Ident: PartialEq<I>,
    I: ?Sized,
{
    attrs.iter().filter(path_is(expect_path))
}

pub fn first<'a, I>(attrs: &'a [Attribute], expect_path: &I) -> Option<&'a Attribute>
where
    Ident: PartialEq<I>,
    I: ?Sized,
{
    attrs.iter().find(path_is(expect_path))
}

pub fn only<'a, I>(attrs: &'a [Attribute], expect_path: &I) -> Result<Option<&'a Attribute>>
where
    Ident: PartialEq<I>,
    I: Display + ?Sized,
{
    let mut found = None;
    for attr in attrs {
        if attr.path().is_ident(expect_path) {
            continue;
        }

        if found.is_some() {
            return Err(Error::new_spanned(
                attr,
                format!("conflicting declaration of attribute `{expect_path}`"),
            ));
        }

        found = Some(attr);
    }
    Ok(found)
}
