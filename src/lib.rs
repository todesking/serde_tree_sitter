mod access;
mod deserializer;
mod error;

pub use deserializer::NodeDeserializer;
pub use error::DeserializeError;

pub trait TsNode<'de>: Clone
where
    Self: Sized,
{
    fn named_child(&self, index: usize) -> Option<Self>;
    fn named_child_count(&self) -> usize;
    fn named_children(&self) -> impl ExactSizeIterator<Item = Self>;
    fn children_by_field_name(&self, name: &str) -> impl ExactSizeIterator<Item = Self>;
    fn kind(&self) -> &'static str;
    fn src(&self) -> &'de str;
}

#[derive(Clone)]
struct TsNodeImpl<'a, 'de> {
    node: tree_sitter::Node<'a>,
    src: &'de str,
}

impl<'a, 'de> TsNode<'de> for TsNodeImpl<'a, 'de> {
    fn named_child(&self, index: usize) -> Option<Self> {
        self.node.named_child(index).map(|c| TsNodeImpl {
            node: c,
            src: self.src,
        })
    }

    fn named_child_count(&self) -> usize {
        self.node.named_child_count()
    }

    fn named_children(&self) -> impl ExactSizeIterator<Item = Self> {
        let mut cursor = self.node.walk();
        let children = self.node.named_children(&mut cursor).collect::<Vec<_>>();
        children.into_iter().map(|node| TsNodeImpl {
            node,
            src: self.src,
        })
    }

    fn children_by_field_name(&self, name: &str) -> impl ExactSizeIterator<Item = Self> {
        let mut cursor = self.node.walk();
        self.node
            .children_by_field_name(name, &mut cursor)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|node| TsNodeImpl {
                node,
                src: self.src,
            })
    }

    fn kind(&self) -> &'static str {
        self.node.kind()
    }

    fn src(&self) -> &'de str {
        &self.src[self.node.byte_range()]
    }
}

pub fn show_node<'de, N: TsNode<'de>>(node: &N) {
    fn show<'de, N: TsNode<'de>>(node: &N, indent: usize) {
        let indent_string = " ".to_string().repeat(indent * 2);
        print!("{indent_string}");
        println!("- {}", node.kind());
        for i in 0..node.named_child_count() {
            show(&node.named_child(i).unwrap(), indent + 1);
        }
    }
    show(node, 0);
}

pub fn from_tree<'d, D: serde::Deserialize<'d>>(
    tree: &'d tree_sitter::Tree,
    src: &'d str,
) -> Result<D, DeserializeError> {
    from_node(tree.root_node(), src)
}

pub fn from_node<'de, D: serde::Deserialize<'de>>(
    node: tree_sitter::Node<'de>,
    src: &'de str,
) -> Result<D, DeserializeError> {
    let deserializer = NodeDeserializer::new(TsNodeImpl { node, src });
    D::deserialize(deserializer)
}

#[cfg(test)]
mod test {
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
        D::deserialize(NodeDeserializer::new(node))
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

    #[test]
    fn unit() {
        assert_eq!(deserialize::<()>(&make_node!(root)).unwrap(), ());
    }

    #[test]
    fn unit_struct() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root;

        assert_eq!(deserialize::<Root>(&make_node!(root)).unwrap(), Root);
        assert_eq!(
            deserialize::<Root>(&make_node!(expr)).unwrap_err(),
            DeserializeError::NodeType {
                expected: "root".into(),
                actual: "expr".into()
            }
        );
    }

    #[test]
    fn tuple_struct() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root();

        assert_eq!(
            deserialize::<Root>(&make_node!(root)).unwrap_err(),
            DeserializeError::TupleStructNotSupported,
        )
    }

    #[test]
    fn tuple() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "child")]
        struct Child;

        assert_eq!(
            deserialize::<(Child,)>(&make_node!(root(child))).unwrap(),
            (Child,)
        );
        assert_eq!(
            deserialize::<(Child,)>(&make_node!(root)).unwrap_err(),
            DeserializeError::ChildCount {
                expected: 1,
                actual: 0
            }
        );
        assert_eq!(
            deserialize::<(Child,)>(&make_node!(root(child)(child))).unwrap_err(),
            DeserializeError::ChildCount {
                expected: 1,
                actual: 2
            }
        );
    }

    #[test]
    fn vec() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "child")]
        struct Child;

        assert_eq!(
            deserialize::<Vec<Child>>(&make_node!(root)).unwrap(),
            Vec::new()
        );
        assert_eq!(
            deserialize::<Vec<Child>>(&make_node!(root(child))).unwrap(),
            vec![Child]
        );
        assert_eq!(
            deserialize::<Vec<Child>>(&make_node!(root(child)(child))).unwrap(),
            vec![Child, Child]
        );
    }

    #[test]
    fn newtype_struct_tuple() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root((Child, Child));

        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "child")]
        struct Child;

        assert_eq!(
            deserialize::<Root>(&make_node!(root(child)(child))).unwrap(),
            Root((Child, Child))
        );
        assert_eq!(
            deserialize::<Root>(&make_node!(foo(child)(child))).unwrap_err(),
            DeserializeError::node_type("root", "foo")
        );
        assert_eq!(
            deserialize::<Root>(&make_node!(root(child))).unwrap_err(),
            DeserializeError::ChildCount {
                expected: 2,
                actual: 1
            }
        );
    }

    #[test]
    fn newtype_struct_vec() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "root")]
        struct Root(Vec<Child>);

        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        #[serde(rename = "child")]
        struct Child;

        assert_eq!(
            deserialize::<Root>(&make_node!(root)).unwrap(),
            Root(vec![])
        );
        assert_eq!(
            deserialize::<Root>(&make_node!(root(child)(child))).unwrap(),
            Root(vec![Child, Child])
        );
        assert_eq!(
            deserialize::<Root>(&make_node!(foo)).unwrap_err(),
            DeserializeError::node_type("root", "foo")
        );
    }

    #[test]
    fn value_num() {
        assert_eq!(deserialize::<u32>(&make_node!(aaa "123")).unwrap(), 123);
        assert_eq!(
            deserialize::<u32>(&make_node!(aaa "foo")).unwrap_err(),
            "foo"
                .parse::<u32>()
                .map_err(DeserializeError::ParseIntError)
                .unwrap_err()
        );
        assert_eq!(deserialize::<f32>(&make_node!(aaa "123")).unwrap(), 123f32);
        assert_eq!(
            deserialize::<f32>(&make_node!(aaa "foo")).unwrap_err(),
            "foo"
                .parse::<f32>()
                .map_err(DeserializeError::ParseFloatError)
                .unwrap_err()
        );
    }

    #[test]
    fn value_borrowed_str() {
        assert_eq!(deserialize::<&str>(&make_node!(aaa "foo")).unwrap(), "foo",);
    }

    #[test]
    fn value_string() {
        assert_eq!(
            deserialize::<String>(&make_node!(aaa "foo")).unwrap(),
            "foo",
        );
    }

    #[test]
    fn value_borrowed_bytes() {
        assert_eq!(
            deserialize::<&[u8]>(&make_node!(aaa "foo")).unwrap(),
            "foo".as_bytes(),
        );
    }

    #[test]
    fn r#enum() {
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
            DeserializeError::child_count(2, 1),
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
    fn r#struct() {
        #[derive(serde::Deserialize, PartialEq, Eq, Debug)]
        struct Foo {
            a: u64,
            b: Option<String>,
            c: Vec<bool>,
            d: String,
        }
        assert_eq!(
            deserialize::<Foo>(&make_node!(foo
                 a: (int "123")
                 b: (str "foo")
                 c: (bool "true")
                 c: (bool "false")
                 d: (str "bar")
            ))
            .unwrap(),
            Foo {
                a: 123,
                b: Some("foo".into()),
                c: vec![true, false],
                d: "bar".into(),
            }
        );
    }

    #[test]
    fn json() {
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
            String(StringContainer),
            Null,
        }

        #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        struct Pair {
            key: StringContainer,
            value: Box<Value>,
        }

        let src = r#"
        {
            "foo": 123,
            "bar": 4.5,
            "baz": [null, 0, ""]
        }
        {}
        "#;

        #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
        #[serde(rename = "string")]
        struct StringContainer(Option<StringContent>);

        #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
        #[serde(rename = "string_content")]
        struct StringContent(String);

        let tree = parser.parse(src, None).unwrap();
        show_node(&TsNodeImpl {
            node: tree.root_node(),
            src,
        });
        let ast: Document = from_tree(&tree, src).unwrap();

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
                            Value::String(StringContainer(None)),
                        ]))
                    }
                ]),
                Value::Object(vec![]),
            ])
        );
    }
}
