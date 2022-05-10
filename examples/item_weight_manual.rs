#![feature(const_ptr_offset_from)]
#![feature(const_type_id)]

#![deny(warnings)]

mod items {
    use components_arena::{Arena, Component, NewtypeComponentId, Id};
    use debug_panic::debug_panic;
    use dep_obj::{DetachedDepObjId, dep_obj, dep_type};
    use dyn_context::state::{RequiresStateDrop, SelfState, State, StateExt, StateDrop};
    use macro_attr_2018::macro_attr;
    use std::borrow::Cow;

    pub struct Items(StateDrop<Items_>);

    impl SelfState for Items { }

    struct Items_ {
        items: Arena<ItemData>,
    }

    impl RequiresStateDrop for Items_ {
        fn get(state: &dyn State) -> &StateDrop<Self> {
            &state.get::<Items>().0
        }

        fn get_mut(state: &mut dyn State) -> &mut StateDrop<Self> {
            &mut state.get_mut::<Items>().0
        }

        fn before_drop(state: &mut dyn State) {
            let items = Self::get(state).get().items.items().ids().map(Item).collect::<Vec<_>>();
            for item in items {
                item.drop_bindings_priv(state);
            }
        }

        fn drop_incorrectly(self) {
            debug_panic!("Items should be dropped with the drop_self method");
        }
    }

    impl Items {
        pub fn new() -> Items {
            Items(StateDrop::new(Items_ { items: Arena::new() }))
        }

        pub fn drop_self(state: &mut dyn State) {
            <StateDrop<Items_>>::drop_self(state);
        }
    }

    macro_attr! {
        #[derive(Debug, Component!)]
        struct ItemData {
            props: ItemProps,
        }
    }

    macro_attr! {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
        pub struct Item(Id<ItemData>);
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
            let item = items.0.get_mut().items.insert(|id| (ItemData { props: ItemProps::new_priv() }, Item(id)));
            init(state, item);
            item
        }

        pub fn drop_self(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let items: &mut Items = state.get_mut();
            items.0.get_mut().items.remove(self.0);
        }
    }

    dep_obj! {
        impl Item {
            ItemProps => fn(self as this, items: Items) -> (ItemProps) {
                if mut {
                    &mut items.0.get_mut().items[this.0].props
                } else {
                    &items.0.get().items[this.0].props
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