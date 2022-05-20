#![feature(const_ptr_offset_from)]
#![feature(const_type_id)]
#![feature(explicit_generic_args_with_impl_trait)]

#![deny(warnings)]

mod items {
    use components_arena::{Arena, Component, ComponentStop, NewtypeComponentId, Id, with_arena_in_state_part};
    use dep_obj::{DepType, DetachedDepObjId, GenericBuilder, dep_type, impl_dep_obj, with_builder};
    use dep_obj::binding::Binding3;
    use downcast_rs::{Downcast, impl_downcast};
    use dyn_context::{SelfState, State, StateExt, Stop};
    use macro_attr_2018::macro_attr;
    use std::borrow::Cow;

    pub trait ItemObj: Downcast + DepType<Id=Item> { }

    impl_downcast!(ItemObj);

    macro_attr! {
        #[derive(Debug, Component!(stop=ItemStop))]
        struct ItemComponent {
            props: ItemProps,
            obj: Box<dyn ItemObj>,
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
        with_builder!(ItemProps<'a>);

        pub fn new(state: &mut dyn State, obj: Box<dyn ItemObj>) -> Item {
            let items: &mut Items = state.get_mut();
            let item = items.0.insert(|id| (ItemComponent {
                props: ItemProps::new_priv(),
                obj
            }, Item(id)));
            item.bind_weight(state);
            item
        }

        pub fn drop_self(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let items: &mut Items = state.get_mut();
            items.0.remove(self.0);
        }

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
        type ItemProps => Items | .props,
        trait ItemObj => Items | .obj,
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
        pub struct ItemProps in Item {
            name: Cow<'static, str> = Cow::Borrowed(""),
            base_weight: f32 = 0.0,
            weight: f32 = 0.0,
            equipped: bool = false,
            cursed: bool = false,
        }

        type BaseBuilder<'a> = GenericBuilder<'a, Item>;
    }
}

mod weapon {
    use dep_obj::{dep_type, ext_builder};
    use dep_obj::binding::Binding3;
    use dyn_context::State;
    use crate::items::*;

    dep_type! {
        #[derive(Debug)]
        pub struct Weapon in Item {
            base_damage: f32 = 0.0,
            damage: f32 = 0.0,
        }
        type BaseBuilder<'a> = ItemPropsBuilder<'a>;
    }

    ext_builder!(<'a> ItemPropsBuilder<'a> as ItemPropsBuilderWeaponExt { Weapon<'b> });

    impl ItemObj for Weapon { }

    impl Weapon {
        #[allow(clippy::new_ret_no_self)]
        pub fn new(state: &mut dyn State) -> Item {
            let item = Item::new(state, Box::new(Self::new_priv()));
            Self::bind_damage(state, item);
            item
        }

        fn bind_damage(state: &mut dyn State, item: Item) {
            let damage = Binding3::new(state, (), |(), base_damage, cursed, equipped| Some(
                if equipped && cursed { base_damage / 2.0 } else { base_damage }
            ));
            Weapon::DAMAGE.bind(state, item, damage);
            damage.set_source_1(state, &mut Weapon::BASE_DAMAGE.value_source(item));
            damage.set_source_2(state, &mut ItemProps::CURSED.value_source(item));
            damage.set_source_3(state, &mut ItemProps::EQUIPPED.value_source(item));
        }
    }
}

mod armor {
    use dep_obj::dep_type;
    use dep_obj::binding::Binding3;
    use dyn_context::State;
    use crate::items::*;

    dep_type! {
        #[derive(Debug)]
        pub struct Armor in Item {
            base_armor_class: f32 = 0.0,
            armor_class: f32 = 0.0,
        }
    }

    impl ItemObj for Armor { }

    impl Armor {
        #[allow(clippy::new_ret_no_self)]
        pub fn new(state: &mut dyn State) -> Item {
            let item = Item::new(state, Box::new(Self::new_priv()));
            Self::bind_armor_class(state, item);
            item
        }

        fn bind_armor_class(state: &mut dyn State, item: Item) {
            let armor_class = Binding3::new(state, (), |(), base_armor_class, cursed, equipped| Some(
                if equipped && cursed { base_armor_class / 2.0 } else { base_armor_class }
            ));
            Armor::ARMOR_CLASS.bind(state, item, armor_class);
            armor_class.set_source_1(state, &mut Armor::BASE_ARMOR_CLASS.value_source(item));
            armor_class.set_source_2(state, &mut ItemProps::CURSED.value_source(item));
            armor_class.set_source_3(state, &mut ItemProps::EQUIPPED.value_source(item));
        }
    }
}

use dep_obj::{Change, Convenient, DepObj, DepObjId, DepProp, DepType};
use dep_obj::binding::{Binding2, Bindings};
use dyn_context::{Stop, State, StateRefMut};
use items::*;
use std::borrow::Cow;
use std::fmt::Display;
use weapon::*;
use armor::*;

fn track_prop<D: DepType<Id=Item> + 'static, T: Convenient + Display>(
    state: &mut dyn State,
    item: Item,
    prop_name: &'static str,
    prop: DepProp<D, T>
) where Item: DepObj<D> {
    let binding = Binding2::new(state, (), |(), name, value: Option<Change<T>>|
        value.map(|value| (name, value.new))
    );
    binding.set_target_fn(state, prop_name, |_state, prop_name, (name, value)| {
        print!("{name} {prop_name} now is {value}.\n\n");
    });
    item.add_binding::<ItemProps, _>(state, binding);
    binding.set_source_1(state, &mut ItemProps::NAME.value_source(item));
    binding.set_source_2(state, &mut prop.change_source(item));
}

fn run(state: &mut dyn State) {
    let sword = Weapon::new(state);
    track_prop(state, sword, "weight", ItemProps::WEIGHT);
    track_prop(state, sword, "damage", Weapon::DAMAGE);
    sword.build(state, |sword| sword
        .name(Cow::Borrowed("Sword"))
        .base_weight(5.0)
        .weapon(|weapon| weapon
            .base_damage(8.0)
        )
    );

    let boots = Armor::new(state);
    track_prop(state, boots, "weight", ItemProps::WEIGHT);
    track_prop(state, boots, "armor_class", Armor::ARMOR_CLASS);
    ItemProps::NAME.set(state, boots, Cow::Borrowed("Boots")).immediate();

    print!("> boots.base_armor_class = 4.0\n\n");
    Armor::BASE_ARMOR_CLASS.set(state, boots, 4.0).immediate();

    print!("> boots.base_weight = 1.5\n\n");
    ItemProps::BASE_WEIGHT.set(state, boots, 1.5).immediate();

    print!("> sword.cursed = true\n\n");
    ItemProps::CURSED.set(state, sword, true).immediate();

    print!("> sword.equipped = true\n\n");
    ItemProps::EQUIPPED.set(state, sword, true).immediate();

    print!("> sword.cursed = false\n\n");
    ItemProps::CURSED.set(state, sword, false).immediate();

    boots.drop_self(state);
    sword.drop_self(state);
}

fn main() {
    (&mut Items::new()).merge_mut_and_then(|state| {
        run(state);
        Items::stop(state);
    }, &mut Bindings::new());
}
