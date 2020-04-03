// Copyright 2019-2020
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
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
#![feature(alloc_error_handler, lang_items, start)]

#[start]
fn start(_argc: isize, _argv: *const *const u8) -> isize {
	0
}

#[lang = "eh_personality"]
pub extern "C" fn rust_eh_personality() {}
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
	unsafe {
		libc::abort();
	}
}
#[alloc_error_handler]
fn error_handler(_: core::alloc::Layout) -> ! {
	unsafe {
		libc::abort();
	}
}

extern crate alloc;
use alloc::alloc::{GlobalAlloc, Layout};

pub struct Allocator;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

extern "C" {
	fn ext_malloc(size: usize) -> *mut u8;
	fn ext_free(ptr: *mut u8);
}
unsafe impl GlobalAlloc for Allocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		ext_malloc(layout.size()) as *mut u8
	}
	unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
		ext_free(ptr as *mut u8)
	}
}

use scale_info::Metadata;

#[allow(unused)]
#[derive(Metadata)]
struct UnitStruct;

#[allow(unused)]
#[derive(Metadata)]
struct TupleStruct(u128, bool);

#[allow(unused)]
#[derive(Metadata)]
struct Struct<T> {
	t: T,
}

#[allow(unused)]
#[derive(Metadata)]
enum CLike {
	A,
	B,
	C,
}

#[allow(unused)]
#[derive(Metadata)]
enum E<T> {
	A(T),
	B { b: T },
	C,
}
