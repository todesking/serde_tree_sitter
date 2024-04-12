use crate::{access::FieldsAsSeqAccess, DeserializeError, TsNode};
use std::marker::PhantomData;

pub struct NodeDeserializer<'de, N: TsNode<'de>> {
    node: N,
    _p: PhantomData<&'de N>,
}
impl<'de, N: TsNode<'de>> NodeDeserializer<'de, N> {
    fn parse_int<T: std::str::FromStr<Err = std::num::ParseIntError>>(
        &self,
    ) -> Result<T, DeserializeError> {
        self.node
            .src()
            .parse::<T>()
            .map_err(DeserializeError::ParseIntError)
    }
    fn parse_float<T: std::str::FromStr<Err = std::num::ParseFloatError>>(
        &self,
    ) -> Result<T, DeserializeError> {
        self.node
            .src()
            .parse::<T>()
            .map_err(DeserializeError::ParseFloatError)
    }
    fn parse_bool<T: std::str::FromStr<Err = std::str::ParseBoolError>>(
        &self,
    ) -> Result<T, DeserializeError> {
        self.node
            .src()
            .parse::<T>()
            .map_err(DeserializeError::ParseBoolError)
    }
    fn into_newtype_struct_deserializer(
        self,
    ) -> crate::deserializer::NewtypeStructDeserializer<'de, N> {
        super::NewtypeStructDeserializer::new(self.node)
    }
}

macro_rules! handle_primitive {
    ($name:ident, $parse:ident, $visit:ident) => {
        fn $name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            visitor.$visit(self.$parse()?)
        }
    };
}

impl<'de, N: TsNode<'de>> serde::Deserializer<'de> for NodeDeserializer<'de, N> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(crate::access::SeqAccess::new(self.node.named_children()))
    }

    handle_primitive!(deserialize_bool, parse_bool, visit_bool);
    handle_primitive!(deserialize_u8, parse_int, visit_u8);
    handle_primitive!(deserialize_u16, parse_int, visit_u16);
    handle_primitive!(deserialize_u32, parse_int, visit_u32);
    handle_primitive!(deserialize_u64, parse_int, visit_u64);
    handle_primitive!(deserialize_i8, parse_int, visit_i8);
    handle_primitive!(deserialize_i16, parse_int, visit_i16);
    handle_primitive!(deserialize_i32, parse_int, visit_i32);
    handle_primitive!(deserialize_i64, parse_int, visit_i64);
    handle_primitive!(deserialize_f32, parse_float, visit_f32);
    handle_primitive!(deserialize_f64, parse_float, visit_f64);

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(DeserializeError::DataTypeNotSupported(
            "Data type `char` is not supported".into(),
        ))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.node.src())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_string(self.node.src().to_owned())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.node.src().as_bytes())
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(DeserializeError::DataTypeNotSupported(
            "Data type `byte_buf` is not supported".into(),
        ))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut children = self.node.named_children().collect::<Vec<_>>();
        match children.len() {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(NodeDeserializer::new(children.pop().unwrap())),
            n => Err(DeserializeError::child_count(1, n)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if name != self.node.kind() {
            return Err(DeserializeError::node_type(name, self.node.kind()));
        }
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if name != self.node.kind() {
            return Err(DeserializeError::node_type(name, self.node.kind()));
        }
        visitor.visit_newtype_struct(self.into_newtype_struct_deserializer())
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let seq_access = crate::access::SeqAccess::new(self.node.named_children());
        visitor.visit_seq(seq_access)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if len != self.node.named_child_count() {
            return Err(DeserializeError::ChildCount {
                expected: len,
                actual: self.node.named_child_count(),
            });
        }
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if name != self.node.kind() {
            return Err(DeserializeError::node_type(name, self.node.kind()));
        }
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(DeserializeError::DataTypeNotSupported(
            "Data type `map` is not supported".into(),
        ))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if name != self.node.kind() {
            return Err(DeserializeError::node_type(name, self.node.kind()));
        }
        visitor.visit_seq(FieldsAsSeqAccess::new(self.node, fields))
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
        let enum_access = crate::access::EnumAccess::new(self.node);
        visitor.visit_enum(enum_access)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.node.kind())
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }
}
impl<'de, N: TsNode<'de>> NodeDeserializer<'de, N> {
    pub fn new(node: N) -> NodeDeserializer<'de, N> {
        NodeDeserializer {
            node,
            _p: PhantomData,
        }
    }
}
