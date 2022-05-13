#![feature(const_ptr_offset_from)]
#![feature(const_type_id)]

#![deny(warnings)]

mod items {
    use components_arena::{Arena, Component, ComponentStop, NewtypeComponentId, Id, arena_newtype};
    use dep_obj::{DetachedDepObjId, dep_obj, dep_type};
    use dyn_context::NewtypeStop;
    use dyn_context::state::{SelfState, State, StateExt};
    use macro_attr_2018::macro_attr;
    use std::borrow::Cow;

    macro_attr! {
        #[derive(NewtypeStop!)]
        pub struct Items(Arena<ItemComponent>);
    }

    impl SelfState for Items { }

    impl Items {
        pub fn new() -> Items {
            Items(Arena::new())
        }
    }

    macro_attr! {
        #[derive(Debug, Component!(stop=ItemStop))]
        struct ItemComponent(ItemProps);
    }

    impl ComponentStop for ItemStop {
        arena_newtype!(Items);

        fn stop(&self, state: &mut dyn State, id: Id<ItemComponent>) {
            Item(id).drop_bindings_priv(state);
        }
    }

    macro_attr! {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
        pub struct Item(Id<ItemComponent>);
    }

    impl DetachedDepObjId for Item { }

    dep_type! {
        #[derive(Debug)]
        pub struct ItemProps in Item as ItemProps {
            name: Cow<'static, str> = Cow::Borrowed(""),
            base_weight: f32 = 0.0,
            weight: f32 = 0.0,
            equipped: bool = false,
            cursed: bool = false,
        }
    }

    impl Item {
        pub fn new(state: &mut dyn State, init: impl FnOnce(&mut dyn State, Item)) -> Item {
            let items: &mut Items = state.get_mut();
            let item = items.0.insert(|id| (ItemComponent(ItemProps::new_priv()), Item(id)));
            init(state, item);
            item
        }

        pub fn drop_self(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let items: &mut Items = state.get_mut();
            items.0.remove(self.0);
        }
    }

    dep_obj! {
        impl Item {
            ItemProps => fn(self as this, items: Items) -> (ItemProps) {
                if mut {
                    &mut items.0[this.0].0
                } else {
                    &items.0[this.0].0
                }
            }
        }
    }
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
use dyn_context::state::{Stop, State, StateRefMut};
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
        Items::stop(state);
    }, &mut Bindings::new());
}
