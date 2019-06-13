use derive_more::From;
use serde::Serialize;

pub trait HasTypeId {
	fn type_id() -> TypeId;
}

/// Represents the namespace of a type definition.
///
/// This consists of several segments that each have to be a valid Rust identifier.
/// The first segment represents the crate name in which the type has been defined.
///
/// Rust prelude type may have an empty namespace definition.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct Namespace {
    /// The segments of the namespace.
	segments: Vec<&'static str>,
}

/// An error that may be encountered upon constructing namespaces.
#[derive(PartialEq, Eq, Debug)]
pub enum NamespaceError {
    /// If the module path does not at least have one segment.
    MissingSegments,
    /// If a segment within a module path is not a proper Rust identifier.
    InvalidIdentifier {
        /// The index of the errorneous segment.
        segment: usize,
    },
}

impl Namespace {
	/// Creates a new namespace from the given segments.
	pub fn new<S>(segments: S) -> Result<Self, NamespaceError>
	where
		S: IntoIterator<Item = &'static str>,
	{
		let segments = segments.into_iter().collect::<Vec<_>>();
		if segments.len() == 0 {
			return Err(NamespaceError::MissingSegments)
        }
        if let Some(err_at) = segments.iter().position(|seg| {
            /// Returns `true` if the given string is a proper Rust identifier.
            fn is_rust_identifier(s: &str) -> bool {
                // Only ascii encoding is allowed.
                // Note: Maybe this check is superseeded by the `head` and `tail` check.
                println!("is_rust_identifier? \"{}\"", s);
                if !s.is_ascii() {
                    return false
                }
                if let Some((&head, tail)) = s.as_bytes().split_first() {
                    // Check if head and tail make up a proper Rust identifier.
                    let head_ok =
                        head == b'_' ||
                        head >= b'a' && head <= b'z' ||
                        head >= b'A' && head <= b'Z';
                    let tail_ok =
                        tail.iter().all(|&ch| {
                            ch == b'_' ||
                            ch >= b'a' && ch <= b'z' ||
                            ch >= b'A' && ch <= b'Z' ||
                            ch >= b'0' && ch <= b'9'
                        });
                    head_ok && tail_ok
                } else {
                    // String is empty and thus not a valid Rust identifier.
                    false
                }
            }
            !is_rust_identifier(seg)
        }) {
            return Err(NamespaceError::InvalidIdentifier { segment: err_at })
        }
		Ok(Self { segments })
	}

	/// Creates a new namespace from the given module path.
	///
	/// # Note
	///
	/// Module path is generally obtained from the `module_path!` Rust macro.
	pub fn from_str(module_path: &'static str) -> Result<Self, NamespaceError> {
		Self::new(module_path.split("::"))
	}

	/// Creates the prelude namespace.
	pub fn prelude() -> Self {
		Self { segments: vec![] }
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, From, Serialize)]
pub enum TypeId {
	Custom(TypeIdCustom),
	Slice(TypeIdSlice),
	Array(TypeIdArray),
	Tuple(TypeIdTuple),
	Primitive(TypeIdPrimitive),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeIdPrimitive {
	Bool,
	Str,
	U8,
	U16,
	U32,
	U64,
	U128,
	I8,
	I16,
	I32,
	I64,
	I128,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct TypeIdCustom {
	name: &'static str,
	namespace: Namespace,
	#[serde(rename = "type")]
	type_params: Vec<TypeId>,
}

impl TypeIdCustom {
	pub fn new<T>(name: &'static str, namespace: Namespace, type_params: T) -> Self
	where
		T: IntoIterator<Item = TypeId>,
	{
		Self {
			name,
			namespace,
			type_params: type_params.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct TypeIdArray {
	pub len: u16,
	#[serde(rename = "type")]
	pub type_param: Box<TypeId>,
}

impl TypeIdArray {
	pub fn new<T>(len: u16, type_param: T) -> Self
	where
		T: Into<TypeId>,
	{
		Self {
			len,
			type_param: Box::new(type_param.into()),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct TypeIdTuple {
	#[serde(rename = "type")]
	pub type_params: Vec<TypeId>,
}

impl TypeIdTuple {
	pub fn new<T>(type_params: T) -> Self
	where
		T: IntoIterator<Item = TypeId>,
	{
		Self {
			type_params: type_params.into_iter().collect(),
		}
	}

	pub fn unit() -> Self {
		Self::new(vec![])
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct TypeIdSlice {
	#[serde(rename = "type")]
	type_param: Box<TypeId>,
}

impl TypeIdSlice {
	pub fn new<T>(type_param: T) -> Self
	where
		T: Into<TypeId>,
	{
		Self {
			type_param: Box::new(type_param.into()),
		}
	}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_ok() {
        assert_eq!(
            Namespace::new(vec!["hello"]),
            Ok(Namespace { segments: vec!["hello"] })
        );
        assert_eq!(
            Namespace::new(vec!["Hello", "World"]),
            Ok(Namespace { segments: vec!["Hello", "World"] })
        );
        assert_eq!(
            Namespace::new(vec!["_"]),
            Ok(Namespace { segments: vec!["_"] })
        );
    }

    #[test]
    fn namespace_err() {
        assert_eq!(
            Namespace::new(vec![]),
            Err(NamespaceError::MissingSegments)
        );
        assert_eq!(
            Namespace::new(vec![""]),
            Err(NamespaceError::InvalidIdentifier { segment: 0 })
        );
        assert_eq!(
            Namespace::new(vec!["1"]),
            Err(NamespaceError::InvalidIdentifier { segment: 0 })
        );
        assert_eq!(
            Namespace::new(vec!["Hello", ", World!"]),
            Err(NamespaceError::InvalidIdentifier { segment: 1 })
        );
    }

    #[test]
    fn namespace_from_str() {
        assert_eq!(
            Namespace::from_str("hello::world"),
            Ok(Namespace { segments: vec!["hello", "world"] })
        );
        assert_eq!(
            Namespace::from_str("::world"),
            Err(NamespaceError::InvalidIdentifier { segment: 0 })
        );
    }
}
