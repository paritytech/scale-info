use crate::{
	TypeId,
	HasTypeId,
	TypeDef,
	HasTypeDef,
	RegisterSubtypes,
	Metadata,
	Registry,
};

#[derive(Clone, Copy)]
pub struct TraitTable {
	fn_type_id: fn() -> TypeId,
	fn_type_def: fn() -> TypeDef,
	fn_register_subtypes: fn(&mut Registry),
}

impl TraitTable {
	pub fn new<T>() -> Self
	where
		T: Metadata,
	{
		Self {
			fn_type_id: <T as HasTypeId>::type_id,
			fn_type_def: <T as HasTypeDef>::type_def,
			fn_register_subtypes: <T as RegisterSubtypes>::register_subtypes,
		}
	}

	pub fn type_id(&self) -> TypeId {
		(self.fn_type_id)()
	}

	pub fn type_def(&self) -> TypeDef {
		(self.fn_type_def)()
	}

	pub fn register_subtypes(&self, registry: &mut Registry) {
		(self.fn_register_subtypes)(registry)
	}
}

pub struct NoTraitTable;
