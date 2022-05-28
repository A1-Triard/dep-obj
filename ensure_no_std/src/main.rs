#![feature(allocator_api)]
#![feature(const_ptr_offset_from)]
#![feature(const_type_id)]
#![feature(default_alloc_error_handler)]
#![feature(explicit_generic_args_with_impl_trait)]
#![feature(start)]

#![deny(warnings)]

#![no_std]

extern crate alloc;

use core::alloc::Layout;
use core::panic::PanicInfo;
#[cfg(not(windows))]
use libc::exit;
use libc_alloc::LibcAlloc;
#[cfg(windows)]
use winapi::shared::minwindef::UINT;
#[cfg(windows)]
use winapi::um::processthreadsapi::ExitProcess;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;

#[cfg(windows)]
unsafe fn exit(code: UINT) -> ! {
    ExitProcess(code);
    loop { }
}

#[panic_handler]
pub extern fn panic(_info: &PanicInfo) -> ! {
    unsafe { exit(99) }
}

#[no_mangle]
pub fn rust_oom(_layout: Layout) -> ! {
    unsafe { exit(98) }
}

mod items {
    use alloc::borrow::Cow;
    use components_arena::{Arena, Component, ComponentStop, NewtypeComponentId, Id, with_arena_in_state_part};
    use dep_obj::{DetachedDepObjId, dep_type, impl_dep_obj};
    use dyn_context::{SelfState, State, StateExt, Stop};
    use macro_attr_2018::macro_attr;

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
        pub fn new(state: &mut dyn State, init: impl FnOnce(&mut dyn State, Item)) -> Item {
            let items: &mut Items = state.get_mut();
            let item = items.0.insert(|id| (ItemComponent { props: ItemProps::new_priv() }, Item(id)));
            init(state, item);
            item
        }

        #[allow(dead_code)]
        pub fn drop_self(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let items: &mut Items = state.get_mut();
            items.0.remove(self.0);
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

mod behavior {
    use dep_obj::binding::Binding3;
    use dyn_context::State;
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

use dep_obj::binding::Bindings;
use dyn_context::{Stop, State, StateRefMut};
use items::*;

fn run(state: &mut dyn State) {
    let item = Item::new(state, behavior::item);
    ItemProps::BASE_WEIGHT.set(state, item, 5.0).immediate();
    ItemProps::CURSED.set(state, item, true).immediate();
    ItemProps::EQUIPPED.set(state, item, true).immediate();
    ItemProps::CURSED.set(state, item, false).immediate();
}

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    (&mut Items::new()).merge_mut_and_then(|state| {
        run(state);
        Items::stop(state);
    }, &mut Bindings::new());
    0
}
