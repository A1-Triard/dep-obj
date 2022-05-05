#![feature(const_ptr_offset_from)]

#![deny(warnings)]

mod game {
    use dep_obj::dep_type;
    use dep_obj::templates::detached_static_dep_obj;
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

    pub type Item = detached_static_dep_obj::Id<ItemProps>;
    pub type Game = detached_static_dep_obj::Arena<ItemProps>;
}

use dep_obj::binding::{Binding1, Binding3, Bindings};
use dyn_context::state::{State, StateRefMut};
use game::*;

fn new_item(state: &mut dyn State) -> Item {
    let item = Item::new_raw(state);
    let weight = Binding3::new(state, (), |(), base_weight, cursed, equipped| Some(
        if equipped && cursed { base_weight + 100.0 } else { base_weight }
    ));
    ItemProps::WEIGHT.bind(state, item.props(), weight);
    weight.set_source_1(state, &mut ItemProps::BASE_WEIGHT.value_source(item.props()));
    weight.set_source_2(state, &mut ItemProps::CURSED.value_source(item.props()));
    weight.set_source_3(state, &mut ItemProps::EQUIPPED.value_source(item.props()));
    return item;
}

fn run(state: &mut dyn State) {
    let item = new_item(state);
    let weight = Binding1::new(state, (), |(), weight| Some(weight));
    weight.set_target_fn(state, (), |_state, (), weight| {
        println!("Item weight changed, new weight: {}", weight);
    });
    weight.set_source_1(state, &mut ItemProps::WEIGHT.value_source(item.props()));
    ItemProps::BASE_WEIGHT.set(state, item.props(), 5.0).immediate();
    weight.drop_self(state);
    item.drop_self(state);
}

fn main() {
    (&mut Game::new()).merge_mut_and_then(|state| {
        run(state);
        Game::drop_self(state);
    }, &mut Bindings::new());
}
