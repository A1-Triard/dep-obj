#![feature(const_ptr_offset_from)]
#![feature(const_type_id)]
#![feature(explicit_generic_args_with_impl_trait)]

#![deny(warnings)]
#![allow(dead_code)]

mod circuit {
    use components_arena::{Arena, Component, ComponentStop, Id, NewtypeComponentId, with_arena_in_state_part};
    use dep_obj::{DetachedDepObjId, DepType, impl_dep_obj};
    use downcast_rs::{Downcast, impl_downcast};
    use dyn_context::{SelfState, State, StateExt, Stop};
    use macro_attr_2018::macro_attr;

    pub enum ChipLegsKey { }

    pub trait ChipLegs: Downcast + DepType<Id=Chip> { }

    impl_downcast!(ChipLegs);

    macro_attr! {
        #[derive(Debug, Component!(stop=ChipStop))]
        struct ChipNode {
            legs: Box<dyn ChipLegs>,
        }
    }

    impl ComponentStop for ChipStop {
        with_arena_in_state_part!(Circuit);

        fn stop(&self, state: &mut dyn State, id: Id<ChipNode>) {
            Chip(id).drop_bindings_priv(state);
        }
    }

    macro_attr! {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, NewtypeComponentId!)]
        pub struct Chip(Id<ChipNode>);
    }

    impl Chip {
        pub fn new<T>(
            state: &mut dyn State,
            legs: impl FnOnce(Chip) -> (Box<dyn ChipLegs>, T)
        ) -> T {
            let circuit: &mut Circuit = state.get_mut();
            circuit.0.insert(|chip| {
                let (legs,  result) = legs(Chip(chip));
                (ChipNode { legs }, result)
            })
        }

        pub fn drop_self(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let circuit: &mut Circuit = state.get_mut();
            circuit.0.remove(self.0);
        }
    }

    impl_dep_obj!(Chip {
        trait ChipLegs => Circuit | .legs
    });

    impl DetachedDepObjId for Chip { }

    #[derive(Debug, Stop)]
    pub struct Circuit(Arena<ChipNode>);

    impl SelfState for Circuit { }

    impl Circuit {
        pub fn new() -> Self { Circuit(Arena::new()) }
    }
}

mod or_chip {
    use crate::circuit::*;
    use dep_obj::dep_type;
    use dep_obj::binding::Binding2;
    use dyn_context::State;

    dep_type! {
        #[derive(Debug)]
        pub struct OrLegs in Chip {
            in_1: bool = false,
            in_2: bool = false,
            out: bool = false,
        }
    }

    impl OrLegs {
        pub fn new(state: &mut dyn State) -> Chip {
            let chip = Chip::new(state, |chip| (Box::new(Self::new_priv()) as _, chip));
            let binding = Binding2::new(state, (), |(), in_1, in_2| Some(in_1 | in_2));
            OrLegs::OUT.bind(state, chip, binding);
            binding.set_source_1(state, &mut OrLegs::IN_1.value_source(chip));
            binding.set_source_2(state, &mut OrLegs::IN_2.value_source(chip));
            chip
        }
    }

    impl ChipLegs for OrLegs { }
}

mod not_chip {
    use crate::circuit::*;
    use dep_obj::dep_type;
    use dep_obj::binding::Binding1;
    use dyn_context::State;

    dep_type! {
        #[derive(Debug)]
        pub struct NotLegs in Chip {
            in_: bool = false,
            out: bool = true,
        }
    }

    impl NotLegs {
        pub fn new(state: &mut dyn State) -> Chip {
            let chip = Chip::new(state, |chip| (Box::new(Self::new_priv()) as _, chip));
            let binding = Binding1::new(state, (), |(), in_1: bool| Some(!in_1));
            NotLegs::OUT.bind(state, chip, binding);
            binding.set_source_1(state, &mut NotLegs::IN_.value_source(chip));
            chip
        }
    }

    impl ChipLegs for NotLegs { }
}

use circuit::*;
use dep_obj::{Change, DepObjId};
use dep_obj::binding::{Binding1, Bindings};
use dyn_context::{State, StateExt, StateRefMut, Stop};
use not_chip::*;
use or_chip::*;
use std::fmt::Write;

#[derive(Debug, Clone)]
struct TriggerChips {
    pub or_1: Chip,
    pub or_2: Chip,
    pub not_1: Chip,
    pub not_2: Chip,
}

#[derive(State, Stop)]
#[state(part)]
struct TriggerState {
    #[state(part)]
    bindings: Bindings,
    #[state(part)]
    #[stop]
    circuit: Circuit,
    #[state(part)]
    chips: TriggerChips,
    log: String,
}

fn main() {
    let mut circuit = Circuit::new();
    let mut bindings = Bindings::new();
    let chips = (&mut circuit).merge_mut_and_then(|state| {
        let not_1 = NotLegs::new(state);
        let not_2 = NotLegs::new(state);
        let or_1 = OrLegs::new(state);
        let or_2 = OrLegs::new(state);
        TriggerChips { or_1, or_2, not_1, not_2 }
    }, &mut bindings);
    let state = &mut TriggerState {
        circuit,
        bindings,
        chips: chips.clone(),
        log: String::new(),
    };

    let not_1_out_to_or_2_in = Binding1::new(state, (), |(), value| Some(value));
    OrLegs::IN_2.bind(state, chips.or_2, not_1_out_to_or_2_in);
    not_1_out_to_or_2_in.set_source_1(state, &mut NotLegs::OUT.value_source(chips.not_1));
    let not_2_out_to_or_1_in = Binding1::new(state, (), |(), value| Some(value));
    OrLegs::IN_2.bind(state, chips.or_1, not_2_out_to_or_1_in);
    not_2_out_to_or_1_in.set_source_1(state, &mut NotLegs::OUT.value_source(chips.not_2));
    let or_1_out_to_not_1_in = Binding1::new(state, (), |(), value| Some(value));
    NotLegs::IN_.bind(state, chips.not_1, or_1_out_to_not_1_in);
    or_1_out_to_not_1_in.set_source_1(state, &mut OrLegs::OUT.value_source(chips.or_1));
    let or_2_out_to_not_2_in = Binding1::new(state, (), |(), value| Some(value));
    NotLegs::IN_.bind(state, chips.not_2, or_2_out_to_not_2_in);
    or_2_out_to_not_2_in.set_source_1(state, &mut OrLegs::OUT.value_source(chips.or_2));

    let print_out = Binding1::new(state, (), |(), change: Option<Change<bool>>| change);
    print_out.set_target_fn(state, (), |state, (), change| {
        let state: &mut TriggerState = state.get_mut();
        let old = if change.old { "1" } else { "0" };
        let new = if change.new { "1" } else { "0" };
        writeln!(state.log, "{} -> {}", old, new).unwrap();
    });
    chips.not_2.add_binding::<NotLegs, _>(state, print_out);
    print_out.set_source_1(state, &mut NotLegs::OUT.change_source(chips.not_2));
    OrLegs::IN_1.set(state, chips.or_1, true).immediate();
    OrLegs::IN_1.set(state, chips.or_1, false).immediate();
    OrLegs::IN_1.set(state, chips.or_2, true).immediate();
    OrLegs::IN_1.set(state, chips.or_2, false).immediate();
    OrLegs::IN_1.set(state, chips.or_1, true).immediate();
    OrLegs::IN_1.set(state, chips.or_1, false).immediate();
    OrLegs::IN_1.set(state, chips.or_2, true).immediate();
    OrLegs::IN_1.set(state, chips.or_2, false).immediate();

    TriggerState::stop(state);

    print!("{}", state.log);
    assert_eq!(state.log, "\
        1 -> 0\n\
        0 -> 1\n\
        1 -> 0\n\
    ");
}
