use std::marker::PhantomData;

use crate::{deserializer::NodeDeserializer, tsnode::TsNode, DeserializeError};

pub struct SeqAccess<'de, N: TsNode<'de>, I: Iterator<Item = N>> {
    nodes: I,
    _p: PhantomData<&'de N>,
}

impl<'de, N: TsNode<'de>, I: Iterator<Item = N>> serde::de::SeqAccess<'de>
    for SeqAccess<'de, N, I>
{
    type Error = crate::DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let Some(n) = self.nodes.next() else {
            return Ok(None);
        };
        let v = seed.deserialize(NodeDeserializer::new(n))?;
        Ok(Some(v))
    }
}

impl<'de, N: TsNode<'de>, I: Iterator<Item = N>> SeqAccess<'de, N, I> {
    pub fn new(nodes: I) -> SeqAccess<'de, N, I> {
        SeqAccess {
            nodes,
            _p: PhantomData,
        }
    }
}

pub struct EnumAccess<'de, N: TsNode<'de>> {
    node: N,
    _p: PhantomData<&'de N>,
}
impl<'de, N: TsNode<'de>> EnumAccess<'de, N> {
    pub fn new(node: N) -> EnumAccess<'de, N> {
        EnumAccess {
            node,
            _p: PhantomData,
        }
    }
}
impl<'de, N: TsNode<'de>> serde::de::EnumAccess<'de> for EnumAccess<'de, N> {
    type Error = DeserializeError;

    type Variant = VariantAccess<'de, N>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = seed.deserialize(NodeDeserializer::new(self.node.clone()))?;
        let variant_access = VariantAccess::new(self.node);
        Ok((value, variant_access))
    }
}

pub struct VariantAccess<'de, N: TsNode<'de>> {
    node: N,
    _p: PhantomData<&'de N>,
}
impl<'de, N: TsNode<'de>> VariantAccess<'de, N> {
    pub fn new(node: N) -> VariantAccess<'de, N> {
        VariantAccess {
            node,
            _p: PhantomData,
        }
    }
}
impl<'de, N: TsNode<'de>> serde::de::VariantAccess<'de> for VariantAccess<'de, N> {
    type Error = DeserializeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        _seed.deserialize(crate::deserializer::NewtypeStructDeserializer::new(
            self.node,
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if self.node.named_child_count() != _len {
            return Err(DeserializeError::ChildCount {
                expected: _len,
                actual: self.node.named_child_count(),
            });
        }
        let seq = SeqAccess::new(self.node.named_children());
        _visitor.visit_seq(seq)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_seq(FieldsAsSeqAccess {
            node: self.node,
            fields: _fields,
            index: 0,
            _p: PhantomData,
        })
    }
}

pub struct FieldsAsSeqAccess<'de, N: TsNode<'de>> {
    node: N,
    fields: &'static [&'static str],
    index: usize,
    _p: PhantomData<&'de N>,
}
impl<'de, N: TsNode<'de>> FieldsAsSeqAccess<'de, N> {
    pub fn new(node: N, fields: &'static [&'static str]) -> Self {
        FieldsAsSeqAccess {
            node,
            fields,
            index: 0,
            _p: PhantomData,
        }
    }
}
impl<'de, N: TsNode<'de>> serde::de::SeqAccess<'de> for FieldsAsSeqAccess<'de, N> {
    type Error = DeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.fields.len() <= self.index {
            return Ok(None);
        }
        let field = self.fields[self.index];
        self.index += 1;
        let nodes = self.node.children_by_field_name(field);
        seed.deserialize(crate::deserializer::FieldDeserializer::new(
            field,
            nodes.collect(),
        ))
        .map(Some)
    }
}
