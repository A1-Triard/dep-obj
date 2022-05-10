use alloc::boxed::Box;
use alloc::vec::Vec;
use components_arena::NewtypeComponentId;
use components_arena::Component as components_arena_Component;
use components_arena::Arena as components_arena_Arena;
use components_arena::Id as components_arena_Id;
use debug_panic::debug_panic;
use downcast_rs::{Downcast, impl_downcast};
use dyn_context::state::{RequiresStateDrop, SelfState, State, StateExt, StateDrop};
use educe::Educe;
use macro_attr_2018::macro_attr;
use crate::{DepType, DetachedDepObjId, dep_obj};

pub enum ObjKey { }

pub trait Obj<P>: Downcast + DepType<Id=Id<P>, DepObjKey=ObjKey> { }

impl_downcast!(Obj<P>);

#[derive(Educe)]
#[educe(Default)]
pub struct Arena<P: 'static>(StateDrop<ArenaImpl<P>>);

impl<P: 'static> SelfState for Arena<P> { }

#[derive(Educe)]
#[educe(Default)]
struct ArenaImpl<P>(components_arena_Arena<Component<P>>);

impl<P: 'static> RequiresStateDrop for ArenaImpl<P> {
    fn get(state: &dyn State) -> &StateDrop<Self> {
        &state.get::<Arena<P>>().0
    }

    fn get_mut(state: &mut dyn State) -> &mut StateDrop<Self> {
        &mut state.get_mut::<Arena<P>>().0
    }

    fn before_drop(state: &mut dyn State) {
        let items = Self::get(state).get().0.items().ids().map(Id).collect::<Vec<_>>();
        for item in items {
            item.drop_bindings_priv(state);
        }
    }

    fn drop_incorrectly(self) {
        debug_panic!("Arena should be dropped with the drop_self method");
    }
}

impl<P: 'static> Arena<P> {
    pub fn new() -> Self {
        Arena(StateDrop::new(ArenaImpl(components_arena_Arena::new())))
    }

    pub fn drop_self(state: &mut dyn State) {
        <StateDrop<ArenaImpl<P>>>::drop_self(state);
    }
}

macro_attr! {
    #[derive(Debug, components_arena_Component!(class=ComponentClass))]
    struct Component<P>(Box<dyn Obj<P>>);
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Id<P>(components_arena_Id<Component<P>>);
}

impl<P> DetachedDepObjId for Id<P> { }

impl<P: 'static> Id<P> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<T>(
        state: &mut dyn State,
        obj: impl FnOnce(Id<P>) -> (Box<dyn Obj<P>>, T),
        init: impl FnOnce(&mut dyn State, Id<P>)
    ) -> T {
        let arena: &mut Arena<P> = state.get_mut();
        let (id, result) = arena.0.get_mut().0.insert(|id| {
            let (obj, result) = obj(Id(id));
            (Component(obj), (Id(id), result))
        });
        init(state, id);
        result
    }

    pub fn drop_self(self, state: &mut dyn State) {
        self.drop_bindings_priv(state);
        let arena: &mut Arena<P> = state.get_mut();
        arena.0.get_mut().0.remove(self.0);
    }
}

dep_obj! {
    impl<P: 'static> Id<P> {
        ObjKey => fn(self as this, arena: Arena<P>) -> (trait Obj<P>) {
            if mut {
                arena.0.get_mut().0[this.0].0.as_mut()
            } else {
                arena.0.get().0[this.0].0.as_ref()
            }
        }
    }
}
