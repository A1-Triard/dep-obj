#![deny(warnings)]
#![allow(dead_code)]

mod circuit {
    use dep_obj::dep_system;
    use components_arena::{RawId, Component, ComponentId, Id, Arena, ComponentClassToken};
    use std::fmt::Debug;
    use educe::Educe;
    use macro_attr_2018::macro_attr;
    use downcast_rs::{Downcast, impl_downcast};

    pub trait ChipLegs: Downcast + Debug + Send + Sync { }

    impl_downcast!(ChipLegs);

    macro_attr! {
        #[derive(Component!)]
        #[derive(Debug)]
        struct ChipNode {
            chip: Chip,
            legs: Box<dyn ChipLegs>,
            tag: RawId,
        }
    }

    macro_attr! {
        #[derive(ComponentId!)]
        #[derive(Educe)]
        #[educe(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        pub struct Chip(Id<ChipNode>);
    }

    impl Chip {
        pub fn new<Tag: ComponentId, T>(
            circuit: &mut Circuit,
            legs_tag: impl FnOnce(Chip) -> (Box<dyn ChipLegs>, Tag, T)
        ) -> T {
            circuit.arena.insert(|chip| {
                let (legs, tag, result) = legs_tag(Chip(chip));
                (ChipNode { chip: Chip(chip), legs, tag: tag.into_raw() }, result)
            })
        }

        pub fn tag<Tag: ComponentId>(self, circuit: &Circuit) -> Tag {
            Tag::from_raw(circuit.arena[self.0].tag)
        }

        pub fn drop(self, circuit: &mut Circuit) {
            circuit.arena.remove(self.0);
        }

        dep_system! {
            pub dyn fn legs(self as this, circuit: Circuit) -> ChipLegs {
                if mut { &mut circuit.arena[this.0].legs } else { &circuit.arena[this.0].legs }
            }
        }
    }

    #[derive(Debug)]
    pub struct Circuit {
        arena: Arena<ChipNode>,
    }

    impl Circuit {
        pub fn new(token: &mut CircuitToken) -> Self {
            Circuit {
                arena: Arena::new(&mut token.0)
            }
        }
    }

    pub struct CircuitToken(ComponentClassToken<ChipNode>);

    impl CircuitToken {
        pub fn new() -> Option<Self> {
            ComponentClassToken::new().map(CircuitToken)
        }
    }
}

mod or_chip {
    use crate::circuit::*;
    use components_arena::ComponentId;
    use dep_obj::{dep_obj, DepTypeToken};
    use dyn_context::{Context, ContextExt};

    dep_obj! {
        #[derive(Debug)]
        pub struct OrLegs become legs in Chip {
            in_1: bool = false,
            in_2: bool = false,
            out: bool = false,
        }
    }

    impl OrLegsType {
        pub fn new() -> Option<DepTypeToken<Self>> { Self::new_priv() }
    }

    impl OrLegs {
        pub fn new<Tag: ComponentId, T>(
            circuit: &mut Circuit,
            token: &DepTypeToken<OrLegsType>,
            tag: impl FnOnce(Chip) -> (Tag, T)
        ) -> T {
            let legs = Self::new_priv(token);
            let (chip, result) = Chip::new(circuit, |chip| {
                let (tag, result) = tag(chip);
                (Box::new(legs) as _, tag, (chip, result))
            });
            chip.legs_on_changed(circuit, token.ty().in_1(), Self::update);
            chip.legs_on_changed(circuit, token.ty().in_2(), Self::update);
            result
        }

        fn update(context: &mut dyn Context, chip: Chip, _old: &bool) {
            let token: &DepTypeToken<OrLegsType> = context.get();
            let circuit: &Circuit= context.get();
            let in_1 = *chip.legs_get(circuit, token.ty().in_1());
            let in_2 = *chip.legs_get(circuit, token.ty().in_2());
            let out = token.ty().out();
            chip.legs_set_distinct(context, out, in_1 | in_2);
        }
    }

    impl ChipLegs for OrLegs { }
}

mod not_chip {
    use crate::circuit::*;
    use components_arena::ComponentId;
    use dep_obj::{dep_obj, DepTypeToken};
    use dyn_context::{Context, ContextExt};

    dep_obj! {
        #[derive(Debug)]
        pub struct NotLegs become legs in Chip {
            in_: bool = false,
            out: bool = true,
        }
    }

    impl NotLegsType {
        pub fn new() -> Option<DepTypeToken<Self>> { Self::new_priv() }
    }

    impl NotLegs {
        pub fn new<Tag: ComponentId, T>(
            circuit: &mut Circuit,
            token: &DepTypeToken<NotLegsType>,
            tag: impl FnOnce(Chip) -> (Tag, T)
        ) -> T {
            let legs = Self::new_priv(token);
            let (chip, result) = Chip::new(circuit, |chip| {
                let (tag, result) = tag(chip);
                (Box::new(legs) as _, tag, (chip, result))
            });
            chip.legs_on_changed(circuit, token.ty().in_(), Self::update);
            result
        }

        fn update(context: &mut dyn Context, chip: Chip, _old: &bool) {
            let token: &DepTypeToken<NotLegsType> = context.get();
            let circuit: &Circuit = context.get();
            let in_ = *chip.legs_get(circuit, token.ty().in_());
            let out = token.ty().out();
            chip.legs_set_distinct(context, out, !in_);
        }
    }

    impl ChipLegs for NotLegs { }
}

use dep_obj::{DepTypeToken};
use dyn_context::{Context, ContextExt, context};
use circuit::*;
use or_chip::*;
use not_chip::*;

struct TriggerChips {
    pub or_1: Chip,
    pub or_2: Chip,
    pub not_1: Chip,
    pub not_2: Chip,
}

context! {
    dyn struct TriggerContext {
        circuit: mut Circuit,
        or_legs: ref DepTypeToken<OrLegsType>,
        not_legs: ref DepTypeToken<NotLegsType>,
        chips: ref TriggerChips,
    }
}

fn main() {
    let mut circuit_token = CircuitToken::new().unwrap();
    let circuit = &mut Circuit::new(&mut circuit_token);
    let or_legs: DepTypeToken<OrLegsType> =  OrLegsType::new().unwrap();
    let not_legs: DepTypeToken<NotLegsType> = NotLegsType::new().unwrap();
    let not_1 = NotLegs::new(circuit, &not_legs, |chip| (1usize, chip));
    let not_2 = NotLegs::new(circuit, &not_legs, |chip| (2usize, chip));
    let or_1 = OrLegs::new(circuit, &or_legs, |chip| (1usize, chip));
    let or_2 = OrLegs::new(circuit, &or_legs, |chip| (2usize, chip));
    let on_not_out_changed = |context: &mut dyn Context, not: Chip, _old: &_| {
        let not_legs: &DepTypeToken<NotLegsType> = context.get();
        let or_legs: &DepTypeToken<OrLegsType> = context.get();
        let circuit: &Circuit = context.get();
        let chips: &TriggerChips = context.get();
        let or = if not.tag::<usize>(circuit) == 1 { chips.or_2 } else { chips.or_1 };
        let &out = not.legs_get(circuit, not_legs.ty().out());
        let in_2 = or_legs.ty().in_2();
        or.legs_set_uncond(context, in_2, out);
    };
    let on_or_out_changed = |context: &mut dyn Context, or: Chip, _old: &_| {
        let not_legs: &DepTypeToken<NotLegsType> = context.get();
        let or_legs: &DepTypeToken<OrLegsType> = context.get();
        let circuit: &Circuit = context.get();
        let chips: &TriggerChips = context.get();
        let not = if or.tag::<usize>(circuit) == 1 { chips.not_1 } else { chips.not_2 };
        let &out = or.legs_get(circuit, or_legs.ty().out());
        let in_ = not_legs.ty().in_();
        not.legs_set_uncond(context, in_, out);
    };
    not_1.legs_on_changed(circuit, not_legs.ty().out(), on_not_out_changed);
    not_2.legs_on_changed(circuit, not_legs.ty().out(), on_not_out_changed);
    or_1.legs_on_changed(circuit, or_legs.ty().out(), on_or_out_changed);
    or_2.legs_on_changed(circuit, or_legs.ty().out(), on_or_out_changed);
    not_1.legs_on_changed(circuit, not_legs.ty().out(), |context, not_1, _old| {
        let not_legs: &DepTypeToken<NotLegsType> = context.get();
        let circuit: &Circuit = context.get();
        let &out = not_1.legs_get(circuit, not_legs.ty().out());
        println!("{}", if out { "0 -> 1" } else { "1 -> 0" });
    });
    let chips = TriggerChips { or_1, or_2, not_1, not_2 };
    TriggerContext::call(circuit, &or_legs, &not_legs, &chips, |context| {
        or_1.legs_set_distinct(context, or_legs.ty().in_1(), true);
        or_1.legs_set_distinct(context, or_legs.ty().in_1(), false);
        or_2.legs_set_distinct(context, or_legs.ty().in_1(), true);
        or_2.legs_set_distinct(context, or_legs.ty().in_1(), false);
        or_1.legs_set_distinct(context, or_legs.ty().in_1(), true);
        or_1.legs_set_distinct(context, or_legs.ty().in_1(), false);
        or_2.legs_set_distinct(context, or_legs.ty().in_1(), true);
        or_2.legs_set_distinct(context, or_legs.ty().in_1(), false);
    });
}
