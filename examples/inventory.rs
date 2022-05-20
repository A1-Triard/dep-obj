#![feature(const_ptr_offset_from)]
#![feature(const_type_id)]
#![feature(explicit_generic_args_with_impl_trait)]

#![deny(warnings)]
#![allow(dead_code)]

use components_arena::{Arena, Component, ComponentStop, NewtypeComponentId, Id, with_arena_in_state_part};
use dep_obj::{Change, DepObjId, DepVecItemPos, DetachedDepObjId, GenericBuilder, ItemChange};
use dep_obj::{dep_type, impl_dep_obj, with_builder};
use macro_attr_2018::macro_attr;
use dep_obj::binding::{Binding1, Binding2, BindingExt2, Bindings, Re};
use dyn_context::{State, StateExt, Stop};
use std::borrow::Cow;
use std::fmt::Write;

macro_attr! {
    #[derive(Debug, Component!(stop=ItemStop))]
    struct ItemComponent {
        props: ItemProps,
    }
}

impl ComponentStop for ItemStop {
    with_arena_in_state_part!(Game { .items });

    fn stop(&self, state: &mut dyn State, id: Id<ItemComponent>) {
        Item(id).drop_bindings_priv(state);
    }
}

macro_attr! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
    struct Item(Id<ItemComponent>);
}

impl DetachedDepObjId for Item { }

dep_type! {
    #[derive(Debug)]
    struct ItemProps in Item {
        name: Cow<'static, str> = Cow::Borrowed(""),
        equipped: bool = false,
        enhancement: i8 = 0,
    }
}

impl Item {
    fn new(state: &mut dyn State) -> Item {
        let game: &mut Game = state.get_mut();
        game.items.insert(|id| (ItemComponent { props: ItemProps::new_priv() }, Item(id)))
    }

    fn drop_self(self, state: &mut dyn State) {
        self.drop_bindings_priv(state);
        let game: &mut Game = state.get_mut();
        game.items.remove(self.0);
    }

    with_builder!(ItemProps<'a>);
}

impl_dep_obj!(Item {
    type ItemProps => Game { .items } | .props
});

macro_attr! {
    #[derive(Debug, Component!(stop=NpcStop))]
    struct NpcComponent {
        props: NpcProps,
    }
}

impl ComponentStop for NpcStop {
    with_arena_in_state_part!(Game { .npcs });

    fn stop(&self, state: &mut dyn State, id: Id<NpcComponent>) {
        Npc(id).drop_bindings_priv(state);
    }
}

macro_attr! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
    struct Npc(Id<NpcComponent>);
}

impl DetachedDepObjId for Npc { }

dep_type! {
    #[derive(Debug)]
    struct NpcProps in Npc {
        equipped_items [Item],
        items_enhancement: i8 = 0,
    }
}

impl Npc {
    fn new(state: &mut dyn State) -> Npc {
        let game: &mut Game = state.get_mut();
        let npc = game.npcs.insert(|id| (NpcComponent { props: NpcProps::new_priv() }, Npc(id)));

        let equipped = Binding1::new(state, (), |(), change: Option<ItemChange<Item>>| change);
        equipped.dispatch(state, (), |state, (), change|
            ItemProps::EQUIPPED.set(state, change.item, change.is_insert())
        );
        npc.add_binding::<NpcProps, _>(state, equipped);
        equipped.set_source_1(state, &mut NpcProps::EQUIPPED_ITEMS.item_source(npc));

        let enhancement = BindingExt2::new(state, (), |state, _, enhancement, change: Option<ItemChange<Item>>| {
            if let Some(change) = change {
                if change.is_remove() {
                    ItemProps::ENHANCEMENT.unset(state, change.item)
                } else if change.is_insert() || change.is_update_insert() {
                    ItemProps::ENHANCEMENT.set(state, change.item, enhancement)
                } else {
                    Re::Continue
                }
            } else {
                Re::Yield(())
            }
        });
        enhancement.set_source_2(state, &mut NpcProps::EQUIPPED_ITEMS.item_source_with_update(enhancement, npc));
        enhancement.set_source_1(state, &mut NpcProps::ITEMS_ENHANCEMENT.value_source(npc));

        npc
    }

    fn drop_self(self, state: &mut dyn State) {
        self.drop_bindings_priv(state);
        let game: &mut Game = state.get_mut();
        game.npcs.remove(self.0);
    }
}

impl_dep_obj!(Npc {
    type NpcProps => Game { .npcs } | .props
});

#[derive(State, Stop)]
#[state(part)]
struct Game {
    #[stop]
    items: Arena<ItemComponent>,
    #[stop]
    npcs: Arena<NpcComponent>,
    #[state(part)]
    bindings: Bindings,
    log: String,
}

impl Game {
    fn new() -> Game {
        Game {
            items: Arena::new(),
            npcs: Arena::new(),
            bindings: Bindings::new(),
            log: String::new(),
        }
    }
}

fn main() {
    let game = &mut Game::new();
    let npc = Npc::new(game);
    NpcProps::ITEMS_ENHANCEMENT.set(game, npc, 5).immediate();
    let sword = Item::new(game);
    sword.build(game, |sword| sword
        .name(Cow::Borrowed("Sword"))
    );
    let shield = Item::new(game);
    ItemProps::NAME.set(game, shield, Cow::Borrowed("Shield")).immediate();
    for item in [sword, shield] {
        let equipped = Binding2::new(game, (), |(), name, equipped: Option<Change<bool>>|
            equipped.map(|equipped| (name, equipped.new))
        );
        equipped.set_target_fn(game, (), |game, (), (name, equipped)| {
            let game: &mut Game = game.get_mut();
            writeln!(&mut game.log, "{} {}.", name, if equipped { "equipped" } else { "unequipped" }).unwrap();
        });
        item.add_binding::<ItemProps, _>(game, equipped);
        equipped.set_source_1(game, &mut ItemProps::NAME.value_source(item));
        equipped.set_source_2(game, &mut ItemProps::EQUIPPED.change_source(item));
        let enhancement = Binding2::new(game, (), |(), name, enhancement: Option<Change<i8>>|
            enhancement.map(|enhancement| (name, enhancement))
        );
        enhancement.set_target_fn(game, (), |game, (), (name, enhancement)| {
            let game: &mut Game = game.get_mut();
            writeln!(&mut game.log, "{} enhancement changed: {} -> {}.", name, enhancement.old, enhancement.new).unwrap();
        });
        item.add_binding::<ItemProps, _>(game, enhancement);
        enhancement.set_source_1(game, &mut ItemProps::NAME.value_source(item));
        enhancement.set_source_2(game, &mut ItemProps::ENHANCEMENT.change_source(item));
    }
    NpcProps::EQUIPPED_ITEMS.push(game, npc, sword).immediate();
    NpcProps::EQUIPPED_ITEMS.push(game, npc, shield).immediate();
    NpcProps::ITEMS_ENHANCEMENT.set(game, npc, 4).immediate();
    NpcProps::EQUIPPED_ITEMS.remove(game, npc, DepVecItemPos::FirstItem).immediate();
    NpcProps::ITEMS_ENHANCEMENT.set(game, npc, 5).immediate();

    Game::stop(game);

    print!("{}", game.log);
    assert_eq!(game.log, "\
        Sword equipped.\n\
        Sword enhancement changed: 0 -> 5.\n\
        Shield equipped.\n\
        Shield enhancement changed: 0 -> 5.\n\
        Sword enhancement changed: 5 -> 4.\n\
        Shield enhancement changed: 5 -> 4.\n\
        Sword unequipped.\n\
        Sword enhancement changed: 4 -> 0.\n\
        Shield enhancement changed: 4 -> 5.\n\
    ");
}
