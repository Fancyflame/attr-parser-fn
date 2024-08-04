use attr_parser_fn::{
    meta::{conflicts, key_value, meta_list, path_only, ParseMetaExt},
    ParseArgs, ParseAttrTrait,
};

use syn::{parse_quote, Attribute, Expr, Lit, LitStr, Type};

fn main() {
    let attr: Attribute = parse_quote! {
        #[my_attr(
            "hello",
            "world",
            122,
            conf1 = 114 + 514,
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
            ("kv_optional", key_value::<Expr>()).optional(),
            conflicts((
                ("conf1", path_only()).value("conf1"),
                ("conf1", key_value::<Expr>()).value("conf1_expr"),
                ("conf2", key_value::<Expr>()).value("conf2"),
            )),
            (
                "nested",
                meta_list((
                    ("milk", path_only()),
                    (
                        "tea",
                        meta_list(conflicts((
                            ("red_tea", path_only()).value("red_tea"),
                            ("green_tea", path_only()).value("green_tea"),
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
                "conf1_expr",
                (false, "green_tea"),
            ),
    } = parser.parse_attr(&attr).unwrap()
    else {
        unreachable!()
    };
}
