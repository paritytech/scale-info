// Copyright 2019 Centrality Investments Limited
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(test)]
use super::*;

#[test]
fn primitives_metadata_impl_should_work() {
	assert_eq!(bool::type_ident(), TypeIdent::new(IdentKind::Bool),);
	assert_eq!(String::type_ident(), TypeIdent::new(IdentKind::Str),);
	assert_eq!(<&str>::type_ident(), TypeIdent::new(IdentKind::Str),);
	assert_eq!(<()>::type_ident(), TypeIdent::new(IdentKind::Unit),);
	assert_eq!(i8::type_ident(), TypeIdent::new(IdentKind::I8),);

	assert_eq!(
		<Option<isize>>::type_ident(),
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Option,
			args: vec![TypeIdent::new(IdentKind::Isize)],
		}
	);
	assert_eq!(
		<Result<bool, String>>::type_ident(),
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Result,
			args: vec![TypeIdent::new(IdentKind::Bool), TypeIdent::new(IdentKind::Str)],
		}
	);
	assert_eq!(<Box<String>>::type_ident(), TypeIdent::new(IdentKind::Str));
	assert_eq!(<&String>::type_ident(), TypeIdent::new(IdentKind::Str));
	assert_eq!(
		<[usize]>::type_ident(),
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Vector,
			args: vec![TypeIdent::new(IdentKind::Usize)],
		}
	);
	assert_eq!(
		<std::marker::PhantomData<bool>>::type_ident(),
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Unit,
			args: vec![TypeIdent::new(IdentKind::Bool)],
		}
	)
}

#[test]
fn lists_metadata_impl_should_work() {
	// tuple
	assert_eq!(
		<(bool,)>::type_ident(),
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Tuple,
			args: vec![TypeIdent::new(IdentKind::Bool)],
		},
	);
	assert_eq!(
		<(bool, String)>::type_ident(),
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Tuple,
			args: vec![TypeIdent::new(IdentKind::Bool), TypeIdent::new(IdentKind::Str)],
		},
	);

	// array
	assert_eq!(
		<[usize; 3]>::type_ident(),
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Array(3),
			args: vec![TypeIdent::new(IdentKind::Usize)],
		},
	);

	// vec
	assert_eq!(
		<Vec<bool>>::type_ident(),
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Vector,
			args: vec![TypeIdent::new(IdentKind::Bool)],
		},
	);
}

#[test]
fn struct_with_generics_metadata_impl_should_work() {
	struct MyStruct<T> {
		data: T,
	}

	impl<T: Metadata> Metadata for MyStruct<T> {
		fn type_ident() -> TypeIdent {
			TypeIdent {
				namespace: vec!["MyTestMod".into()],
				ident: IdentKind::Custom("MyStruct".into()),
				args: vec![T::type_ident()],
			}
		}

		fn type_def(registry: &mut Registry) -> TypeDef {
			registry.register(T::type_ident(), T::type_def);
			TypeDef::Struct(vec![Field {
				name: FieldName::Named("data".into()),
				ident: T::type_ident(),
			}])
		}
	}

	// normal struct
	let struct_bool_ident = TypeIdent {
		namespace: vec!["MyTestMod".into()],
		ident: IdentKind::Custom("MyStruct".into()),
		args: vec![TypeIdent::new(IdentKind::Bool)],
	};
	assert_eq!(<MyStruct<bool>>::type_ident(), struct_bool_ident);
	let mut registry = Registry::new();
	let struct_bool_def = TypeDef::Struct(vec![Field {
		name: FieldName::Named("data".into()),
		ident: TypeIdent::new(IdentKind::Bool),
	}]);
	assert_eq!(<MyStruct<bool>>::type_def(&mut registry), struct_bool_def);

	// with "`Self` typed" fields
	type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
	assert_eq!(
		SelfTyped::type_ident(),
		TypeIdent {
			namespace: vec!["MyTestMod".into()],
			ident: IdentKind::Custom("MyStruct".into()),
			args: vec![struct_bool_ident.clone()],
		},
	);
	assert_eq!(
		SelfTyped::type_def(&mut registry),
		TypeDef::Struct(vec![Field {
			name: FieldName::Named("data".into()),
			ident: struct_bool_ident,
		}]),
	)
}
