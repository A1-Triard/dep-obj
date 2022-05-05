#![feature(const_ptr_offset_from)]

#![deny(warnings)]

mod objs {
    use dep_obj::dep_type;
    use dep_obj::templates::detached_static_dep_type;
    use std::borrow::Cow;

    dep_type! {
        #[derive(Debug)]
        pub struct ItemProps in Item {
            name: Cow<'static, str> = Cow::Borrowed(""),
            base_weight: f32 = 0.0,
            weight: f32 = 0.0,
            equipped: bool = false,
            cursed: bool = false,
        }
    }

    pub type Item = detached_static_dep_type::Id<ItemProps>;
    pub type Objs = detached_static_dep_type::Arena<ItemProps>;
}

mod behavior {
    use dep_obj::binding::Binding3;
    use dyn_context::state::State;
    use crate::objs::*;

    pub fn new_item(state: &mut dyn State) -> Item {
        let item = Item::new(state);
        let weight = Binding3::new(state, (), |(), base_weight, cursed, equipped| Some(
            if equipped && cursed { base_weight + 100.0 } else { base_weight }
        ));
        ItemProps::WEIGHT.bind(state, item.props(), weight);
        weight.set_source_1(state, &mut ItemProps::BASE_WEIGHT.value_source(item.props()));
        weight.set_source_2(state, &mut ItemProps::CURSED.value_source(item.props()));
        weight.set_source_3(state, &mut ItemProps::EQUIPPED.value_source(item.props()));
        return item;
    }
}

use dep_obj::binding::{Binding1, Bindings};
use dyn_context::state::{State, StateRefMut};
use objs::*;
use behavior::*;

fn run(state: &mut dyn State) {
    let item = new_item(state);

    let weight = Binding1::new(state, (), |(), weight| Some(weight));
    weight.set_target_fn(state, (), |_state, (), weight| {
        println!("Item weight changed, new weight: {}", weight);
    });
    weight.set_source_1(state, &mut ItemProps::WEIGHT.value_source(item.props()));

    println!("> item.base_weight = 5.0");
    ItemProps::BASE_WEIGHT.set(state, item.props(), 5.0).immediate();

    println!("> item.cursed = true");
    ItemProps::CURSED.set(state, item.props(), true).immediate();

    println!("> item.equipped = true");
    ItemProps::EQUIPPED.set(state, item.props(), true).immediate();

    println!("> item.cursed = false");
    ItemProps::CURSED.set(state, item.props(), false).immediate();

    weight.drop_self(state);
    item.drop_self(state);
}

fn main() {
    (&mut Objs::new()).merge_mut_and_then(|state| {
        run(state);
        Objs::drop_self(state);
    }, &mut Bindings::new());
}
