#![feature(const_ptr_offset_from)]

#![deny(warnings)]

mod items {
    use dep_obj::dep_type;
    use dep_obj::templates::detached_static_dep_type;
    use std::borrow::Cow;

    dep_type! {
        #[derive(Debug)]
        pub struct ItemProps in Item as detached_static_dep_type::Obj {
            name: Cow<'static, str> = Cow::Borrowed(""),
            base_weight: f32 = 0.0,
            weight: f32 = 0.0,
            equipped: bool = false,
            cursed: bool = false,
        }
    }

    pub type Item = detached_static_dep_type::Id<ItemProps>;
    pub type Items = detached_static_dep_type::Arena<ItemProps>;
}

mod behavior {
    use dep_obj::binding::Binding3;
    use dyn_context::state::State;
    use crate::items::*;

    pub fn item(state: &mut dyn State, item: Item) {
        let weight = Binding3::new(state, (), |(), base_weight, cursed, equipped| Some(
            if equipped && cursed { base_weight + 100.0 } else { base_weight }
        ));
        ItemProps::WEIGHT.bind(state, item, weight);
        weight.set_source_1(state, &mut ItemProps::BASE_WEIGHT.value_source(item));
        weight.set_source_2(state, &mut ItemProps::CURSED.value_source(item));
        weight.set_source_3(state, &mut ItemProps::EQUIPPED.value_source(item));
    }
}

use dep_obj::binding::{Binding1, Bindings};
use dyn_context::state::{State, StateRefMut};
use items::*;

fn run(state: &mut dyn State) {
    let item = Item::new(state, behavior::item);

    let weight = Binding1::new(state, (), |(), weight| Some(weight));
    weight.set_target_fn(state, (), |_state, (), weight| {
        println!("Item weight changed, new weight: {}", weight);
    });
    weight.set_source_1(state, &mut ItemProps::WEIGHT.value_source(item));

    println!("> item.base_weight = 5.0");
    ItemProps::BASE_WEIGHT.set(state, item, 5.0).immediate();

    println!("> item.cursed = true");
    ItemProps::CURSED.set(state, item, true).immediate();

    println!("> item.equipped = true");
    ItemProps::EQUIPPED.set(state, item, true).immediate();

    println!("> item.cursed = false");
    ItemProps::CURSED.set(state, item, false).immediate();

    weight.drop_self(state);
    item.drop_self(state);
}

fn main() {
    (&mut Items::new()).merge_mut_and_then(|state| {
        run(state);
        Items::drop_self(state);
    }, &mut Bindings::new());
}
