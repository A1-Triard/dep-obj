use alloc::vec::Vec;
use components_arena::NewtypeComponentId;
use components_arena::Component as components_arena_Component;
use components_arena::Arena as components_arena_Arena;
use components_arena::Id as components_arena_Id;
use debug_panic::debug_panic;
use dyn_context::state::{RequiresStateDrop, SelfState, State, StateExt, StateDrop};
use educe::Educe;
use macro_attr_2018::macro_attr;
use crate::{DetachedDepObjId, SizedDepType, dep_obj};

pub enum Obj { }

#[derive(Educe)]
#[educe(Default)]
pub struct Arena<P: SizedDepType + 'static>(StateDrop<ArenaImpl<P>>);

impl<P: SizedDepType + 'static> SelfState for Arena<P> { }

#[derive(Educe)]
#[educe(Default)]
struct ArenaImpl<P>(components_arena_Arena<Component<P>>);

impl<P: SizedDepType + 'static> RequiresStateDrop for ArenaImpl<P> {
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

impl<P: SizedDepType + 'static> Arena<P> {
    pub fn new() -> Self {
        Arena(StateDrop::new(ArenaImpl(components_arena_Arena::new())))
    }

    pub fn drop_self(state: &mut dyn State) {
        <StateDrop<ArenaImpl<P>>>::drop_self(state);
    }
}

macro_attr! {
    #[derive(Debug, components_arena_Component!(class=ComponentClass))]
    struct Component<P>(P);
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Id<P>(components_arena_Id<Component<P>>);
}

impl<P> DetachedDepObjId for Id<P> { }

impl<P: SizedDepType + 'static> Id<P> {
    pub fn new(state: &mut dyn State, init: impl FnOnce(&mut dyn State, Id<P>)) -> Self {
        let arena: &mut Arena<P> = state.get_mut();
        let id = arena.0.get_mut().0.insert(|id| (Component(P::new_priv()), Id(id)));
        init(state, id);
        id
    }

    pub fn drop_self(self, state: &mut dyn State) {
        self.drop_bindings_priv(state);
        let arena: &mut Arena<P> = state.get_mut();
        arena.0.get_mut().0.remove(self.0);
    }
}

dep_obj! {
    impl<P: SizedDepType + 'static> Id<P> {
        Obj => fn(self as this, arena: Arena<P>) -> (P) {
            if mut {
                &mut arena.0.get_mut().0[this.0].0
            } else {
                &arena.0.get().0[this.0].0
            }
        }
    }
}
