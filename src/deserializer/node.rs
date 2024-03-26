use std::marker::PhantomData;

use crate::{access::FieldsAsSeqAccess, DeserializeError, TsNode};

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
}
impl<'de, N: TsNode<'de>> serde::Deserializer<'de> for NodeDeserializer<'de, N> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_seq(crate::access::SeqAccess::new(self.node.named_children()))
    }

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_i8(self.parse_int()?)
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_i16(self.parse_int()?)
    }

    fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_i32(self.parse_int()?)
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_u8(self.parse_int()?)
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_u16(self.parse_int()?)
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_u32(self.parse_int()?)
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_f32(self.parse_float()?)
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_f64(self.parse_float()?)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(DeserializeError::DataTypeNotSupported(
            "Data type `char` is not supported".into(),
        ))
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_borrowed_str(self.node.src())
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_string(self.node.src().to_owned())
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_borrowed_bytes(self.node.src().as_bytes())
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(DeserializeError::DataTypeNotSupported(
            "Data type `byte_buf` is not supported".into(),
        ))
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let mut children = self.node.named_children().collect::<Vec<_>>();
        match children.len() {
            0 => _visitor.visit_none(),
            1 => _visitor.visit_some(NodeDeserializer::new(children.pop().unwrap())),
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
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.debug_print("deserialize_seq()");
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
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(DeserializeError::TupleStructNotSupported)
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
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_seq(FieldsAsSeqAccess::new(self.node, _fields))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.debug_print(&format!("deserialize_enum({name}, {variants:?})"));
        let enum_access = crate::access::EnumAccess::new(self.node);
        visitor.visit_enum(enum_access)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.debug_print("deserialize_identifier()");
        visitor.visit_borrowed_str(self.node.kind())
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        _visitor.visit_unit()
    }
}
impl<'de, N: TsNode<'de>> NodeDeserializer<'de, N> {
    pub fn new(node: N) -> NodeDeserializer<'de, N> {
        let d = NodeDeserializer {
            node,
            _p: PhantomData,
        };
        d.debug_print("new()");
        d
    }
    fn debug_print(&self, msg: &str) {
        println!(
            "{} - node(kind={}) src={}",
            msg,
            self.node.kind(),
            self.node
                .src()
                .chars()
                .filter(|x| *x != '\n')
                .take(10)
                .collect::<String>()
        );
    }
}
