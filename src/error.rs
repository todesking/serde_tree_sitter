use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DeserializeError {
    #[error("Child count not match: expected={expected}, actual={actual}")]
    ChildLength { expected: usize, actual: usize },
    #[error("Node count not match(field = {field_name}): expected={expected}, actual={actual}")]
    FieldLength {
        field_name: &'static str,
        expected: usize,
        actual: usize,
    },
    #[error("Node type not match: expected={expected}, actual={actual}")]
    NodeType { expected: String, actual: String },
    #[error("{0}")]
    DataTypeNotSupported(String),
    #[error(transparent)]
    ParseIntError(std::num::ParseIntError),
    #[error(transparent)]
    ParseFloatError(std::num::ParseFloatError),
    #[error(transparent)]
    ParseBoolError(std::str::ParseBoolError),
    #[error("Tree-sitter node contain error(s)")]
    TreeSitterError(Vec<tree_sitter::Range>),
    #[error("{0}")]
    Custom(String),
}

impl DeserializeError {
    pub fn node_type<S1: Into<String>, S2: Into<String>>(
        expected: S1,
        actual: S2,
    ) -> DeserializeError {
        DeserializeError::NodeType {
            expected: expected.into(),
            actual: actual.into(),
        }
    }
    pub fn child_length(expected: usize, actual: usize) -> Self {
        DeserializeError::ChildLength { expected, actual }
    }
    pub fn field_length(field_name: &'static str, expected: usize, actual: usize) -> Self {
        DeserializeError::FieldLength {
            field_name,
            expected,
            actual,
        }
    }
}

impl serde::de::Error for DeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        // dbg!(std::backtrace::Backtrace::capture());
        DeserializeError::Custom(msg.to_string())
    }
}
