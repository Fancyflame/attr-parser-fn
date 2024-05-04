use derive_attr::{
    meta::{conflicts, key_value, meta_list, optional, path_only, value},
    ParseArgs, ParseAttrTrait,
};

use syn::{parse_quote, Attribute, Expr, Lit, LitStr, Type};

fn main() {
    let attr: Attribute = parse_quote! {
        #[my_attr(
            "hello",
            "world",
            122,
            conf2 = 114 + 514,
            key_value = SomeType<A, B>,
            path_only,
            nested(tea(green_tea)))
        ]
    };

    let parser = ParseArgs::new()
        .args::<(LitStr, LitStr)>()
        .opt_args::<(Lit, Lit)>()
        .rest_args::<Vec<Lit>>()
        .meta((
            ("path_only", path_only()),
            ("key_value", key_value::<Type>()),
            optional(("kv_optional", key_value::<Expr>())),
            conflicts((
                value(("conf1", path_only()), "conf1"),
                value(("conf2", key_value::<Expr>()), "conf2"),
            )),
            (
                "nested",
                meta_list((
                    ("milk", path_only()),
                    (
                        "tea",
                        meta_list(conflicts((
                            value(("red_tea", path_only()), "red_tea"),
                            value(("green_tea", path_only()), "green_tea"),
                        ))),
                    ),
                )),
            ),
        ));

    let ParseArgs {
        args: (_, _),              // ("hello", "world")
        opt_args: (Some(_), None), // (Some(112), None)
        rest_args: _,              // []
        meta:
            (
                true,
                _, // SomeType<A, B>
                None,
                "conf2",
                (false, "green_tea"),
            ),
    } = parser.parse_attrs(&attr).unwrap()
    else {
        unreachable!()
    };
}
