// Copyright 2019-2020
//     Parity Technologies (UK) Ltd. Technologies (UK) Ltd.
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

#![no_std]

extern crate alloc;

use scale_info::TypeInfo;

#[allow(unused)]
#[derive(TypeInfo)]
struct UnitStruct;

#[allow(unused)]
#[derive(TypeInfo)]
struct TupleStruct(u128, bool);

#[allow(unused)]
#[derive(TypeInfo)]
struct Struct<T> {
	t: T,
}

#[allow(unused)]
#[derive(TypeInfo)]
enum CLike {
	A,
	B,
	C,
}

#[allow(unused)]
#[derive(TypeInfo)]
enum E<T> {
	A(T),
	B { b: T },
	C,
}

fn main() {
}
