pub trait TsNode<'de>: Clone + std::fmt::Debug
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
pub struct TsNodeImpl<'a, 'de> {
    node: tree_sitter::Node<'a>,
    src: &'de str,
}
impl<'a, 'de> TsNodeImpl<'a, 'de> {
    pub fn new(node: tree_sitter::Node<'a>, src: &'de str) -> Self {
        Self { node, src }
    }
}

impl<'a, 'de> std::fmt::Debug for TsNodeImpl<'a, 'de> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TsNodeImpl")
            .field("kind", &self.kind())
            .field("named_children_count", &self.named_child_count())
            .field("children", &self.named_children().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
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

#[allow(dead_code)]
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
