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
	assert_eq!(bool::type_ident(), IdentKind::Bool);
	assert_eq!(String::type_ident(), IdentKind::Str);
	assert_eq!(<&str>::type_ident(), IdentKind::Str);
	assert_eq!(i8::type_ident(), IdentKind::I8);

	assert_eq!(
		<Option<u128>>::type_ident(),
		IdentKind::Option(OptionIdent::new(IdentKind::U128)),
	);
	assert_eq!(
		<Result<bool, String>>::type_ident(),
		IdentKind::Result(ResultIdent::new((IdentKind::Bool, IdentKind::Str))),
	);
	assert_eq!(<Box<String>>::type_ident(), IdentKind::Str);
	assert_eq!(<&String>::type_ident(), IdentKind::Str);
	assert_eq!(
		<[bool]>::type_ident(),
		IdentKind::Slice(SliceIdent::new(IdentKind::Bool)),
	);
	assert_eq!(
		<std::marker::PhantomData<bool>>::type_ident(),
		IdentKind::Tuple(TupleIdent::new(vec![IdentKind::Bool])),
	)
}

#[test]
fn lists_metadata_impl_should_work() {
	// unit
	assert_eq!(<()>::type_ident(), IdentKind::Tuple(TupleIdent::new(vec![])),);
	// tuple with one element
	assert_eq!(
		<(bool,)>::type_ident(),
		IdentKind::Tuple(TupleIdent::new(vec![IdentKind::Bool])),
	);
	// tuple with multiple elements
	assert_eq!(
		<(bool, String)>::type_ident(),
		IdentKind::Tuple(TupleIdent::new(vec![IdentKind::Bool, IdentKind::Str])),
	);

	// array
	assert_eq!(
		<[bool; 3]>::type_ident(),
		IdentKind::Array(ArrayIdent::new(3, IdentKind::Bool)),
	);

	// vec
	assert_eq!(
		<Vec<bool>>::type_ident(),
		IdentKind::Slice(SliceIdent::new(IdentKind::Bool)),
	);
}

#[test]
fn struct_with_generics_metadata_impl_should_work() {
	struct MyStruct<T> {
		data: T,
	}

	impl<T: Metadata> Metadata for MyStruct<T> {
		fn type_ident() -> IdentKind {
			IdentKind::Custom(CustomIdent {
				name: "MyStruct",
				namespace: Namespace::new(vec!["MyTestMod"]),
				type_params: vec![T::type_ident()],
			})
		}

		fn type_def(registry: &mut Registry) -> TypeDef {
			registry.register(T::type_ident(), T::type_def);
			TypeDef::Struct(StructDef(vec![Field {
				name: "data",
				ident: T::type_ident(),
			}]))
		}
	}

	// normal struct
	let struct_bool_ident = IdentKind::Custom(CustomIdent {
		name: "MyStruct",
		namespace: Namespace::new(vec!["MyTestMod"]),
		type_params: vec![IdentKind::Bool],
	});
	assert_eq!(<MyStruct<bool>>::type_ident(), struct_bool_ident);
	let mut registry = Registry::new();
	let struct_bool_def = TypeDef::Struct(StructDef(vec![Field {
		name: "data",
		ident: IdentKind::Bool,
	}]));
	assert_eq!(<MyStruct<bool>>::type_def(&mut registry), struct_bool_def);

	// with "`Self` typed" fields
	type SelfTyped = MyStruct<Box<MyStruct<bool>>>;
	assert_eq!(
		SelfTyped::type_ident(),
		IdentKind::Custom(CustomIdent {
			name: "MyStruct",
			namespace: Namespace::new(vec!["MyTestMod"]),
			type_params: vec![struct_bool_ident.clone()],
		}),
	);
	assert_eq!(
		SelfTyped::type_def(&mut registry),
		TypeDef::Struct(StructDef(vec![Field {
			name: "data",
			ident: struct_bool_ident,
		}])),
	);
}
