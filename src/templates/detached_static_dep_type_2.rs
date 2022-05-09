use alloc::vec::Vec;
use components_arena::NewtypeComponentId;
use components_arena::Component as components_arena_Component;
use components_arena::Arena as components_arena_Arena;
use components_arena::Id as components_arena_Id;
use debug_panic::debug_panic;
use dyn_context::state::{RequiresStateDrop, SelfState, State, StateExt, StateDrop};
use educe::Educe;
use macro_attr_2018::macro_attr;
use crate::fw::{DepType, DetachedDepObjId, NewPriv};
use crate::dep_obj;

pub enum Obj1 { }

pub enum Obj2 { }

pub trait NewDepType: NewPriv + DepType { }

impl<T: NewPriv + DepType> NewDepType for T { }

#[derive(Educe)]
#[educe(Default)]
pub struct Arena<P1: NewDepType + 'static, P2: NewDepType + 'static>(StateDrop<ArenaImpl<P1, P2>>);

impl<P1: NewDepType + 'static, P2: NewDepType + 'static> SelfState for Arena<P1, P2> { }

#[derive(Educe)]
#[educe(Default)]
struct ArenaImpl<P1, P2>(components_arena_Arena<Component<P1, P2>>);

impl<P1: NewDepType + 'static, P2: NewDepType + 'static> RequiresStateDrop for ArenaImpl<P1, P2> {
    fn get(state: &dyn State) -> &StateDrop<Self> {
        &state.get::<Arena<P1, P2>>().0
    }

    fn get_mut(state: &mut dyn State) -> &mut StateDrop<Self> {
        &mut state.get_mut::<Arena<P1, P2>>().0
    }

    fn before_drop(state: &mut dyn State) {
        let ids = Self::get(state).get().0.items().ids().map(Id).collect::<Vec<_>>();
        for id in ids {
            id.drop_bindings_priv(state);
        }
    }

    fn drop_incorrectly(self) {
        debug_panic!("Arena should be dropped with the drop_self method");
    }
}

impl<P1: NewDepType + 'static, P2: NewDepType + 'static> Arena<P1, P2> {
    pub fn new() -> Self {
        Arena(StateDrop::new(ArenaImpl(components_arena_Arena::new())))
    }

    pub fn drop_self(state: &mut dyn State) {
        <StateDrop<ArenaImpl<P1, P2>>>::drop_self(state);
    }
}

macro_attr! {
    #[derive(Debug, components_arena_Component!(class=ComponentClass))]
    struct Component<P1, P2>(P1, P2);
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Id<P1, P2>(components_arena_Id<Component<P1, P2>>);
}

impl<P1, P2> DetachedDepObjId for Id<P1, P2> { }

impl<P1: NewDepType + 'static, P2: NewDepType + 'static> Id<P1, P2> {
    pub fn new(state: &mut dyn State, init: impl FnOnce(&mut dyn State, Id<P1, P2>)) -> Self {
        let arena: &mut Arena<P1, P2> = state.get_mut();
        let id = arena.0.get_mut().0.insert(|id| (Component(P1::new_priv(), P2::new_priv()), Id(id)));
        init(state, id);
        id
    }

    pub fn drop_self(self, state: &mut dyn State) {
        self.drop_bindings_priv(state);
        let arena: &mut Arena<P1, P2> = state.get_mut();
        arena.0.get_mut().0.remove(self.0);
    }
}

dep_obj! {
    impl<P1: NewDepType + 'static, P2: NewDepType + 'static> Id<P1, P2> {
        Obj1 => fn(self as this, arena: Arena<P1, P2>) -> (P1) {
            if mut {
                &mut arena.0.get_mut().0[this.0].0
            } else {
                &arena.0.get().0[this.0].0
            }
        },

        Obj2 => fn(self as this, arena: Arena<P1, P2>) -> (P2) {
            if mut {
                &mut arena.0.get_mut().0[this.0].1
            } else {
                &arena.0.get().0[this.0].1
            }
        },
    }
}
