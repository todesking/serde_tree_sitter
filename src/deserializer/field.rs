use std::marker::PhantomData;

use crate::{DeserializeError, NodeDeserializer, TsNode};

pub struct FieldDeserializer<'de, N: TsNode<'de>> {
    field_name: &'static str,
    nodes: Vec<N>,
    _p: PhantomData<&'de N>,
}
impl<'de, N: TsNode<'de>> FieldDeserializer<'de, N> {
    pub fn new(field_name: &'static str, nodes: Vec<N>) -> Self {
        FieldDeserializer {
            field_name,
            nodes,
            _p: PhantomData,
        }
    }
    fn delegate<F, R>(mut self, f: F) -> Result<R, DeserializeError>
    where
        F: FnOnce(NodeDeserializer<'de, N>) -> Result<R, DeserializeError>,
    {
        if self.nodes.len() != 1 {
            return Err(DeserializeError::field_length(
                self.field_name,
                1,
                self.nodes.len(),
            ));
        }
        f(NodeDeserializer::new(self.nodes.pop().unwrap()))
    }
}

macro_rules! delegate_to_node_deserializer {
    () => {};
    ($name:ident, $($rest:ident ,)*$(,)?) => {
        fn $name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            self.delegate(move |de| de.$name(visitor))
        }
        delegate_to_node_deserializer!($($rest,)*);
    };
}

impl<'de, N: TsNode<'de>> serde::de::Deserializer<'de> for FieldDeserializer<'de, N> {
    type Error = DeserializeError;

    delegate_to_node_deserializer!(
        deserialize_any,
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_str,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_string,
        deserialize_map,
        deserialize_unit,
        deserialize_identifier,
        deserialize_ignored_any,
    );

    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.nodes.len() {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(NodeDeserializer::new(self.nodes.pop().unwrap())),
            n => Err(DeserializeError::field_length(self.field_name, 1, n)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.delegate(move |de| de.deserialize_unit_struct(_name, visitor))
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.delegate(move |de| de.deserialize_newtype_struct(_name, visitor))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(crate::access::SeqAccess::new(self.nodes.into_iter()))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.delegate(move |de| de.deserialize_tuple(_len, visitor))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.delegate(move |de| de.deserialize_tuple_struct(_name, _len, visitor))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.delegate(|de| de.deserialize_struct(_name, _fields, visitor))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.delegate(move |de| de.deserialize_enum(_name, _variants, visitor))
    }
}
