#![feature(const_ptr_offset_from)]
#![feature(const_type_id)]
#![feature(explicit_generic_args_with_impl_trait)]

#![deny(warnings)]

mod items {
    use components_arena::{Arena, Component, ComponentStop, NewtypeComponentId, Id, with_arena_in_state_part};
    use dep_obj::{DetachedDepObjId, dep_type, impl_dep_obj, with_builder};
    use dep_obj::binding::Binding3;
    use dyn_context::{SelfState, State, StateExt, Stop};
    use macro_attr_2018::macro_attr;
    use std::borrow::Cow;

    macro_attr! {
        #[derive(Debug, Component!(stop=ItemStop))]
        struct ItemComponent {
            props: ItemProps,
        }
    }

    impl ComponentStop for ItemStop {
        with_arena_in_state_part!(Items);

        fn stop(&self, state: &mut dyn State, id: Id<ItemComponent>) {
            Item(id).drop_bindings_priv(state);
        }
    }

    macro_attr! {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
        pub struct Item(Id<ItemComponent>);
    }

    impl DetachedDepObjId for Item { }

    impl Item {
        pub fn new(state: &mut dyn State) -> Item {
            let items: &mut Items = state.get_mut();
            let item = items.0.insert(|id| (ItemComponent { props: ItemProps::new_priv() }, Item(id)));
            item.bind_weight(state);
            item
        }

        pub fn drop_self(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let items: &mut Items = state.get_mut();
            items.0.remove(self.0);
        }

        with_builder!(ItemProps);

        fn bind_weight(self, state: &mut dyn State) {
            let weight = Binding3::new(state, (), |(), base_weight, cursed, equipped| Some(
                if equipped && cursed { base_weight + 100.0 } else { base_weight }
            ));
            ItemProps::WEIGHT.bind(state, self, weight);
            weight.set_source_1(state, &mut ItemProps::BASE_WEIGHT.value_source(self));
            weight.set_source_2(state, &mut ItemProps::CURSED.value_source(self));
            weight.set_source_3(state, &mut ItemProps::EQUIPPED.value_source(self));
        }
    }

    impl_dep_obj!(Item {
        fn<ItemProps>() -> (ItemProps) { Items | .props }
    });

    #[derive(Debug, Stop)]
    pub struct Items(Arena<ItemComponent>);

    impl SelfState for Items { }

    impl Items {
        pub fn new() -> Items {
            Items(Arena::new())
        }
    }

    dep_type! {
        #[derive(Debug)]
        pub struct ItemProps = Item[ItemProps] {
            name: Cow<'static, str> = Cow::Borrowed(""),
            base_weight: f32 = 0.0,
            weight: f32 = 0.0,
            equipped: bool = false,
            cursed: bool = false,
        }
    }
}

use dep_obj::{Change, DepObjId};
use dep_obj::binding::{Binding2, Bindings};
use dyn_context::{Stop, State, StateRefMut};
use items::*;
use std::borrow::Cow;

fn track_weight(state: &mut dyn State, item: Item) {
    let weight = Binding2::new(state, (), |(), name, weight: Option<Change<f32>>|
        weight.map(|weight| (name, weight.new))
    );
    weight.set_target_fn(state, (), |_state, (), (name, weight)| {
        println!("\n{name} now weights {weight}.");
    });
    item.add_binding::<ItemProps, _>(state, weight);
    weight.set_source_1(state, &mut ItemProps::NAME.value_source(item));
    weight.set_source_2(state, &mut ItemProps::WEIGHT.change_source(item));
}

fn run(state: &mut dyn State) {
    let the_item = Item::new(state);
    track_weight(state, the_item);
    the_item.build(state, |props| props
        .name(Cow::Borrowed("The Item"))
        .base_weight(5.0)
        .cursed(true)
    );

    println!("\n> the_item.equipped = true");
    ItemProps::EQUIPPED.set(state, the_item, true).immediate();

    println!("\n> the_item.cursed = false");
    ItemProps::CURSED.set(state, the_item, false).immediate();

    the_item.drop_self(state);
}

fn main() {
    (&mut Items::new()).merge_mut_and_then(|state| {
        run(state);
        Items::stop(state);
    }, &mut Bindings::new());
}
