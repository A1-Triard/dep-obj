#![feature(const_ptr_offset_from)]
#![feature(const_type_id)]

#![deny(warnings)]
#![allow(dead_code)]

mod circuit {
    use dep_obj::templates::detached_dynamic_dep_type;

    pub enum ChipLegs { }
    pub type ChipLegsKey = detached_dynamic_dep_type::ObjKey;
    pub type Chip = detached_dynamic_dep_type::Id<ChipLegs>;
    pub type Circuit = detached_dynamic_dep_type::Arena<ChipLegs>;
    pub use detached_dynamic_dep_type::Obj as ChipObj;
}

mod or_chip {
    use crate::circuit::*;
    use dep_obj::{dep_type};
    use dep_obj::binding::Binding2;
    use dyn_context::state::State;

    dep_type! {
        #[derive(Debug)]
        pub struct OrLegs in Chip as ChipLegsKey {
            in_1: bool = false,
            in_2: bool = false,
            out: bool = false,
        }
    }

    impl OrLegs {
        pub fn new(state: &mut dyn State) -> Chip {
            Chip::new(state, |chip| (Box::new(Self::new_priv()) as _, chip), |state, chip| {
                let binding = Binding2::new(state, (), |(), in_1, in_2| Some(in_1 | in_2));
                OrLegs::OUT.bind(state, chip, binding);
                binding.set_source_1(state, &mut OrLegs::IN_1.value_source(chip));
                binding.set_source_2(state, &mut OrLegs::IN_2.value_source(chip));
            })
        }
    }

    impl ChipObj<ChipLegs> for OrLegs { }
}

mod not_chip {
    use crate::circuit::*;
    use dep_obj::dep_type;
    use dep_obj::binding::Binding1;
    use dyn_context::state::State;

    dep_type! {
        #[derive(Debug)]
        pub struct NotLegs in Chip as ChipLegsKey {
            in_: bool = false,
            out: bool = true,
        }
    }

    impl NotLegs {
        pub fn new(state: &mut dyn State) -> Chip {
            Chip::new(state, |chip| (Box::new(Self::new_priv()) as _, chip), |state, chip| {
                let binding = Binding1::new(state, (), |(), in_1: bool| Some(!in_1));
                NotLegs::OUT.bind(state, chip, binding);
                binding.set_source_1(state, &mut NotLegs::IN_.value_source(chip));
            })
        }
    }

    impl ChipObj<ChipLegs> for NotLegs { }
}

use circuit::*;
use dep_obj::binding::{Binding1, Bindings};
use dyn_context::state::{State, StateExt, StateRefMut};
use not_chip::*;
use or_chip::*;
use std::any::{Any, TypeId};
use std::fmt::Write;
use dep_obj::Change;

#[derive(Debug, Clone)]
struct TriggerChips {
    pub or_1: Chip,
    pub or_2: Chip,
    pub not_1: Chip,
    pub not_2: Chip,
}

struct TriggerState {
    bindings: Bindings,
    circuit: Circuit,
    chips: TriggerChips,
    log: String,
}

impl State for TriggerState {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if ty == TypeId::of::<TriggerState>() {
            Some(self)
        } else if ty == TypeId::of::<Bindings>() {
            Some(&self.bindings)
        } else if ty == TypeId::of::<Circuit>() {
            Some(&self.circuit)
        } else if ty == TypeId::of::<TriggerChips>() {
            Some(&self.chips)
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if ty == TypeId::of::<TriggerState>() {
            Some(self)
        } else if ty == TypeId::of::<Bindings>() {
            Some(&mut self.bindings)
        } else if ty == TypeId::of::<Circuit>() {
            Some(&mut self.circuit)
        } else {
            None
        }
    }
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
    print_out.set_source_1(state, &mut NotLegs::OUT.change_source(chips.not_2));
    OrLegs::IN_1.set(state, chips.or_1, true).immediate();
    OrLegs::IN_1.set(state, chips.or_1, false).immediate();
    OrLegs::IN_1.set(state, chips.or_2, true).immediate();
    OrLegs::IN_1.set(state, chips.or_2, false).immediate();
    OrLegs::IN_1.set(state, chips.or_1, true).immediate();
    OrLegs::IN_1.set(state, chips.or_1, false).immediate();
    OrLegs::IN_1.set(state, chips.or_2, true).immediate();
    OrLegs::IN_1.set(state, chips.or_2, false).immediate();

    print_out.drop_self(state);
    chips.or_1.drop_self(state);
    chips.or_2.drop_self(state);
    chips.not_2.drop_self(state);
    chips.not_1.drop_self(state);

    print!("{}", state.log);
    assert_eq!(state.log, "\
        1 -> 0\n\
        0 -> 1\n\
        1 -> 0\n\
    ");
    Circuit::drop_self(state);
}
