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

use scale_info::{form::CompactForm, RegistryReadOnly, TypeDef, Type, TypeDefPrimitive, TypeDefComposite};
use proc_macro2::{TokenStream as TokenStream2};
use quote::{
    quote,
    format_ident,
};

// todo: [AJ] this could be a separate crate so can be used from other macros to generate e.g. all runtime types
pub fn generate(root_mod: &str, registry: &RegistryReadOnly) -> TokenStream2 {
    let mut tokens = TokenStream2::new();
    for (_, ty) in registry.enumerate() {
        ty.generate_type(&mut tokens, ty, registry);
    }
    let root_mod = format_ident!("{}", root_mod);

    quote! {
		// required that this be placed at crate root so can do ::registry_types.
		// alternatively use relative paths? more complicated
		mod #root_mod {
            #tokens
		}
	}
}

trait GenerateType {
    fn type_name(&self, ty: &Type<CompactForm>) -> String;
    fn generate_type(&self, tokens: &mut TokenStream2, ty: &Type<CompactForm>, registry: &RegistryReadOnly);
}

impl GenerateType for Type<CompactForm> {
    fn type_name(&self, ty: &Type<CompactForm>) -> String {
        self.type_def().type_name(ty)
    }

    fn generate_type(&self, tokens: &mut TokenStream2, ty: &Type<CompactForm>, registry: &RegistryReadOnly) {
        self.type_def().generate_type(tokens, ty, registry)
    }
}

impl GenerateType for TypeDef<CompactForm> {
    fn type_name(&self, ty: &Type<CompactForm>) -> String {
        match self {
            TypeDef::Composite(composite) => composite.type_name(ty),
            TypeDef::Variant(_) => todo!(),
            TypeDef::Sequence(_) => todo!(),
            TypeDef::Array(_) => todo!(),
            TypeDef::Tuple(_) => todo!(),
            TypeDef::Primitive(primitive) => primitive.type_name(ty)
        }
    }

    fn generate_type(&self, tokens: &mut TokenStream2, ty: &Type<CompactForm>, registry: &RegistryReadOnly) {
        match self {
            TypeDef::Composite(composite) => composite.generate_type(tokens, ty, registry),
            TypeDef::Variant(_) => {}
            TypeDef::Sequence(_) => {}
            TypeDef::Array(_) => {}
            TypeDef::Tuple(_) => {}
            TypeDef::Primitive(primitive) => primitive.generate_type(tokens, ty, registry)
        }
    }
}

impl GenerateType for TypeDefComposite<CompactForm> {
    fn type_name(&self, ty: &Type<CompactForm>) -> String {
        ty.path().ident().expect("structs should have a name")
    }

    fn generate_type(&self, tokens: &mut TokenStream2, ty: &Type<CompactForm>, registry: &RegistryReadOnly) {
        let named = self.fields().iter().all(|f| f.name().is_some());
        let unnamed = self.fields().iter().all(|f| f.name().is_none());
        let type_name = format_ident!("{}", self.type_name(ty));
        let ty_toks =
            if named {
                let fields =
                    self.fields()
                        .iter()
                        .map(|field| {
                            let name = format_ident!("{}", field.name().expect("named field without a name"));
                            let ty = registry.resolve(field.ty().id()).expect("type not resolved");
                            let ty = format_ident!("{}", ty.type_name(&ty));
                            quote! { pub #name: #ty }
                        });
                quote! {
                    pub struct #type_name {
                        #( #fields, )*
                    }
                }
            } else if unnamed {
                let fields =
                    self.fields()
                        .iter()
                        .map(|field| {
                            let ty = registry.resolve(field.ty().id()).expect("type not resolved");
                            let ty = format_ident!("{}", ty.type_name(&ty));
                            quote! { pub #ty }
                        });
                quote! {
                    pub struct #type_name (
                        #( #fields, )*
                    );
                }
            } else {
                panic!("Fields must be either all named or all unnamed")
            };
        tokens.extend(ty_toks);
    }
}

impl GenerateType for TypeDefPrimitive {
    fn type_name(&self, _ty: &Type<CompactForm>) -> String {
        match self {
            TypeDefPrimitive::Bool => "bool",
            TypeDefPrimitive::Char => "char",
            TypeDefPrimitive::Str => "String",
            TypeDefPrimitive::U8 => "u8",
            TypeDefPrimitive::U16 => "u16",
            TypeDefPrimitive::U32 => "u32",
            TypeDefPrimitive::U64 => "u64",
            TypeDefPrimitive::U128 => "u128",
            TypeDefPrimitive::U256 => unimplemented!("not a rust primitive"),
            TypeDefPrimitive::I8 => "i8",
            TypeDefPrimitive::I16 => "i16",
            TypeDefPrimitive::I32 => "i32",
            TypeDefPrimitive::I64 => "i64",
            TypeDefPrimitive::I128 => "i128",
            TypeDefPrimitive::I256 => unimplemented!("not a rust primitive"),
        }.to_owned()
    }

    fn generate_type(&self, _tokens: &mut TokenStream2, _ty: &Type<CompactForm>, _registry: &RegistryReadOnly) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale_info::{Registry, TypeInfo, meta_type};

    #[test]
    fn generate_struct_with_primitives() {
        #[allow(unused)]
        #[derive(TypeInfo)]
        struct S {
            a: bool,
            b: u32,
            c: char,
        }

        let mut registry = Registry::new();
        registry.register_type(&meta_type::<S>());

        let types = generate("root",&registry.into());

        assert_eq!(types.to_string(), quote! {
            mod root {
                pub struct S {
                    pub a: bool,
                    pub b: u32,
                    pub c: char,
                }
            }
        }.to_string())
    }

    #[test]
    fn generate_struct_with_a_struct_field() {
        #[allow(unused)]
        #[derive(TypeInfo)]
        struct Parent {
            a: bool,
            b: Child,
        }

        #[allow(unused)]
        #[derive(TypeInfo)]
        struct Child {
            a: i32,
        }

        let mut registry = Registry::new();
        registry.register_type(&meta_type::<Parent>());

        let types = generate("root",&registry.into());

        assert_eq!(types.to_string(), quote! {
            mod root {
                pub struct Parent {
                    pub a: bool,
                    pub b: Child,
                }

                pub struct Child {
                    pub a: i32,
                }
            }
        }.to_string())
    }

    #[test]
    fn generate_tuple_struct() {
        #[allow(unused)]
        #[derive(TypeInfo)]
        struct Parent(bool, Child);

        #[allow(unused)]
        #[derive(TypeInfo)]
        struct Child(i32);

        let mut registry = Registry::new();
        registry.register_type(&meta_type::<Parent>());

        let types = generate("root",&registry.into());

        assert_eq!(types.to_string(), quote! {
            mod root {
                pub struct Parent(pub bool, pub Child,);
                pub struct Child(pub i32,);
            }
        }.to_string())
    }
}


