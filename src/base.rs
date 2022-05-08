use components_arena::{RawId};
use core::any::{Any, TypeId};
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
use dyn_context::state::State;

/// A type should satisfy this trait to be a dependency property type,
/// a dependency vector item type, or a flow data type.
pub trait Convenient: PartialEq + Clone + Debug + 'static { }

impl<T: PartialEq + Clone + Debug + 'static> Convenient for T { }

pub struct Glob<Obj> {
    pub arena: TypeId,
    pub field_ref: fn(arena: &dyn Any, id: RawId) -> &Obj,
    pub field_mut: fn(arena: &mut dyn Any, id: RawId) -> &mut Obj
}

pub struct GlobRef<'a, Obj> {
    arena: &'a dyn Any,
    id: RawId,
    field_ref: fn(arena: &dyn Any, id: RawId) -> &Obj,
}

impl<'a, Obj> Deref for GlobRef<'a, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        (self.field_ref)(self.arena, self.id)
    }
}

pub struct GlobMut<'a, Obj> {
    arena: &'a mut dyn Any,
    id: RawId,
    field_ref: fn(arena: &dyn Any, id: RawId) -> &Obj,
    field_mut: fn(arena: &mut dyn Any, id: RawId) -> &mut Obj,
}

impl<'a, Obj> Deref for GlobMut<'a, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        (self.field_ref)(self.arena, self.id)
    }
}

impl<'a, Obj> DerefMut for GlobMut<'a, Obj> {
    fn deref_mut(&mut self) -> &mut Obj {
        (self.field_mut)(self.arena, self.id)
    }
}

impl<Obj> Glob<Obj> {
    pub fn get(self, state: &dyn State, id: RawId) -> GlobRef<Obj> {
        GlobRef {
            id,
            arena: state.get_raw(self.arena).unwrap_or_else(|| panic!("{:?} required", self.arena)),
            field_ref: self.field_ref,
        }
    }

    pub fn get_mut(self, state: &mut dyn State, id: RawId) -> GlobMut<Obj> {
        GlobMut {
            id,
            arena: state.get_mut_raw(self.arena).unwrap_or_else(|| panic!("{:?} required", self.arena)),
            field_ref: self.field_ref,
            field_mut: self.field_mut,
        }
    }
}
