#![feature(default_alloc_error_handler)]
#![feature(start)]

#![deny(warnings)]

#![no_std]

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

macro_attr! {
    #[derive(Debug, Component!)]
    struct ItemData {
        props: ItemProps,
    }
}

macro_attr! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
    struct Item(Id<ItemData>);
}

impl DepObjId for Item { }

dep_type_with_builder! {
    #[derive(Debug)]
    struct ItemProps become props in Item {
        name: Cow<'static, str> = Cow::Borrowed(""),
        equipped: bool = false,
        enhancement: i8 = 0,
    }

    type BaseBuilder<'a> = ItemBuilder<'a>;
}

struct ItemBuilder<'a> {
    item: Item,
    state: &'a mut dyn State,
}

impl<'a> DepObjBaseBuilder<Item> for ItemBuilder<'a> {
    fn id(&self) -> Item { self.item }
    fn state(&self) -> &dyn State { self.state }
    fn state_mut(&mut self) -> &mut dyn State { self.state }
}

impl<'a> ItemBuilder<'a> {
    fn props(
        self,
        f: impl for<'b> FnOnce(ItemPropsBuilder<'b>) -> ItemPropsBuilder<'b>
    ) -> Self {
        f(ItemPropsBuilder::new_priv(self)).base_priv()
    }
}

impl Item {
    fn new(state: &mut dyn State) -> Item {
        let game: &mut Game = state.get_mut();
        game.items.insert(|id| (ItemData { props: ItemProps::new_priv() }, Item(id)))
    }

    fn drop_item(self, state: &mut dyn State) {
        self.drop_bindings_priv(state);
        let game: &mut Game = state.get_mut();
        game.items.remove(self.0);
    }

    fn build<'a>(
        self,
        state: &'a mut dyn State,
        f: impl FnOnce(ItemBuilder<'a>) -> ItemBuilder<'a>
    ) {
        f(ItemBuilder { item: self, state });
    }

    dep_obj! {
        fn props(self as this, game: Game) -> (ItemProps) {
            if mut {
                &mut game.items[this.0].props
            } else {
                &game.items[this.0].props
            }
        }
    }
}


#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let tree = &mut WidgetTree::new();
    let widget = Widget::new(tree, tree.root());
    assert_eq!(widget.parent(tree), Some(tree.root()));
    widget.drop(tree);
    0
}
