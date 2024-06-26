//! Serde Deserializer for tree-sitter.
//!
//! Use [`from_tree`] or [`from_node`] function to map tree-sitter's parse result to any type you
//! want.
//!
//! # Mapping rules
//!
//! ## Root types(`R`)
//!
//! * Atom types
//! * Tuple
//! * `Vec<R>`
//! * Unit struct(`struct Foo;`)
//! * Newtype struct(`struct Foo(N)`)
//! * Tuple struct(`struct Foo(T1, T2)`)
//! * Struct(`struct Foo{f1: F1, f2: F2}`)
//! * Enum
//!  * `UnitVariant`
//!  * `NewtypeVariant(N)`
//!  * `TupleVariant(R1, R2)`
//!  * `StructVariant{f1: F1, f2: F2}`
//!
//! ## Atom types
//!
//! * `()`
//! * `String`, `&str`, `&[u8]`
//! * `bool`
//! * Number types: `(u|i)(8|16|32|64)` and `f(32|64)`
//!
//! ## Newtype struct member type(`N`)
//!
//! * Atom types: Matches the node itself.
//! * `Vec<R>`: Matches named children.
//! * `Option<R>` Matches 0 or 1 named child.
//! * `(R1, R2, ..., RN)`: Matches exact N named children.
//! * Structs/Enums: Matches exact one named child.
//!
//! ## Field member types(`F`)
//!
//! * `(R1, R2, ..., RN)`: Matches exact N named children in the field.
//! * `Vec<R>`: Matches named children in the field.
//! * `Option<R>` Matches 0 or 1 named child in the field.
//! * Any other root types: If there is exact one node in the field, matches against it.

mod access;
mod deserializer;
mod error;
mod tsnode;

pub use error::DeserializeError;

pub fn from_tree<'d, D: serde::Deserialize<'d>>(
    tree: &'d tree_sitter::Tree,
    src: &'d str,
    check_error: bool,
) -> Result<D, DeserializeError> {
    from_node(tree.root_node(), src, check_error)
}

pub fn from_node<'de, D: serde::Deserialize<'de>>(
    node: tree_sitter::Node<'de>,
    src: &'de str,
    check_error: bool,
) -> Result<D, DeserializeError> {
    if check_error && node.has_error() {
        return Err(DeserializeError::TreeSitterError(collect_errors(node)));
    }
    let deserializer =
        crate::deserializer::NodeDeserializer::new(tsnode::TsNodeImpl::new(node, src));
    D::deserialize(deserializer)
}

fn collect_errors(node: tree_sitter::Node) -> Vec<tree_sitter::Range> {
    fn rec(node: tree_sitter::Node, buf: &mut Vec<tree_sitter::Range>) {
        if node.is_error() {
            buf.push(node.range());
        }
        if node.has_error() {
            for c in node.children(&mut node.walk()) {
                rec(c, buf);
            }
        }
    }
    let mut buf = Vec::new();
    rec(node, &mut buf);
    buf
}

#[cfg(test)]
mod test {
    use self::tsnode::TsNode;

    use super::*;

    struct DummyNode {
        kind: &'static str,
        src: &'static str,
        named_children: Vec<(Option<&'static str>, DummyNode)>,
    }
    impl DummyNode {
        fn new(
            kind: &'static str,
            src: &'static str,
            named_children: Vec<(Option<&'static str>, DummyNode)>,
        ) -> DummyNode {
            DummyNode {
                kind,
                src,
                named_children,
            }
        }
    }
    impl std::fmt::Debug for &DummyNode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DummyNode")
                .field("kind", &self.kind)
                .field("named_child_count", &self.named_child_count())
                .field(
                    "children",
                    &self
                        .named_children
                        .iter()
                        .map(|(f, n)| (f, n))
                        .collect::<Vec<_>>(),
                )
                .finish_non_exhaustive()
        }
    }
    impl<'de> TsNode<'de> for &DummyNode {
        fn named_child(&self, index: usize) -> Option<Self> {
            self.named_children.get(index).map(|x| &x.1)
        }

        fn named_child_count(&self) -> usize {
            self.named_children.len()
        }

        fn named_children(&self) -> impl ExactSizeIterator<Item = Self> {
            self.named_children.iter().map(|(_, n)| n)
        }

        fn children_by_field_name(&self, name: &str) -> impl ExactSizeIterator<Item = Self> {
            self.named_children
                .iter()
                .filter_map(|(f, n)| f.filter(|f| f == &name).map(|_| n))
                .collect::<Vec<_>>()
                .into_iter()
        }

        fn kind(&self) -> &'static str {
            self.kind
        }

        fn src(&self) -> &'de str {
            self.src
        }
    }

    #[ctor::ctor]
    fn before_all() {
        color_backtrace::install();
    }

    use pretty_assertions::assert_eq;
    use serde::Deserialize;

    fn deserialize<'de, D: Deserialize<'de>>(node: &'de DummyNode) -> Result<D, DeserializeError> {
        D::deserialize(crate::deserializer::NodeDeserializer::new(node))
    }

    macro_rules! make_node {
        ($tpe:ident $($src:literal)? $($($field:ident :)? ($($child:tt)*))*) => {
            make_node!(@make $tpe, $($src)?, vec![
                $(
                    make_node!(@child $($field)?: ($($child)*))
                ),*
            ])
        };
        (@make $tpe:ident, $src:literal, $children:expr) => {
            DummyNode::new(stringify!($tpe), $src, $children)
        };
        (@make $tpe:ident, , $children:expr) => {
            DummyNode::new(stringify!($tpe), "", $children)
        };

        (@child $field:ident : ($($child:tt)*)) => {
            (Some(stringify!($field)), make_node!($($child)*))
        };
        (@child : ($($child:tt)*)) => {
            (None, make_node!($($child)*))
        };
    }

    macro_rules! assert_ok {
        ($t:ty, ($($node:tt)+), $expected:expr) => {
            assert_eq!(deserialize::<$t>(&make_node!($($node)+)).unwrap(), $expected);
        };
    }
    macro_rules! assert_err {
        ($t:ty, ($($node:tt)+), $expected:expr) => {
            assert_eq!(deserialize::<$t>(&make_node!($($node)+)).unwrap_err(), $expected);
        };
    }

    #[test]
    fn test_unit_ok() {
        assert_ok!((), (root), ());
    }

    macro_rules! define_test_simple_ok {
        ($name:ident, $t:ty, $repr:literal, $expected:expr) => {
            #[test]
            fn $name() {
                assert_ok!($t, (root $repr), $expected as $t);
            }
        };
    }
    macro_rules! define_test_simple {
        ($name:ident, $t:ty, $repr:literal, $expected:expr, $err:ident) => {
            #[test]
            fn $name() {
                assert_ok!($t, (root $repr), $expected as $t);
                assert_err!(
                    $t,
                    (root "invalid_value"),
                    DeserializeError::$err("invalid_value".parse::<$t>().unwrap_err())
                );
            }
        };
    }
    macro_rules! define_test_int {
        ($name:ident, $t:ty, $repr:literal, $expected:expr) => {
            define_test_simple!($name, $t, $repr, $expected, ParseIntError);
        };
    }
    macro_rules! define_test_float {
        ($name:ident, $t:ty, $repr:literal, $expected:expr) => {
            define_test_simple!($name, $t, $repr, $expected, ParseFloatError);
        };
    }

    define_test_int!(test_i8_ok, i8, "123", 123);
    define_test_int!(test_i16_ok, i16, "123", 123);
    define_test_int!(test_i32_ok, i32, "123", 123);
    define_test_int!(test_i64_ok, i64, "123", 123);
    define_test_int!(test_u8_ok, u8, "123", 123);
    define_test_int!(test_u16_ok, u16, "123", 123);
    define_test_int!(test_u32_ok, u32, "123", 123);
    define_test_int!(test_u64_ok, u64, "123", 123);
    define_test_float!(test_f32_ok, f32, "1234.5", 1234.5);
    define_test_float!(test_f64_ok, f64, "1234.5", 1234.5);
    define_test_simple!(test_bool_ok, bool, "true", true, ParseBoolError);

    define_test_simple_ok!(test_string_ok, String, "abc", "abc".to_owned());
    define_test_simple_ok!(test_str_ok, &str, "abc", "abc");
    define_test_simple_ok!(test_array_u8_ok, &[u8], "abc", "abc".as_bytes());

    #[test]
    fn test_unit_struct() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root;

        assert_ok!(Root, (root), Root);
        assert_err!(
            Root,
            (not_root),
            DeserializeError::NodeType {
                expected: "root".into(),
                actual: "not_root".into()
            }
        );
    }

    #[test]
    fn test_tuple_struct_0() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root();

        assert_ok!(Root, (root), Root());

        assert_err!(
            Root,
            (not_root),
            DeserializeError::NodeType {
                expected: "root".into(),
                actual: "not_root".into()
            }
        );

        assert_err!(Root, (root(child)), DeserializeError::child_length(0, 1));
    }

    #[test]
    fn test_tuple_struct_n() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root(u32, u32);

        assert_ok!(
            Root,
            (root (child "123") (child "456")),
            Root(123, 456)
        );
        assert_err!(
            Root,
            (root (child "123")),
            DeserializeError::child_length(2, 1)
        );
        assert_err!(
            Root,
            (root (child "123") (child "456") (child "789")),
            DeserializeError::child_length(2, 3)
        );
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root(Vec<i32>);

        assert_ok!(Root, (root), Root(vec![]));
        assert_err!(
            Root,
            (not_root),
            DeserializeError::node_type("root", "not_root")
        );
    }

    #[test]
    fn test_newtype_struct_vec() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root(Vec<u32>);

        assert_ok!(
            Root,
            (root "xxx"),
            Root(vec![])
        );
        assert_ok!(
            Root,
            (root "xxx" (child "123") (c "456")),
            Root(vec![123, 456])
        );
        assert_err!(
            Root,
            (not_root "xxx"),
            DeserializeError::node_type("root", "not_root")
        );
    }

    #[test]
    fn test_newtype_struct_option() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root(Option<u32>);

        assert_ok!(
            Root,
            (root "xxx"),
            Root(None)
        );
        assert_ok!(
            Root,
            (root "xxx" (child "123")),
            Root(Some(123))
        );
        assert_err!(
            Root,
            (not_root "xxx"),
            DeserializeError::node_type("root", "not_root")
        );
        assert_err!(
            Root,
            (root "xxx" (child "123") (child "456")),
            DeserializeError::child_length(1, 2)
        );
    }

    #[test]
    fn test_newtype_struct_tuple() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root((u32, u32));

        assert_ok!(
            Root,
            (root (child "123") (child "456")),
            Root((123, 456))
        );
        assert_err!(
            Root,
            (root (child "123")),
            DeserializeError::child_length(2, 1)
        );
        assert_err!(
            Root,
            (root (child "123") (child "456") (child "789")),
            DeserializeError::child_length(2, 3)
        );
    }

    #[test]
    fn test_newtype_struct_string() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root(String);

        assert_ok!(
            Root,
            (root "abc"),
            Root("abc".into())
        );
        assert_ok!(
            Root,
            (root "abc" (child "xxx")),
            Root("abc".into())
        );
    }

    #[test]
    fn test_newtype_struct_num() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root(i32);

        assert_ok!(Root, (root "123"), Root(123));
        assert_ok!(Root, (root "123" (child "456")), Root(123));
    }

    #[test]
    fn test_newtype_struct_struct() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root(Child);

        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "child")]
        struct Child {
            a: i32,
        }

        assert_ok!(Root, (root (child a: (num "123"))), Root(Child{a: 123}));
    }

    #[test]
    fn test_struct() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root {
            a: u64,
            b: String,
        }

        assert_ok!(
            Root,
            (root a: (child "123") b: (child "abc")),
            Root { a: 123, b: "abc".into()}
        );
        assert_err!(
            Root,
            (not_root a: (child "123") b: (child "abc")),
            DeserializeError::node_type("root", "not_root")
        );
        assert_err!(
            Root,
            (root b: (child "abc")),
            DeserializeError::field_length("a", 1, 0)
        );
        assert_err!(
            Root,
            (root a: (child "xxx") b: (child "abc")),
            DeserializeError::ParseIntError("xxx".parse::<u64>().unwrap_err())
        );
        assert_err!(
            Root,
            (root a: (child "123") a: (child "456") b: (child "abc")),
            DeserializeError::field_length("a", 1, 2)
        );
    }

    #[test]
    fn test_struct_tuple() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root {
            a: (u32, u32),
        }

        assert_ok!(
            Root,
            (root
                a: (child "123")
                (child "999")
                a: (child "456")),
            Root { a: (123, 456) }
        );
        assert_err!(
            Root,
            (root
                a: (child "123")
                (child "999")),
            DeserializeError::field_length("a", 2, 1)
        );
    }

    #[test]
    fn test_struct_vec() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root {
            a: Vec<u32>,
        }

        assert_ok!(
            Root,
            (root "999"),
            Root { a: vec![]}
        );
        assert_ok!(
            Root,
            (root "999"
                a: (child "123")),
            Root { a: vec![123]}
        );
    }

    #[test]
    fn test_struct_option() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root {
            a: Option<u32>,
        }

        assert_ok!(
            Root,
            (root "123"),
            Root { a: None }
        );
        assert_ok!(
            Root,
            (root "123" a: (child "456")),
            Root { a: Some(456) }
        );
        assert_err!(
            Root,
            (root "123" a: (child "456") a: (child "789")),
            DeserializeError::field_length("a", 1, 2)
        );
    }

    #[test]
    fn test_tuple() {
        // arity = 1
        assert_ok!(
            (i32,),
            (root (child "123")),
            (123,)
        );
        assert_err!((i32,), (root), DeserializeError::child_length(1, 0));
        assert_err!(
            (i32,),
            (root (child "123") (child "456")),
            DeserializeError::child_length(1, 2)
        );
        assert_err!(
            (i32,),
            (root (child "xxx")),
            DeserializeError::ParseIntError("xxx".parse::<i32>().unwrap_err())
        );

        // arity = 2
        assert_ok!(
            (i32, u8),
            (root (child "123") (child "99")),
            (123, 99)
        );
        assert_err!((i32, u8), (root), DeserializeError::child_length(2, 0));
        assert_err!(
            (i32, u8),
            (root (child "1") (child "2") (child "3")),
            DeserializeError::child_length(2, 3)
        );
        assert_err!(
            (i32, u8),
            (root (child "123") (child "yyy")),
            DeserializeError::ParseIntError("yyy".parse::<u8>().unwrap_err())
        );
    }

    #[test]
    fn test_vec() {
        assert_ok!(Vec<i32>, (root), vec![]);
        assert_ok!(Vec<i32>, (root (child "123") (child "456")), vec![123, 456]);
    }

    #[test]
    fn test_option() {
        assert_ok!(Option<i32>, (root), None);
        assert_ok!(Option<i32>, (root (child "123")), Some(123));
        assert_err!(Option<i32>, (root (child "123") (child "456")), DeserializeError::child_length(1, 2));
    }

    #[test]
    fn test_enum() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename_all = "snake_case")]
        enum Value {
            // Unit variant
            Null,
            // Newtype variant
            Int(i64),
            // Tuple variant
            Tuple(String, i32),
            // Struct variant
            Struct {
                a: u32,
                b: Vec<String>,
                c: Option<String>,
            },
        }

        // unit
        assert_eq!(
            deserialize::<Value>(&make_node!(null "foo")).unwrap(),
            Value::Null,
        );

        // newtype(ok)
        assert_eq!(
            deserialize::<Value>(&make_node!(int "999")).unwrap(),
            Value::Int(999),
        );

        // TODO: newtype(error)

        // tuple(ok)
        assert_eq!(
            deserialize::<Value>(&make_node!(tuple "999" (c1 "foo") (c2 "333"))).unwrap(),
            Value::Tuple("foo".into(), 333),
        );

        // tuple(error: child count)
        assert_eq!(
            deserialize::<Value>(&make_node!(tuple "999" (c1 "foo"))).unwrap_err(),
            DeserializeError::child_length(2, 1),
        );

        // tuple(error: type error)
        assert_eq!(
            deserialize::<Value>(&make_node!(tuple "999" (c1 "foo") (c2 "not_a_number")))
                .unwrap_err(),
            DeserializeError::ParseIntError("not_a_number".parse::<i32>().unwrap_err())
        );

        // struct(ok: b = [...], c = None)
        assert_eq!(
            deserialize::<Value>(&make_node!(struct ""
                a: (foo "123")
                b: (bar "a")
                b: (bar "b")
                (baz)
                b: (bar "c")
            ))
            .unwrap(),
            Value::Struct {
                a: 123,
                b: ["a", "b", "c"].into_iter().map(|x| x.to_owned()).collect(),
                c: None,
            }
        );

        // struct(ok: b = [], c = "foo")
        assert_eq!(
            deserialize::<Value>(&make_node!(struct ""
                a: (foo "123")
                (baz)
                c: (foo "foo")
            ))
            .unwrap(),
            Value::Struct {
                a: 123,
                b: vec![],
                c: Some("foo".into()),
            }
        );
        // struct(error: missing a)
        assert_eq!(
            deserialize::<Value>(&make_node!(struct ""
                b: (foo "123")
                (baz)
            ))
            .unwrap_err(),
            DeserializeError::field_length("a", 1, 0)
        );
        // struct(error: option field length > 1)
        assert_eq!(
            deserialize::<Value>(&make_node!(struct ""
                a: (foo "123")
                (baz)
                c: (foo "foo")
                c: (foo "foo")
            ))
            .unwrap_err(),
            DeserializeError::field_length("c", 1, 2)
        );
        // error: unknown variant
        assert!(deserialize::<Value>(&make_node!(unknown ""
            a: (foo "123")
            (baz)
            c: (foo "foo")
            c: (foo "foo")
        ))
        .is_err());
    }

    #[test]
    fn test_json() {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(tree_sitter_json::language()).unwrap();

        #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        #[serde(rename = "document")]
        struct Document(Vec<Value>);

        #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum Value {
            Object(Vec<Pair>),
            Number(String),
            Array(Vec<Value>),
            String(Option<StringContent>),
            Null,
        }

        #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
        #[serde(rename = "pair", rename_all = "snake_case")]
        struct Pair {
            key: StringContainer,
            value: Box<Value>,
        }

        #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
        #[serde(rename = "string")]
        struct StringContainer(Option<StringContent>);

        #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
        #[serde(rename = "string_content")]
        struct StringContent(String);

        let src = r#"
        {
            "foo": 123,
            "bar": 4.5,
            "baz": [null, 0, ""]
        }
        {}
        "#;

        let tree = parser.parse(src, None).unwrap();
        // show_node(&TsNodeImpl { node: tree.root_node(), src, });
        let ast: Document = from_tree(&tree, src, true).unwrap();

        assert_eq!(
            ast,
            Document(vec![
                Value::Object(vec![
                    Pair {
                        key: StringContainer(Some(StringContent("foo".into()))),
                        value: Box::new(Value::Number("123".into()))
                    },
                    Pair {
                        key: StringContainer(Some(StringContent("bar".into()))),
                        value: Box::new(Value::Number("4.5".into()))
                    },
                    Pair {
                        key: StringContainer(Some(StringContent("baz".into()))),
                        value: Box::new(Value::Array(vec![
                            Value::Null,
                            Value::Number("0".into()),
                            Value::String(None),
                        ]))
                    }
                ]),
                Value::Object(vec![]),
            ])
        );
    }
}
