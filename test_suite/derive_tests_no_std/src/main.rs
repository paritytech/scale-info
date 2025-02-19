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
#![allow(internal_features)]
#![feature(lang_items, alloc_error_handler)]
#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn _start() {
    test();

    use core::arch::asm;
    unsafe {
        asm!(
            "syscall",
            in("rax") 60,
            in("rdi") 0,
            options(noreturn)
        );
    }
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        libc::abort();
    }
}

use libc_alloc::LibcAlloc;

#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;

//////////////////////////////////////////////////////////////////////////////

// Note: Use the types in some way to make sure they are not pruned as dead code.
// If an assert fails we will get `Aborted (core dumped)`.
fn test() {
    assert_eq!(UnitStruct::type_info().type_params.len(), 0);
    assert_eq!(TupleStruct::type_info().type_params.len(), 0);
    assert_eq!(Struct::<TupleStruct>::type_info().type_params.len(), 1);
    assert_eq!(CLike::type_info().type_params.len(), 0);
    assert_eq!(E::<CLike>::type_info().type_params.len(), 1);
}

use bitvec::{order::Lsb0, vec::BitVec};
use scale::{Decode, Encode};
use scale_info::TypeInfo;

#[allow(unused)]
#[derive(TypeInfo, Decode, Encode)]
struct UnitStruct;

#[allow(unused)]
#[derive(TypeInfo, Decode, Encode)]
struct TupleStruct(u128, bool);

#[allow(unused)]
#[derive(TypeInfo, Decode, Encode)]
struct Struct<T> {
    t: T,
    bitvec: BitVec<u16, Lsb0>,
}

#[allow(unused)]
#[derive(TypeInfo, Decode, Encode)]
enum CLike {
    A,
    B,
    C,
}

#[allow(unused)]
#[derive(TypeInfo, Decode, Encode)]
enum E<T> {
    A(T),
    B { b: T },
    C,
}
