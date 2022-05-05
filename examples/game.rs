#![feature(const_ptr_offset_from)]

#![deny(warnings)]

mod game {
    use components_arena::{Arena, Component, NewtypeComponentId, Id};
    use debug_panic::debug_panic;
    use dep_obj::{DetachedDepObjId, dep_obj, dep_type};
    use dyn_context::state::{RequiresStateDrop, SelfState, State, StateExt, StateDrop};
    use macro_attr_2018::macro_attr;
    use std::borrow::Cow;

    pub struct Game(StateDrop<Game_>);

    impl SelfState for Game { }

    struct Game_ {
        items: Arena<ItemData>,
    }

    impl RequiresStateDrop for Game_ {
        fn get(state: &dyn State) -> &StateDrop<Self> {
            &state.get::<Game>().0
        }

        fn get_mut(state: &mut dyn State) -> &mut StateDrop<Self> {
            &mut state.get_mut::<Game>().0
        }

        fn before_drop(state: &mut dyn State) {
            let items = Self::get(state).get().items.items().ids().map(Item).collect::<Vec<_>>();
            for item in items {
                item.drop_bindings_priv(state);
            }
        }

        fn drop_incorrectly(self) {
            debug_panic!("Game should be dropped with the drop_self method");
        }
    }

    impl Game {
        pub fn new() -> Game {
            Game(StateDrop::new(Game_ { items: Arena::new() }))
        }

        pub fn drop_self(state: &mut dyn State) {
            <StateDrop<Game_>>::drop_self(state);
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
        pub struct ItemProps in Item {
            name: Cow<'static, str> = Cow::Borrowed(""),
            base_weight: f32 = 0.0,
            weight: f32 = 0.0,
            equipped: bool = false,
            cursed: bool = false,
        }
    }

    impl Item {
        pub fn new_raw(state: &mut dyn State) -> Item {
            let game: &mut Game = state.get_mut();
            game.0.get_mut().items.insert(|id| (ItemData { props: ItemProps::new_priv() }, Item(id)))
        }

        pub fn drop_self(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let game: &mut Game = state.get_mut();
            game.0.get_mut().items.remove(self.0);
        }

        dep_obj! {
            pub fn props(self as this, game: Game) -> (ItemProps) {
                if mut {
                    &mut game.0.get_mut().items[this.0].props
                } else {
                    &game.0.get().items[this.0].props
                }
            }
        }
    }
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
    (&mut Game::new()).merge_mut_and_then(|state| {
        run(state);
        Game::drop_self(state);
    }, &mut Bindings::new());
}
