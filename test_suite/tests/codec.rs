// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

#![allow(unused)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

use pretty_assertions::{assert_eq, assert_ne};
use scale::{Encode, Decode};
use scale_info::{form::CompactForm, MetaType, IntoCompact as _, Registry, RegistryReadOnly, TypeInfo};

#[derive(TypeInfo)]
struct A<T> {
	a: bool,
	b: Result<char, u32>,
	c: T
}

#[derive(TypeInfo)]
enum B {
	A,
	B(A<bool>),
	C {
		d: [u8; 32]
	}
}

#[test]
fn encode_decode_to_readonly() {
	let mut registry = Registry::new();
	registry.register_type(&MetaType::new::<A<B>>());

	let mut encoded = registry.encode();
	// let original_serialized = serde_json::to_value(registry).unwrap();
	//
	// let readonly_decoded = RegistryReadOnly::decode(&mut &encoded[..]).unwrap();
	// let decoded_serialized = serde_json::to_value(readonly_decoded).unwrap();
	//
	// assert_eq!(decoded_serialized, original_serialized);
}
