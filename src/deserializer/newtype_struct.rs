use std::marker::PhantomData;

use crate::deserializer::NodeDeserializer;
use crate::tsnode::TsNode;
use crate::{access::SeqAccess, DeserializeError};

pub struct NewtypeStructDeserializer<'de, N: TsNode<'de>> {
    node: N,
    _p: PhantomData<&'de N>,
}

impl<'de, N: TsNode<'de>> NewtypeStructDeserializer<'de, N> {
    pub fn new(node: N) -> Self {
        Self {
            node,
            _p: PhantomData,
        }
    }
    fn try_into_single_child_deserializer(
        self,
    ) -> Result<NodeDeserializer<'de, N>, DeserializeError> {
        let mut children = self.node.named_children();
        if children.len() != 1 {
            return Err(DeserializeError::child_count(1, children.len()));
        }
        Ok(NodeDeserializer::new(children.next().unwrap()))
    }
    fn err_not_supported<T>(&self, name: &str) -> Result<T, DeserializeError> {
        Err(DeserializeError::DataTypeNotSupported(format!(
            "Method {} is not supported for newtype_struct member type",
            name
        )))
    }
}

macro_rules! not_supported {
    () => {};
    ($name:ident, $($rest:ident ,)*$(,)?) => {
        fn $name<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            self.err_not_supported(stringify!($name))
        }
        not_supported!($($rest,)*);
    };
}
macro_rules! delegate_to_node_serializer {
    () => {};
    ($name:ident, $($rest:ident ,)*$(,)?) => {
        fn $name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: serde::de::Visitor<'de> {
            NodeDeserializer::new(self.node).$name(visitor)
        }
        delegate_to_node_serializer!($($rest,)*);
    };
}

impl<'de, N: TsNode<'de>> serde::Deserializer<'de> for NewtypeStructDeserializer<'de, N> {
    type Error = DeserializeError;

    not_supported!(
        deserialize_any,
        deserialize_char,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_map,
        deserialize_identifier,
    );

    delegate_to_node_serializer!(
        deserialize_unit,
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
        deserialize_ignored_any,
        deserialize_str,
        deserialize_string,
    );

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.node.named_child_count() {
            0 => visitor.visit_none(),
            _ => visitor.visit_some(self.try_into_single_child_deserializer()?),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.err_not_supported("deserialize_unit_struct")
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.err_not_supported("deserialize_newtype_struct")
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess::new(self.node.named_children()))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let children = self.node.named_children();
        if len != children.len() {
            return Err(DeserializeError::child_count(len, children.len()));
        }
        visitor.visit_seq(crate::access::SeqAccess::new(children))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.err_not_supported("deserialize_newtype_struct")
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.err_not_supported("deserialize_struct")
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.err_not_supported("deserialize_enum")
    }
}
