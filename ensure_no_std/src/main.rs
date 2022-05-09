#![feature(const_ptr_offset_from)]
#![feature(default_alloc_error_handler)]
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

mod objs {
    use dep_obj::dep_type;
    use dep_obj::templates::detached_static_dep_type;
    use alloc::borrow::Cow;

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
    pub type Objs = detached_static_dep_type::Arena<ItemProps>;
}

mod behavior {
    use dep_obj::binding::Binding3;
    use dyn_context::state::State;
    use crate::objs::*;

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

use dep_obj::binding::{Bindings};
use dyn_context::state::{State, StateRefMut};
use objs::*;

fn run(state: &mut dyn State) {
    let item = Item::new(state, behavior::item);
    ItemProps::BASE_WEIGHT.set(state, item, 5.0).immediate();
    ItemProps::CURSED.set(state, item, true).immediate();
    ItemProps::EQUIPPED.set(state, item, true).immediate();
    ItemProps::CURSED.set(state, item, false).immediate();
    item.drop_self(state);
}

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    (&mut Objs::new()).merge_mut_and_then(|state| {
        run(state);
        Objs::drop_self(state);
    }, &mut Bindings::new());
    0
}
