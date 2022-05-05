use alloc::vec::Vec;
use components_arena::{Arena, Component, NewtypeComponentId, Id};
use debug_panic::debug_panic;
use dyn_context::state::{RequiresStateDrop, SelfState, State, StateExt, StateDrop};
use educe::Educe;
use macro_attr_2018::macro_attr;
use crate::fw::{DepType, DetachedDepObjId};
use crate::dep_obj;

pub trait DepObjProps: DepType {
    fn new_priv() -> Self;
}

pub struct DepObjArena<Props: DepObjProps + 'static>(StateDrop<DepObjArena_<Props>>);

impl<Props: DepObjProps + 'static> SelfState for DepObjArena<Props> { }

struct DepObjArena_<Props>(Arena<DepObjComponent<Props>>);

impl<Props: DepObjProps + 'static> RequiresStateDrop for DepObjArena_<Props> {
    fn get(state: &dyn State) -> &StateDrop<Self> {
        &state.get::<DepObjArena<Props>>().0
    }

    fn get_mut(state: &mut dyn State) -> &mut StateDrop<Self> {
        &mut state.get_mut::<DepObjArena<Props>>().0
    }

    fn before_drop(state: &mut dyn State) {
        let items = Self::get(state).get().0.items().ids().map(DepObjComponentId).collect::<Vec<_>>();
        for item in items {
            item.drop_bindings_priv(state);
        }
    }

    fn drop_incorrectly(self) {
        debug_panic!("DepObjArena should be dropped with the drop_self method");
    }
}

impl<Props: DepObjProps + 'static> DepObjArena<Props> {
    pub fn new() -> Self {
        DepObjArena(StateDrop::new(DepObjArena_(Arena::new())))
    }

    pub fn drop_self(state: &mut dyn State) {
        <StateDrop<DepObjArena_<Props>>>::drop_self(state);
    }
}

macro_attr! {
    #[derive(Debug, Component!(class=DepObjComponentClass))]
    struct DepObjComponent<Props> {
        props: Props,
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct DepObjComponentId<Props>(Id<DepObjComponent<Props>>);
}

impl<Props> DetachedDepObjId for DepObjComponentId<Props> { }

impl<Props: DepObjProps + 'static> DepObjComponentId<Props> {
    pub fn new(state: &mut dyn State) -> Self {
        let arena: &mut DepObjArena<Props> = state.get_mut();
        arena.0.get_mut().0.insert(|id| (DepObjComponent { props: Props::new_priv() }, DepObjComponentId(id)))
    }

    pub fn drop_self(self, state: &mut dyn State) {
        self.drop_bindings_priv(state);
        let arena: &mut DepObjArena<Props> = state.get_mut();
        arena.0.get_mut().0.remove(self.0);
    }

    dep_obj! {
        pub fn props(self as this, arena: DepObjArena<Props>) -> (Props) {
            if mut {
                &mut arena.0.get_mut().0[this.0].props
            } else {
                &arena.0.get().0[this.0].props
            }
        }
    }
}
