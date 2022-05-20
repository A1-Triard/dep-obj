#![feature(const_mut_refs)]
#![feature(const_ptr_offset_from)]
#![feature(const_type_id)]
#![feature(never_type)]
#![feature(unchecked_math)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]

#![allow(clippy::collapsible_else_if)]
#![allow(clippy::option_map_unit_fn)]
#![allow(clippy::type_complexity)]

#![no_std]

extern crate alloc;

mod base;
pub use base::*;

pub mod binding;

#[cfg(docsrs)]
pub mod example {
    //! The [`dep_type`], [`dep_obj`](crate::dep_obj) / [`impl_dep_obj`], and [`with_builder`]
    //! macro expansion example.
    //!
    //! ```ignore
    //! dep_type! {
    //!     #[derive(Debug)]
    //!     pub struct MyDepType[MyDepTypeId] {
    //!         prop_1: bool = false,
    //!         prop_2: i32 = 10,
    //!     }
    //! }
    //!
    //! macro_attr! {
    //!     #[derive(Component!(stop=MyDepTypeStop), Debug)]
    //!     struct MyDepTypePrivateData {
    //!         dep_data: MyDepType,
    //!     }
    //! }
    //!
    //! impl ComponentStop for MyDepTypeStop {
    //!     with_arena_in_state_part!(MyApp { .my_dep_types });
    //!
    //!     fn stop(&self, state: &mut dyn State, id: Id<MyDepTypePrivateData>) {
    //!         MyDepTypeId(id).drop_bindings_priv(state);
    //!     }
    //! }
    //!
    //! macro_attr! {
    //!     #[derive(NewtypeComponentId!, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    //!     pub struct MyDepTypeId(Id<MyDepTypePrivateData>);
    //! }
    //!
    //! impl DetachedDepObjId for MyDepTypeId { }
    //!
    //! #[derive(Debug, Stop)]
    //! pub struct MyApp {
    //!     my_dep_types: Arena<MyDepTypePrivateData>,
    //! }
    //!
    //! impl MyDepTypeId {
    //!     pub fn new(state: &mut dyn State) -> MyDepTypeId {
    //!         let app: &mut MyApp = state.get_mut();
    //!         app.my_dep_types.insert(|id| (MyDepTypePrivateData {
    //!             dep_data: MyDepType::new_priv()
    //!         }, MyDepTypeId(id)))
    //!     }
    //!
    //!     pub fn drop_self(self, state: &mut dyn State) {
    //!         self.drop_bindings_priv(state);
    //!         let app: &mut MyApp = state.get_mut();
    //!         app.my_dep_types.remove(self.0);
    //!     }
    //!
    //!     with_builder!(MyDepType);
    //! }
    //!
    //! impl_dep_obj!(MyDepTypeId {
    //!     type MyDepType => MyApp { .my_dep_types } | .dep_data
    //! });
    //!
    //! // OR equvalent `dep_obj!` call:
    //!
    //! dep_obj! {
    //!     impl MyDepTypeId {
    //!         fn(self as this, app: MyApp) -> (MyDepType) {
    //!             if mut {
    //!                 &mut app.my_dep_types[this.0].dep_data
    //!             } else {
    //!                 &app.my_dep_types[this.0].dep_data
    //!             }
    //!         }
    //!     }
    //! }
    //! ```

    use crate::{DetachedDepObjId, impl_dep_obj, dep_type, with_builder};
    use components_arena::{Arena, Component, ComponentStop, Id, NewtypeComponentId, with_arena_in_state_part};
    use dyn_context::{SelfState, State, StateExt, Stop};

    dep_type! {
        #[derive(Debug)]
        pub struct MyDepType[MyDepTypeId] {
            prop_1: bool = false,
            prop_2: i32 = 10,
        }
    }

    #[derive(Debug)]
    struct MyDepTypePrivateData {
        dep_data: MyDepType,
    }

    Component!((stop=MyDepTypeStop) struct MyDepTypePrivateData { .. });

    impl ComponentStop for MyDepTypeStop {
        with_arena_in_state_part!(MyApp { .my_dep_types });

        fn stop(&self, state: &mut dyn State, id: Id<MyDepTypePrivateData>) {
            MyDepTypeId(id).drop_bindings_priv(state);
        }
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct MyDepTypeId(Id<MyDepTypePrivateData>);

    NewtypeComponentId!(() pub struct MyDepTypeId(Id<MyDepTypePrivateData>););

    impl DetachedDepObjId for MyDepTypeId { }

    #[derive(Debug, Stop)]
    pub struct MyApp {
        my_dep_types: Arena<MyDepTypePrivateData>,
    }

    impl SelfState for MyApp { }

    impl MyDepTypeId {
        pub fn new(state: &mut dyn State) -> MyDepTypeId {
            let app: &mut MyApp = state.get_mut();
            app.my_dep_types.insert(|id| (MyDepTypePrivateData {
                dep_data: MyDepType::new_priv()
            }, MyDepTypeId(id)))
        }

        pub fn drop_self(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let app: &mut MyApp = state.get_mut();
            app.my_dep_types.remove(self.0);
        }

        with_builder!(MyDepType);
    }

    impl_dep_obj!(MyDepTypeId {
        type MyDepType => MyApp { .my_dep_types } | .dep_data
    });
}

#[doc(hidden)]
pub use alloc::vec::Vec as std_vec_Vec;
#[doc(hidden)]
pub use alloc::boxed::Box as std_boxed_Box;
#[doc(hidden)]
pub use components_arena::ComponentId as components_arena_ComponentId;
#[doc(hidden)]
pub use components_arena::RawId as components_arena_RawId;
#[doc(hidden)]
pub use core::any::Any as std_any_Any;
#[doc(hidden)]
pub use core::any::TypeId as std_any_TypeId;
#[doc(hidden)]
pub use core::compile_error as std_compile_error;
#[doc(hidden)]
pub use core::concat as std_concat;
#[doc(hidden)]
pub use core::convert::From as std_convert_From;
#[doc(hidden)]
pub use core::default::Default as std_default_Default;
#[doc(hidden)]
pub use core::fmt::Debug as std_fmt_Debug;
#[doc(hidden)]
pub use core::mem::take as std_mem_take;
#[doc(hidden)]
pub use core::option::Option as std_option_Option;
#[doc(hidden)]
pub use core::stringify as std_stringify;
#[doc(hidden)]
pub use dyn_context::State as dyn_context_State;
#[doc(hidden)]
pub use dyn_context::StateExt as dyn_context_StateExt;
#[doc(hidden)]
pub use generics::concat as generics_concat;
#[doc(hidden)]
pub use generics::parse as generics_parse;
#[doc(hidden)]
pub use indoc::indoc as indoc_indoc;
#[doc(hidden)]
pub use memoffset::offset_of as memoffset_offset_of;
#[doc(hidden)]
pub use paste::paste as paste_paste;

use crate::binding::*;
use alloc::boxed::Box;
use alloc::collections::{TryReserveError, VecDeque};
use alloc::vec::Vec;
use components_arena::{Arena, ArenaItemsIntoValues, Component, ComponentId, Id, RawId};
use core::any::{Any, TypeId};
use core::fmt::Debug;
use core::iter::once;
use core::mem::{replace, take};
use core::ops::{Deref, DerefMut};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::State;
use educe::Educe;
use macro_attr_2018::macro_attr;
use phantom_type::PhantomType;

macro_attr! {
    #[derive(Educe, Component!(class=ItemHandlerComponent))]
    #[educe(Debug)]
    struct ItemHandler<ItemType: Convenient> {
        handler: Box<dyn Handler<ItemChange<ItemType>>>,
        update: Option<BindingBase<()>>,
    }
}

macro_attr! {
    #[derive(Educe, Component!(class=HandlerComponent))]
    #[educe(Debug, Clone)]
    struct BoxedHandler<T>(Box<dyn Handler<T>>);
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Change<PropType: Convenient> {
    pub old: PropType,
    pub new: PropType,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ItemChangeAction<ItemType: Convenient> {
    Insert { prev: Option<ItemType> },
    Remove,
    UpdateInsert { prev: Option<ItemType> },
    UpdateRemove,
    MoveInsert { prev: Option<ItemType> },
    MoveRemove,
}

impl<ItemType: Convenient> ItemChangeAction<ItemType> {
    pub fn is_insert(&self) -> bool {
        matches!(self, ItemChangeAction::Insert { .. })
    }

    pub fn is_remove(&self) -> bool { self == &ItemChangeAction::Remove }

    pub fn is_update_insert(&self) -> bool {
        matches!(self, ItemChangeAction::UpdateInsert { .. })
    }

    pub fn is_update_remove(&self) -> bool { self == &ItemChangeAction::UpdateRemove }

    pub fn is_move_insert(&self) -> bool {
        matches!(self, ItemChangeAction::MoveInsert { .. })
    }

    pub fn is_move_remove(&self) -> bool { self == &ItemChangeAction::MoveRemove }

    pub fn as_insert_prev(&self) -> Option<&Option<ItemType>> {
        if let ItemChangeAction::Insert { prev } = self { Some(prev) } else { None }
    }

    pub fn as_update_insert_prev(&self) -> Option<&Option<ItemType>> {
        if let ItemChangeAction::UpdateInsert { prev } = self { Some(prev) } else { None }
    }

    pub fn as_insert_or_update_insert_prev(&self) -> Option<&Option<ItemType>> {
        match self {
            ItemChangeAction::Insert { prev } => Some(prev),
            ItemChangeAction::UpdateInsert { prev } => Some(prev),
            _ => None
        }
    }

    pub fn as_move_insert_prev(&self) -> Option<&Option<ItemType>> {
        if let ItemChangeAction::MoveInsert { prev } = self { Some(prev) } else { None }
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ItemChange<ItemType: Convenient> {
    pub item: ItemType,
    pub action: ItemChangeAction<ItemType>,
}

impl<ItemType: Convenient> ItemChange<ItemType> {
    pub fn is_insert(&self) -> bool { self.action.is_insert() }

    pub fn is_remove(&self) -> bool { self.action.is_remove() }

    pub fn is_update_insert(&self) -> bool { self.action.is_update_insert() }

    pub fn is_update_remove(&self) -> bool { self.action.is_update_remove() }

    pub fn is_move_insert(&self) -> bool { self.action.is_move_insert() }

    pub fn is_move_remove(&self) -> bool { self.action.is_move_remove() }

    pub fn as_insert_prev(&self) -> Option<&Option<ItemType>> { self.action.as_insert_prev() }

    pub fn as_update_insert_prev(&self) -> Option<&Option<ItemType>> { self.action.as_update_insert_prev() }

    pub fn as_insert_or_update_insert_prev(&self) -> Option<&Option<ItemType>> {
        self.action.as_insert_or_update_insert_prev()
    }

    pub fn as_move_insert_prev(&self) -> Option<&Option<ItemType>> { self.action.as_move_insert_prev() }
}

#[derive(Debug)]
struct DepPropHandlers<PropType: Convenient> {
    children_has_handlers: Option<bool>,
    value_handlers: Arena<BoxedHandler<PropType>>,
    change_handlers: Arena<BoxedHandler<Change<PropType>>>,
    change_initial_handler: Option<Box<dyn Handler<Change<PropType>>>>,
    change_final_handler: Option<Box<dyn Handler<Change<PropType>>>>,
}

#[derive(Debug)]
struct DepPropHandlersCopy<PropType: Convenient> {
    notify_children: bool,
    value_handlers: ArenaItemsIntoValues<BoxedHandler<PropType>>,
    change_handlers: ArenaItemsIntoValues<BoxedHandler<Change<PropType>>>,
    change_initial_handler: Option<Box<dyn Handler<Change<PropType>>>>,
    change_final_handler: Option<Box<dyn Handler<Change<PropType>>>>,
}

impl<PropType: Convenient> DepPropHandlers<PropType> {
    const fn new(inherits: bool) -> Self {
        DepPropHandlers {
            children_has_handlers: if inherits { Some(false) } else { None },
            value_handlers: Arena::new(),
            change_handlers: Arena::new(),
            change_initial_handler: None,
            change_final_handler: None,
        }
    }

    fn is_empty(&self) -> bool {
        self.children_has_handlers != Some(true) &&
            self.value_handlers.items().is_empty() &&
            self.change_handlers.items().is_empty() &&
            self.change_initial_handler.is_none() &&
            self.change_final_handler.is_none()
    }

    fn take_all(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        handlers.extend(take(&mut self.value_handlers).into_items().into_values().map(|x| x.0.into_any()));
        handlers.extend(take(&mut self.change_handlers).into_items().into_values().map(|x| x.0.into_any()));
        self.change_initial_handler.take().map(|x| handlers.push(x.into_any()));
        self.change_final_handler.take().map(|x| handlers.push(x.into_any()));
    }

    fn clone(&self) -> DepPropHandlersCopy<PropType> {
        DepPropHandlersCopy {
            notify_children: self.children_has_handlers == Some(true),
            value_handlers: Clone::clone(self.value_handlers.items()).into_values(),
            change_handlers: Clone::clone(self.change_handlers.items()).into_values(),
            change_initial_handler: self.change_initial_handler.clone(),
            change_final_handler: self.change_final_handler.clone(),
        }
    }
}

impl<PropType: Convenient> DepPropHandlersCopy<PropType> {
    fn execute<Owner: DepType>(
        self,
        state: &mut dyn State,
        change: &Change<PropType>,
        id: Owner::Id,
        prop: DepProp<Owner, PropType>
    ) where Owner::Id: DepObj<Owner> {
        if let Some(change_initial_handler) = self.change_initial_handler {
            change_initial_handler.execute(state, change.clone());
        }
        for handler in self.value_handlers {
            handler.0.execute(state, change.new.clone());
        }
        for handler in self.change_handlers {
            handler.0.execute(state, change.clone());
        }
        if let Some(change_final_handler) = self.change_final_handler {
            change_final_handler.execute(state, change.clone());
        }
        if self.notify_children {
            prop.notify_children(state, id, change);
        }
    }
}

#[derive(Debug)]
pub struct DepPropEntry<PropType: Convenient> {
    default: &'static PropType,
    style: Option<PropType>,
    local: Option<PropType>,
    handlers: DepPropHandlers<PropType>,
    binding: Option<BindingBase<PropType>>,
    queue: Option<VecDeque<Option<PropType>>>,
    enqueue: bool,
}

impl<PropType: Convenient> DepPropEntry<PropType> {
    pub const fn new(default: &'static PropType, inherits: bool) -> Self {
        DepPropEntry {
            default,
            handlers: DepPropHandlers::new(inherits),
            style: None,
            local: None,
            binding: None,
            queue: None,
            enqueue: false,
        }
    }

    fn inherits(&self) -> bool { self.handlers.children_has_handlers.is_some() }

    #[doc(hidden)]
    pub fn take_all_handlers(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        self.handlers.take_all(handlers);
    }

    #[doc(hidden)]
    pub fn binding(&self) -> Option<BindingBase<PropType>> {
        self.binding
    }
}

#[derive(Debug)]
pub struct DepEventEntry<ArgsType: DepEventArgs> {
    bubble: bool,
    handlers: Arena<BoxedHandler<ArgsType>>,
}

impl<ArgsType: DepEventArgs> DepEventEntry<ArgsType> {
    pub const fn new(bubble: bool) -> Self {
        DepEventEntry {
            bubble,
            handlers: Arena::new(),
        }
    }

    #[doc(hidden)]
    pub fn take_all_handlers(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        handlers.extend(take(&mut self.handlers).into_items().into_values().map(|x| x.0.into_any()));
    }
}


#[derive(Debug)]
struct DepVecHandlers<ItemType: Convenient> {
    changed_handlers: Arena<BoxedHandler<()>>,
    item_handlers: Arena<ItemHandler<ItemType>>,
    item_initial_final_handler: Option<ItemHandler<ItemType>>,
}

impl<ItemType: Convenient> DepVecHandlers<ItemType> {
    const fn new() -> Self {
        DepVecHandlers {
            changed_handlers: Arena::new(),
            item_handlers: Arena::new(),
            item_initial_final_handler: None,
        }
    }

    fn take_all(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        handlers.extend(take(&mut self.changed_handlers).into_items().into_values().map(|x| x.0.into_any()));
        handlers.extend(take(&mut self.item_handlers).into_items().into_values().map(|x| x.handler.into_any()));
        self.item_initial_final_handler.take().map(|x| handlers.push(x.handler.into_any()));
    }

    fn clone(&self) -> DepVecHandlersCopy<ItemType> {
        DepVecHandlersCopy {
            changed_handlers: self.changed_handlers.items().clone().into_values(),
            item_handlers: self.item_handlers.items().values().map(|x| x.handler.clone()).collect(),
            item_initial_final_handler: self.item_initial_final_handler.as_ref().map(|x| x.handler.clone()),
        }
    }
}

#[derive(Debug)]
struct DepVecHandlersCopy<ItemType: Convenient> {
    changed_handlers: ArenaItemsIntoValues<BoxedHandler<()>>,
    item_handlers: Vec<Box<dyn Handler<ItemChange<ItemType>>>>,
    item_initial_final_handler: Option<Box<dyn Handler<ItemChange<ItemType>>>>,
}

impl<ItemType: Convenient> DepVecHandlersCopy<ItemType> {
    fn execute_insert(self, state: &mut dyn State, prev: Option<ItemType>, items: &[ItemType]) {
        for handler in self.item_initial_final_handler.into_iter().chain(self.item_handlers) {
            for (item, prev) in items.iter().zip(once(prev.as_ref()).chain(items.iter().map(Some))) {
                handler.execute(state, ItemChange {
                    action: ItemChangeAction::Insert { prev: prev.cloned() },
                    item: item.clone()
                });
            }
        }
        for handler in self.changed_handlers {
            handler.0.execute(state, ());
        }
    }

    fn execute_remove(self, state: &mut dyn State, items: &[ItemType]) {
        for handler in self.item_handlers.into_iter().chain(self.item_initial_final_handler.into_iter()) {
            for item in items {
                handler.execute(state, ItemChange { action: ItemChangeAction::Remove, item: item.clone() });
            }
        }
        for handler in self.changed_handlers {
            handler.0.execute(state, ());
        }
    }

    fn execute_move(self, state: &mut dyn State, prev: Option<ItemType>, item: ItemType) {
        for handler in self.item_handlers.iter().chain(self.item_initial_final_handler.iter()) {
            handler.execute(state, ItemChange { action: ItemChangeAction::MoveRemove, item: item.clone() });
        }
        for handler in self.item_initial_final_handler.into_iter().chain(self.item_handlers) {
            handler.execute(state, ItemChange {
                action: ItemChangeAction::MoveInsert { prev: prev.clone() },
                item: item.clone()
            });
        }
        for handler in self.changed_handlers {
            handler.0.execute(state, ());
        }
    }
}

#[derive(Debug)]
pub struct DepVecEntry<ItemType: Convenient> {
    items: Vec<ItemType>,
    handlers: DepVecHandlers<ItemType>,
    queue: Option<VecDeque<DepVecModification<ItemType>>>,
    enqueue: bool,
}

impl<ItemType: Convenient> DepVecEntry<ItemType> {
    pub const fn new() -> Self {
        DepVecEntry {
            items: Vec::new(),
            handlers: DepVecHandlers::new(),
            queue: None,
            enqueue: false,
        }
    }

    #[doc(hidden)]
    pub fn take_all_handlers(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        self.handlers.take_all(handlers);
    }

    #[doc(hidden)]
    pub fn collect_all_bindings(&self, bindings: &mut Vec<AnyBindingBase>) {
        bindings.extend(
            self.handlers.item_handlers.items().values().filter_map(|x| x.update).map(|x| {
                let x: AnyBindingBase = x.into();
                x
            })
        );
        if let Some(binding) = self.handlers.item_initial_final_handler.as_ref().and_then(|x| x.update) {
            bindings.push(binding.into());
        }
    }
}

#[derive(Debug)]
pub struct BaseDepObjCore<Owner: DepType> {
    style: Option<Style<Owner>>,
    added_bindings: Arena<AnyBindingBase>,
}

impl<Owner: DepType> BaseDepObjCore<Owner> {
    pub const fn new() -> Self {
        BaseDepObjCore {
            style: None,
            added_bindings: Arena::new(),
        }
    }

    #[doc(hidden)]
    pub fn collect_bindings(&self) -> Vec<AnyBindingBase> {
        self.added_bindings.items().values().copied().collect()
    }
}

pub trait DepObjId: ComponentId {
    fn parent(self, state: &dyn State) -> Option<Self>;
    fn next(self, state: &dyn State) -> Self;
    fn first_child(self, state: &dyn State) -> Option<Self>;

    fn add_binding_raw<Owner: DepType<Id=Self>, T: Convenient>(
        self, state: &mut dyn State, binding: BindingBase<T>
    ) where Owner: 'static, Owner::Id: DepObj<Owner> {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.into_raw());
        let binding_id = obj.core_base_priv_mut().added_bindings.insert(|id| (binding.into(), id));
        binding.set_holder(state, Box::new(AddedBindingHolder::<Owner> { id: self, binding_id }));
    }

    fn add_binding<Owner: DepType<Id=Self>, T: Convenient>(
        self, state: &mut dyn State, binding: impl Into<BindingBase<T>>
    ) where Owner: 'static, Owner::Id: DepObj<Owner> {
        self.add_binding_raw::<Owner, _>(state, binding.into())
    }

    fn apply_style<Owner: DepType<Id=Self>>(
        self,
        state: &mut dyn State,
        style: Option<Style<Owner>>,
    ) -> Option<Style<Owner>> where Owner::Id: DepObj<Owner> {
        let mut on_changed = Vec::new();
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.into_raw());
        let old = obj.core_base_priv_mut().style.take();
        if let Some(old) = old.as_ref() {
            old.setters
                .iter()
                .filter(|setter| style.as_ref().map_or(
                    true,
                    |new| new.setters.binary_search_by_key(
                        &setter.prop_offset(),
                        |x| x.prop_offset()
                    ).is_err()
                ))
                .filter_map(|setter| setter.un_apply(state, self, true))
                .for_each(|x| on_changed.push(x))
            ;
        }
        if let Some(new) = style.as_ref() {
            new.setters
                .iter()
                .filter_map(|setter| setter.un_apply(state, self, false))
                .for_each(|x| on_changed.push(x))
            ;
        }
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.into_raw());
        obj.core_base_priv_mut().style = style;
        for on_changed in on_changed {
            on_changed(state);
        }
        old
    }
}

pub trait DetachedDepObjId: ComponentId { }

impl<T: DetachedDepObjId> DepObjId for T {
    fn parent(self, _state: &dyn State) -> Option<Self> { None }

    fn next(self, _state: &dyn State) -> Self { self }

    fn first_child(self, _state: &dyn State) -> Option<Self> { None }
}

/// A dependency type.
/// Use the [`dep_type`] macro to create a type implementing this trait.
///
/// # Examples
///
/// ```rust
/// # #![feature(const_ptr_offset_from)]
/// # #![feature(const_type_id)]
/// use components_arena::{Arena, Component, NewtypeComponentId, Id};
/// use dep_obj::{DetachedDepObjId, dep_type, impl_dep_obj};
/// use dep_obj::binding::{Bindings, Binding, Binding1};
/// use dyn_context::{State, StateExt};
/// use macro_attr_2018::macro_attr;
///
/// dep_type! {
///     #[derive(Debug)]
///     pub struct MyDepType[MyDepTypeId] {
///         prop_1: bool = false,
///         prop_2: i32 = 10,
///     }
/// }
///
/// macro_attr! {
///     #[derive(Component!, Debug)]
///     struct MyDepTypePrivateData {
///         dep_data: MyDepType,
///     }
/// }
///
/// macro_attr! {
///     #[derive(NewtypeComponentId!, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
///     pub struct MyDepTypeId(Id<MyDepTypePrivateData>);
/// }
///
/// impl DetachedDepObjId for MyDepTypeId { }
///
/// #[derive(State)]
/// #[state(part)]
/// pub struct MyApp {
///     #[state(part)]
///     bindings: Bindings,
///     my_dep_types: Arena<MyDepTypePrivateData>,
///     res: Binding<i32>,
/// }
///
/// impl MyDepTypeId {
///     pub fn new(state: &mut dyn State) -> MyDepTypeId {
///         let app: &mut MyApp = state.get_mut();
///         app.my_dep_types.insert(|id| (MyDepTypePrivateData {
///             dep_data: MyDepType::new_priv()
///         }, MyDepTypeId(id)))
///     }
///
///     pub fn drop_my_dep_type(self, state: &mut dyn State) {
///         self.drop_bindings_priv(state);
///         let app: &mut MyApp = state.get_mut();
///         app.my_dep_types.remove(self.0);
///     }
/// }
///
/// impl_dep_obj!(MyDepTypeId {
///     type MyDepType => MyApp { .my_dep_types } | .dep_data
/// });
///
/// fn main() {
///     let mut bindings = Bindings::new();
///     let res = Binding1::new(&mut bindings, (), |(), x| Some(x));
///     let app = &mut MyApp {
///         bindings,
///         my_dep_types: Arena::new(),
///         res: res.into(),
///     };
///     let id = MyDepTypeId::new(app);
///     res.set_source_1(app, &mut MyDepType::PROP_2.value_source(id));
///     assert_eq!(app.res.get_value(app), Some(10));
///     MyDepType::PROP_2.set(app, id, 5).immediate();
///     assert_eq!(app.res.get_value(app), Some(5));
///     id.drop_my_dep_type(app);
///     res.drop_self(app);
/// }
/// ```
pub trait DepType: Debug {
    type Id: DepObjId;

    #[doc(hidden)]
    fn core_base_priv(&self) -> &BaseDepObjCore<Self> where Self: Sized;

    #[doc(hidden)]
    fn core_base_priv_mut(&mut self) -> &mut BaseDepObjCore<Self> where Self: Sized;

    #[doc(hidden)]
    fn take_all_handlers(&mut self) -> Vec<Box<dyn AnyHandler>>;

    #[doc(hidden)]
    fn collect_all_bindings(&self) -> Vec<AnyBindingBase>;

    #[doc(hidden)]
    fn update_parent_children_has_handlers(state: &mut dyn State, id: RawId) where Self: Sized;
}

pub trait DepEventArgs: Convenient {
    fn handled(&self) -> bool;
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct DepEvent<Owner: DepType, ArgsType: DepEventArgs> {
    offset: usize,
    _phantom: PhantomType<(Owner, ArgsType)>
}

impl<Owner: DepType, ArgsType: DepEventArgs> DepEvent<Owner, ArgsType> {
    /// # Safety
    ///
    /// The function is not intended for direct use.
    /// Using the [`dep_type`] macro garantees the function safe use.
    pub const unsafe fn new(offset: usize) -> Self {
        DepEvent { offset, _phantom: PhantomType::new() }
    }

    pub fn offset(self) -> usize { self.offset }

    fn entry(self, owner: &Owner) -> &DepEventEntry<ArgsType> {
        unsafe {
            let entry = (owner as *const _ as usize).unchecked_add(self.offset);
            let entry = entry as *const DepEventEntry<ArgsType>;
            &*entry
        }
    }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepEventEntry<ArgsType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepEventEntry<ArgsType>;
            &mut *entry
        }
    }

    fn raise_raw(
        self, state: &mut dyn State, id: Owner::Id, args: &ArgsType
    ) -> bool where Owner::Id: DepObj<Owner> {
        let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
        let entry = self.entry(&obj);
        let bubble = entry.bubble;
        let handlers = entry.handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, args.clone());
        }
        bubble
    }

    pub fn raise<X: Convenient>(
        self, state: &mut dyn State, mut id: Owner::Id, args: ArgsType
    ) -> Re<X> where Owner::Id: DepObj<Owner> {
        let bubble = self.raise_raw(state, id, &args);
        if !bubble || args.handled() { return Re::Continue; }
        while let Some(parent) = id.parent(state) {
            id = parent;
            let bubble = self.raise_raw(state, id, &args);
            debug_assert!(bubble);
            if args.handled() { return Re::Continue; }
        }
        Re::Continue
    }

    pub fn source(self, id: Owner::Id) -> DepEventSource<Owner, ArgsType> where
        Owner::Id: DepObj<Owner> {

        DepEventSource { id, event: self }
    }
}

/// A dependency property.
#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct DepProp<Owner: DepType, PropType: Convenient> {
    offset: usize,
    _phantom: PhantomType<(Owner, PropType)>
}

impl<Owner: DepType, PropType: Convenient> DepProp<Owner, PropType> {
    /// Creates dependency property.
    ///
    /// # Safety
    ///
    /// The function is not intended for direct use.
    /// Using the [`dep_type`] macro garantees the function safe use.
    pub const unsafe fn new(offset: usize) -> Self {
        DepProp { offset, _phantom: PhantomType::new() }
    }

    pub fn offset(self) -> usize { self.offset }

    fn entry(self, owner: &Owner) -> &DepPropEntry<PropType> {
        unsafe {
            let entry = (owner as *const _ as usize).unchecked_add(self.offset);
            let entry = entry as *const DepPropEntry<PropType>;
            &*entry
        }
    }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepPropEntry<PropType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepPropEntry<PropType>;
            &mut *entry
        }
    }

    fn unstyled_non_local_value<T>(
        self, state: &dyn State, id: Owner::Id, f: impl FnOnce(&PropType) -> T
    ) -> T where Owner::Id: DepObj<Owner> {
        let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
        let entry = self.entry(&obj);
        if entry.inherits() {
            if let Some(parent) = id.parent(state) {
                self.current_value(state, parent, f)
            } else {
                f(entry.default)
            }
        } else {
            f(entry.default)
        }
    }

    fn non_local_value<T>(
        self, state: &dyn State, id: Owner::Id, f: impl FnOnce(&PropType) -> T
    ) -> T where Owner::Id: DepObj<Owner> {
        let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
        let entry = self.entry(&obj);
        if let Some(value) = entry.style.as_ref() {
            f(value)
        } else {
            self.unstyled_non_local_value(state, id, f)
        }
    }

    fn current_value<T>(
        self, state: &dyn State, id: Owner::Id, f: impl FnOnce(&PropType) -> T
    ) -> T where Owner::Id: DepObj<Owner> {
        let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
        let entry = self.entry(&obj);
        if let Some(value) = entry.local.as_ref() {
            f(value)
        } else {
            self.non_local_value(state, id, f)
        }
    }

    #[doc(hidden)]
    pub fn update_parent_children_has_handlers(
        self, state: &mut dyn State, mut id: Owner::Id
    ) where Owner::Id: DepObj<Owner> {
        while let Some(parent) = id.parent(state) {
            id = parent;
            let children_has_handlers = if let Some(first_child) = id.first_child(state) {
                let mut child = first_child;
                loop {
                    let obj = <Owner::Id as DepObj<Owner>>::get(state, child.into_raw());
                    let entry = self.entry(&obj);
                    debug_assert!(entry.inherits());
                    if !entry.handlers.is_empty() { break true; }
                    child = child.next(state);
                    if child == first_child { break false; }
                }
            } else {
                false
            };
            let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
            let entry_mut = self.entry_mut(&mut obj);
            if children_has_handlers == entry_mut.handlers.children_has_handlers.unwrap() { return; }
            entry_mut.handlers.children_has_handlers = Some(children_has_handlers);
        }
    }

    fn notify_children(
        self,
        state: &mut dyn State,
        id: Owner::Id,
        change: &Change<PropType>,
    ) where Owner::Id: DepObj<Owner> {
        if let Some(first_child) = id.first_child(state) {
            let mut child = first_child;
            loop {
                let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, child.into_raw());
                let entry_mut = self.entry_mut(&mut obj);
                debug_assert!(entry_mut.inherits());
                if entry_mut.local.is_none() && entry_mut.style.is_none() {
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute(state, change, child, self);
                }
                child = child.next(state);
                if child == first_child { break; }
            }
        }
    }

    fn un_set_core(
        self, state: &mut dyn State, id: Owner::Id, value: Option<PropType>
    ) where Owner::Id: DepObj<Owner> {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
        let entry_mut = self.entry_mut(&mut obj);
        let old = replace(&mut entry_mut.local, value.clone());
        if old == value { return; }
        let handlers = entry_mut.handlers.clone();
        let change = if old.is_some() && value.is_some() {
            unsafe { Change { old: old.unwrap_unchecked(), new: value.unwrap_unchecked() } }
        } else {
            if let Some(change) = self.non_local_value(state, id, |non_local| {
                let old_ref = old.as_ref().unwrap_or(non_local);
                let value_ref = value.as_ref().unwrap_or(non_local);
                if old_ref == value_ref {
                    None
                } else {
                    let old = old.unwrap_or_else(|| non_local.clone());
                    let new = value.unwrap_or_else(|| non_local.clone());
                    Some(Change { old, new })
                }
            }) {
                change
            } else {
                return;
            }
        };
        handlers.execute(state, &change, id, self);
    }

    fn un_set(
        self, state: &mut dyn State, id: Owner::Id, mut value: Option<PropType>
    ) where Owner::Id: DepObj<Owner> {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
        let entry_mut = self.entry_mut(&mut obj);
        if replace(&mut entry_mut.enqueue, true) {
            let queue = entry_mut.queue.get_or_insert_with(VecDeque::new);
            queue.push_back(value);
            return;
        }
        loop {
            self.un_set_core(state, id, value);
            let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
            let entry_mut = self.entry_mut(&mut obj);
            if let Some(queue) = entry_mut.queue.as_mut() {
                if let Some(queue_head) = queue.pop_front() { value = queue_head; } else { break; }
            } else {
                break;
            }
        }
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
        let entry_mut = self.entry_mut(&mut obj);
        entry_mut.enqueue = false;
    }

    pub fn set<X: Convenient>(
        self, state: &mut dyn State, id: Owner::Id, value: PropType
    ) -> Re<X> where Owner::Id: DepObj<Owner> {
        self.un_set(state, id, Some(value));
        Re::Continue
    }

    pub fn unset<X: Convenient>(
        self, state: &mut dyn State, id: Owner::Id
    ) -> Re<X> where Owner::Id: DepObj<Owner> {
        self.un_set(state, id, None);
        Re::Continue
    }

    fn bind_raw(
        self,
        state: &mut dyn State,
        id: Owner::Id,
        binding: BindingBase<PropType>
    ) where Owner: 'static, Owner::Id: DepObj<Owner> {
        self.unbind(state, id);
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
        let entry_mut = self.entry_mut(&mut obj);
        entry_mut.binding = Some(binding);
        binding.set_target(state, Box::new(DepPropSet { prop: self, id }));
        binding.set_holder(state, Box::new(DepPropSet { prop: self, id }));
    }

    pub fn bind(
        self,
        state: &mut dyn State,
        id: Owner::Id,
        binding: impl Into<BindingBase<PropType>>
    ) where Owner: 'static, Owner::Id: DepObj<Owner> {
        self.bind_raw(state, id, binding.into());
    }

    pub fn unbind(
        self, state: &mut dyn State, id: Owner::Id
    ) where Owner::Id: DepObj<Owner> {
        if let Some(binding) = {
            let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
            let entry_mut = self.entry_mut(&mut obj);
            entry_mut.binding
        } {
            binding.drop_self(state);
        }
    }

    fn clear_binding(self, state: &mut dyn State, id: Owner::Id) where Owner::Id: DepObj<Owner> {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
        let entry_mut = self.entry_mut(&mut obj);
        let ok = entry_mut.binding.take().is_some();
        debug_assert!(ok);
    }

    pub fn value_source(self, id: Owner::Id) -> DepPropValueSource<Owner, PropType> {
        DepPropValueSource { id, prop: self }
    }

    pub fn change_source(self, id: Owner::Id) -> DepPropChangeSource<Owner, PropType> {
        DepPropChangeSource { id, prop: self }
    }

    pub fn change_initial_source(self, id: Owner::Id) -> DepPropChangeInitialSource<Owner, PropType> {
        DepPropChangeInitialSource { id, prop: self }
    }

    pub fn change_final_source(self, id: Owner::Id) -> DepPropChangeFinalSource<Owner, PropType> {
        DepPropChangeFinalSource { id, prop: self }
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DepPropSet<Owner: DepType, PropType: Convenient> where
    Owner::Id: DepObj<Owner> {

    id: Owner::Id,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> Target<PropType> for DepPropSet<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    fn execute(&self, state: &mut dyn State, value: PropType) {
        self.prop.set(state, self.id, value).immediate();
    }
}

impl<Owner: DepType, PropType: Convenient> Holder for DepPropSet<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    fn release(&self, state: &mut dyn State) {
        self.prop.clear_binding(state, self.id);
    }
}

#[derive(Debug)]
enum DepVecModification<ItemType: Convenient> {
    Clear,
    Insert(DepVecInsertPos<ItemType>, ItemType),
    Remove(DepVecItemPos<ItemType>),
    Move(DepVecItemPos<ItemType>, DepVecInsertPos<ItemType>),
    ExtendFrom(Vec<ItemType>),
    Update(Option<Id<ItemHandler<ItemType>>>),
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub enum DepVecItemPos<ItemType: Convenient> {
    FirstItem,
    LastItem,
    Item(ItemType),
}

impl<ItemType: Convenient> DepVecItemPos<ItemType> {
    fn find(&self, items: &[ItemType]) -> usize {
        match self {
            DepVecItemPos::FirstItem => {
                assert!(!items.is_empty(), "item position not found");
                0
            },
            DepVecItemPos::LastItem => {
                assert!(!items.is_empty(), "item position not found");
                items.len() - 1
            }
            DepVecItemPos::Item(item) => {
                items.iter().enumerate()
                    .find_map(|(i, x)| if x == item { Some(i) } else { None })
                    .expect("item position not found")
            }
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Hash)]
pub enum DepVecInsertPos<ItemType: Convenient> {
    BeforeFirstItem,
    AfterLastItem,
    Before(ItemType),
    After(ItemType)
}

impl<ItemType: Convenient> DepVecInsertPos<ItemType> {
    fn find(&self, items: &[ItemType]) -> usize {
        match self {
            DepVecInsertPos::BeforeFirstItem => 0,
            DepVecInsertPos::AfterLastItem => items.len(),
            DepVecInsertPos::Before(item) => {
                items.iter().enumerate()
                    .find_map(|(i, x)| if x == item { Some(i) } else { None })
                    .expect("insert position not found")
            },
            DepVecInsertPos::After(item) => {
                1 + items.iter().enumerate()
                    .find_map(|(i, x)| if x == item { Some(i) } else { None })
                    .expect("insert position not found")
            },
        }
    }
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct DepVec<Owner: DepType, ItemType: Convenient> {
    offset: usize,
    _phantom: PhantomType<(Owner, ItemType)>
}

impl<Owner: DepType, ItemType: Convenient> DepVec<Owner, ItemType> {
    /// # Safety
    ///
    /// The function is not intended for direct use.
    /// Using the [`dep_type`] macro garantees the function safe use.
    pub const unsafe fn new(offset: usize) -> Self {
        DepVec { offset, _phantom: PhantomType::new() }
    }

    pub fn offset(self) -> usize { self.offset }

    fn entry(self, owner: &Owner) -> &DepVecEntry<ItemType> {
        unsafe {
            let entry = (owner as *const _ as usize).unchecked_add(self.offset);
            let entry = entry as *const DepVecEntry<ItemType>;
            &*entry
        }
    }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepVecEntry<ItemType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepVecEntry<ItemType>;
            &mut *entry
        }
    }

    fn modify(
        self,
        state: &mut dyn State,
        id: Owner::Id,
        mut modification: DepVecModification<ItemType>,
    ) where Owner::Id: DepObj<Owner> {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
        let entry_mut = self.entry_mut(&mut obj);
        if replace(&mut entry_mut.enqueue, true) {
            let queue = entry_mut.queue.get_or_insert_with(VecDeque::new);
            queue.push_back(modification);
            return;
        }
        loop {
            let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
            let entry_mut = self.entry_mut(&mut obj);
            match modification {
                DepVecModification::Clear => {
                    let items = take(&mut entry_mut.items);
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute_remove(state, &items);
                },
                DepVecModification::Insert(pos, item) => {
                    let index = pos.find(&entry_mut.items);
                    let prev = if index == 0 { None } else { Some(entry_mut.items[index - 1].clone()) };
                    entry_mut.items.insert(index, item.clone());
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute_insert(state, prev, &[item]);
                },
                DepVecModification::Remove(pos) => {
                    let index = pos.find(&entry_mut.items);
                    let item = entry_mut.items.remove(index);
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute_remove(state, &[item]);
                },
                DepVecModification::Move(old_pos, new_pos) => {
                    let old_index = old_pos.find(&entry_mut.items);
                    let item = entry_mut.items.remove(old_index);
                    let new_index = new_pos.find(&entry_mut.items);
                    let prev = if new_index == 0 { None } else { Some(entry_mut.items[new_index - 1].clone()) };
                    entry_mut.items.insert(new_index, item.clone());
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute_move(state, prev, item);
                },
                DepVecModification::ExtendFrom(vec) => {
                    let prev = entry_mut.items.last().cloned();
                    entry_mut.items.extend_from_slice(&vec);
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute_insert(state, prev, &vec);
                },
                DepVecModification::Update(handler_id) => {
                    let items = entry_mut.items.clone();
                    let handler = handler_id.map_or_else(
                        || entry_mut.handlers.item_initial_final_handler.as_ref().unwrap().handler.clone(),
                        |handler_id| entry_mut.handlers.item_handlers[handler_id].handler.clone()
                    );
                    for item in &items {
                        handler.execute(state, ItemChange { action: ItemChangeAction::UpdateRemove, item: item.clone() });
                    }
                    for (item, prev) in items.iter().zip(once(None).chain(items.iter().map(Some))) {
                        handler.execute(state, ItemChange {
                            action: ItemChangeAction::UpdateInsert { prev: prev.cloned() },
                            item: item.clone()
                        });
                    }
                },
            };
            let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
            let entry_mut = self.entry_mut(&mut obj);
            if let Some(queue) = entry_mut.queue.as_mut() {
                if let Some(queue_head) = queue.pop_front() { modification = queue_head; } else { break; }
            } else {
                break;
            }
        }
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
        let entry_mut = self.entry_mut(&mut obj);
        entry_mut.enqueue = false;
    }

    pub fn clear<X: Convenient>(
        self, state: &mut dyn State, id: Owner::Id
    ) -> Re<X> where Owner::Id: DepObj<Owner> {
        self.modify(state, id, DepVecModification::Clear);
        Re::Continue
    }

    pub fn push<X: Convenient>(
        self, state: &mut dyn State, id: Owner::Id, item: ItemType
    ) -> Re<X> where Owner::Id: DepObj<Owner> {
        self.insert(state, id, DepVecInsertPos::AfterLastItem, item)
    }

    pub fn insert<X: Convenient>(
        self,
        state: &mut dyn State,
        id: Owner::Id,
        pos: DepVecInsertPos<ItemType>,
        item: ItemType
    ) -> Re<X> where Owner::Id: DepObj<Owner> {
        self.modify(state, id, DepVecModification::Insert(pos, item));
        Re::Continue
    }

    pub fn move_<X: Convenient>(
        self,
        state: &mut dyn State,
        id: Owner::Id,
        old_pos: DepVecItemPos<ItemType>,
        new_pos: DepVecInsertPos<ItemType>
    ) -> Re<X> where Owner::Id: DepObj<Owner> {
        self.modify(state, id, DepVecModification::Move(old_pos, new_pos));
        Re::Continue
    }

    pub fn remove<X: Convenient>(
        self,
        state: &mut dyn State,
        id: Owner::Id,
        pos: DepVecItemPos<ItemType>
    ) -> Re<X> where Owner::Id: DepObj<Owner> {
        self.modify(state, id, DepVecModification::Remove(pos));
        Re::Continue
    }

    pub fn extend_from<X: Convenient>(
        self, state: &mut dyn State, id: Owner::Id, other: Vec<ItemType>
    ) -> Re<X> where Owner::Id: DepObj<Owner> {
        self.modify(state, id, DepVecModification::ExtendFrom(other));
        Re::Continue
    }

    pub fn changed_source(
        self, id: Owner::Id
    ) -> DepVecChangedSource<Owner, ItemType> where Owner::Id: DepObj<Owner> {
        DepVecChangedSource { id, vec: self }
    }

    pub fn item_source(
        self, id: Owner::Id
    ) -> DepVecItemSource<Owner, ItemType> where Owner::Id: DepObj<Owner> {
        DepVecItemSource { id, vec: self, update: None }
    }

    pub fn item_source_with_update(
        self, update: impl Into<BindingBase<()>>, id: Owner::Id
    ) -> DepVecItemSource<Owner, ItemType> where Owner::Id: DepObj<Owner> {
        DepVecItemSource { id, vec: self, update: Some(update.into()) }
    }

    pub fn item_initial_final_source(
        self, id: Owner::Id
    ) -> DepVecItemInitialFinalSource<Owner, ItemType> where Owner::Id: DepObj<Owner> {
        DepVecItemInitialFinalSource { id, vec: self, update: None }
    }

    pub fn item_initial_final_source_with_update(
        self, update: impl Into<BindingBase<()>>, id: Owner::Id
    ) -> DepVecItemInitialFinalSource<Owner, ItemType> where Owner::Id: DepObj<Owner> {
        DepVecItemInitialFinalSource { id, vec: self, update: Some(update.into()) }
    }
}

struct AddedBindingHolder<Owner: DepType> {
    id: Owner::Id,
    binding_id: Id<AnyBindingBase>,
}

impl<Owner: DepType> Holder for AddedBindingHolder<Owner> where
    Owner::Id: DepObj<Owner> {

    fn release(&self, state: &mut dyn State) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        obj.core_base_priv_mut().added_bindings.remove(self.binding_id);
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct Setter<Owner: DepType, PropType: Convenient> {
    prop: DepProp<Owner, PropType>,
    value: PropType,
}

trait AnySetter<Owner: DepType>: Debug + DynClone {
    fn prop_offset(&self) -> usize;
    fn un_apply(
        &self,
        state: &mut dyn State,
        id: Owner::Id,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn State)>>;
}

clone_trait_object!(<Owner: DepType> AnySetter<Owner>);

impl<Owner: DepType + 'static, PropType: Convenient> AnySetter<Owner> for Setter<Owner, PropType> where
    Owner::Id: 'static, Owner::Id: DepObj<Owner> {

    fn prop_offset(&self) -> usize { self.prop.offset }

    fn un_apply(
        &self,
        state: &mut dyn State,
        id: Owner::Id,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn State)>> where {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, id.into_raw());
        let entry_mut = self.prop.entry_mut(&mut obj);
        let value = if unapply { None } else { Some(self.value.clone()) };
        let old = replace(&mut entry_mut.style, value.clone());
        if entry_mut.local.is_some() || old == value { return None; }
        let handlers = entry_mut.handlers.clone();
        let change = if old.is_some() && value.is_some() {
            unsafe { Change { old: old.unwrap_unchecked(), new: value.unwrap_unchecked() } }
        } else {
            self.prop.unstyled_non_local_value(state, id, |unstyled_non_local_value| {
                let old_ref = old.as_ref().unwrap_or(unstyled_non_local_value);
                let value_ref = value.as_ref().unwrap_or(unstyled_non_local_value);
                if old_ref == value_ref {
                    None
                } else {
                    let old = old.unwrap_or_else(|| unstyled_non_local_value.clone());
                    let new = value.unwrap_or_else(|| unstyled_non_local_value.clone());
                    Some(Change { old, new })
                }
            })?
        };
        let prop = self.prop;
        Some(Box::new(move |state: &'_ mut dyn State| handlers.execute(state, &change, id, prop)))
    }
}

/// A dictionary mapping a subset of target type properties to the values.
/// Every dependency object can have an applied style at every moment.
/// To switch an applied style, use the [`DepObjId::apply_style`] function.
#[derive(Educe)]
#[educe(Debug, Clone, Default)]
pub struct Style<Owner: DepType> {
    setters: Vec<Box<dyn AnySetter<Owner>>>,
}

impl<Owner: DepType> Style<Owner> {
    pub fn new() -> Self { Style { setters: Vec::new() } }

    pub fn with_capacity(capacity: usize) -> Self { Style { setters: Vec::with_capacity(capacity) } }

    pub fn capacity(&self) -> usize { self.setters.capacity() }

    pub fn clear(&mut self) { self.setters.clear(); }

    pub fn contains_prop<PropType: Convenient>(&self, prop: DepProp<Owner, PropType>) -> bool {
        self.setters.binary_search_by_key(&prop.offset, |x| x.prop_offset()).is_ok()
    }

    pub fn insert<PropType: Convenient>(
        &mut self,
        prop: DepProp<Owner, PropType>,
        value: PropType
    ) -> bool where Owner: 'static, Owner::Id: DepObj<Owner> {
        let setter = Box::new(Setter { prop, value });
        match self.setters.binary_search_by_key(&prop.offset, |x| x.prop_offset()) {
            Ok(index) => { self.setters[index] = setter; true }
            Err(index) => { self.setters.insert(index, setter); false }
        }
    }

    pub fn is_empty(&self) -> bool { self.setters.is_empty() }

    pub fn len(&self) -> usize { self.setters.len() }

    pub fn remove<PropType: Convenient>(&mut self, prop: DepProp<Owner, PropType>) -> bool {
        match self.setters.binary_search_by_key(&prop.offset, |x| x.prop_offset()) {
            Ok(index) => { self.setters.remove(index); true }
            Err(_) => false
        }
    }

    pub fn reserve(&mut self, additional: usize) { self.setters.reserve(additional) }

    pub fn shrink_to(&mut self, min_capacity: usize) { self.setters.shrink_to(min_capacity) }

    pub fn shrink_to_fit(&mut self) { self.setters.shrink_to_fit() }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.setters.try_reserve(additional)
    }
}

pub trait DepObjBuilder {
    type Id: ComponentId;

    fn state(&self) -> &dyn State;
    fn state_mut(&mut self) -> &mut dyn State;
    fn id(&self) -> Self::Id;
}

#[derive(Educe)]
#[educe(Debug)]
struct DepEventHandledSource<Owner: DepType, ArgsType: DepEventArgs> {
    id: Owner::Id,
    handler_id: Id<BoxedHandler<ArgsType>>,
    event: DepEvent<Owner, ArgsType>,
}

impl<Owner: DepType, ArgsType: DepEventArgs> HandlerId for DepEventHandledSource<Owner, ArgsType> where
    Owner::Id: DepObj<Owner> {

    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry_mut = self.event.entry_mut(&mut obj);
        entry_mut.handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepEventSource<Owner: DepType, ArgsType: DepEventArgs> {
    id: Owner::Id,
    event: DepEvent<Owner, ArgsType>,
}

impl<Owner: DepType + 'static, ArgsType: DepEventArgs + 'static> Source for DepEventSource<Owner, ArgsType> where
    Owner::Id: DepObj<Owner> {

    type Value = ArgsType;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<ArgsType>>,
    ) -> HandledSource {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.event.entry_mut(&mut obj);
        let handler_id = entry.handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        HandledSource {
            handler_id: Box::new(DepEventHandledSource { handler_id, id: self.id, event: self.event }),
            init: None // TODO some events with cached value?
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledValueSource<Owner: DepType, PropType: Convenient> {
    id: Owner::Id,
    handler_id: Id<BoxedHandler<PropType>>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledValueSource<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry_mut = self.prop.entry_mut(&mut obj);
        entry_mut.handlers.value_handlers.remove(self.handler_id);
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.id);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledChangeInitialSource<Owner: DepType, PropType: Convenient> {
    id: Owner::Id,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledChangeInitialSource<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry_mut = self.prop.entry_mut(&mut obj);
        let handler = entry_mut.handlers.change_initial_handler.take();
        debug_assert!(handler.is_some());
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.id);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledChangeFinalSource<Owner: DepType, PropType: Convenient> {
    id: Owner::Id,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledChangeFinalSource<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry_mut = self.prop.entry_mut(&mut obj);
        let handler = entry_mut.handlers.change_final_handler.take();
        debug_assert!(handler.is_some());
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.id);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledChangeSource<Owner: DepType, PropType: Convenient> {
    id: Owner::Id,
    handler_id: Id<BoxedHandler<Change<PropType>>>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledChangeSource<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry_mut = self.prop.entry_mut(&mut obj);
        entry_mut.handlers.change_handlers.remove(self.handler_id);
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.id);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropValueSource<Owner: DepType, PropType: Convenient> {
    id: Owner::Id,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropValueSource<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    type Value = PropType;
    type Cache = ValueCache<PropType>;

    fn handle(&self, state: &mut dyn State, handler: Box<dyn Handler<PropType>>) -> HandledSource {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.prop.entry_mut(&mut obj);
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler_id = entry.handlers.value_handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.id);
        }
        let value = self.prop.current_value(state, self.id, |x| x.clone());
        let prop = self.prop;
        let id = self.id;
        let init = Box::new(move |state: &mut dyn State| {
            let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
            let entry = prop.entry(&obj);
            let handler = entry.handlers.value_handlers[handler_id].0.clone();
            handler.execute(state, value);
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledValueSource { handler_id, id: self.id, prop: self.prop }),
            init: Some(init)
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropChangeSource<Owner: DepType, PropType: Convenient> {
    id: Owner::Id,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropChangeSource<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    type Value = Change<PropType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<Change<PropType>>>,
    ) -> HandledSource {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.prop.entry_mut(&mut obj);
        let default_value = entry.default;
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler_id = entry.handlers.change_handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.id);
        }
        let change = self.prop.current_value(state, self.id, |value| {
            if value == default_value {
                None
            } else {
                Some(Change { old: default_value.clone(), new: value.clone() })
            }
        });
        let init = change.map(|change| {
            let prop = self.prop;
            let id = self.id;
            Box::new(move |state: &mut dyn State| {
                let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
                let entry = prop.entry(&obj);
                let handler = entry.handlers.change_handlers[handler_id].0.clone();
                handler.execute(state, change);
            }) as _
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledChangeSource { handler_id, id: self.id, prop: self.prop }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropChangeInitialSource<Owner: DepType, PropType: Convenient> {
    id: Owner::Id,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropChangeInitialSource<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    type Value = Change<PropType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<Change<PropType>>>,
    ) -> HandledSource {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.prop.entry_mut(&mut obj);
        let default_value = entry.default;
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler = entry.handlers.change_initial_handler.replace(handler);
        assert!(handler.is_none(), "duplicate initial handler");
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.id);
        }
        let change = self.prop.current_value(state, self.id, |value| {
            if value == default_value {
                None
            } else {
                Some(Change { old: default_value.clone(), new: value.clone() })
            }
        });
        let init = change.map(|change| {
            let prop = self.prop;
            let id = self.id;
            Box::new(move |state: &mut dyn State| {
                let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
                let entry = prop.entry(&obj);
                let handler = entry.handlers.change_initial_handler.clone().unwrap();
                handler.execute(state, change);
            }) as _
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledChangeInitialSource { id: self.id, prop: self.prop }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropChangeFinalSource<Owner: DepType, PropType: Convenient> {
    id: Owner::Id,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropChangeFinalSource<Owner, PropType> where
    Owner::Id: DepObj<Owner> {

    type Value = Change<PropType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<Change<PropType>>>,
    ) -> HandledSource {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.prop.entry_mut(&mut obj);
        let default_value = entry.default;
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler = entry.handlers.change_final_handler.replace(handler);
        assert!(handler.is_none(), "duplicate final handler");
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.id);
        }
        let change = self.prop.current_value(state, self.id, |value| {
            if value == default_value {
                None
            } else {
                Some(Change { old: default_value.clone(), new: value.clone() })
            }
        });
        let init = change.map(|change| {
            let prop = self.prop;
            let id = self.id;
            Box::new(move |state: &mut dyn State| {
                let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
                let entry = prop.entry(&obj);
                let handler = entry.handlers.change_final_handler.clone().unwrap();
                handler.execute(state, change);
            }) as _
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledChangeFinalSource { id: self.id, prop: self.prop }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecChangedHandledSource<Owner: DepType, ItemType: Convenient> {
    id: Owner::Id,
    handler_id: Id<BoxedHandler<()>>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecChangedHandledSource<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry_mut = self.vec.entry_mut(&mut obj);
        entry_mut.handlers.changed_handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecItemHandledInitialFinalSource<Owner: DepType, ItemType: Convenient> {
    id: Owner::Id,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecItemHandledInitialFinalSource<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    fn unhandle(&self, state: &mut dyn State, dropping_binding: AnyBindingBase) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry_mut = self.vec.entry_mut(&mut obj);
        let handler = entry_mut.handlers.item_initial_final_handler.take().unwrap();
        handler.update.filter(|&x| {
            let x: AnyBindingBase = x.into();
            x != dropping_binding
        }).map(|binding| binding.drop_self(state));
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecItemHandledSource<Owner: DepType, ItemType: Convenient> {
    id: Owner::Id,
    handler_id: Id<ItemHandler<ItemType>>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecItemHandledSource<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    fn unhandle(&self, state: &mut dyn State, dropping_binding: AnyBindingBase) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry_mut = self.vec.entry_mut(&mut obj);
        let handler = entry_mut.handlers.item_handlers.remove(self.handler_id);
        handler.update.filter(|&x| {
            let x: AnyBindingBase = x.into();
            x != dropping_binding
        }).map(|binding| binding.drop_self(state));
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecChangedSource<Owner: DepType, ItemType: Convenient> {
    id: Owner::Id,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> Source for DepVecChangedSource<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    type Value = ();
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<()>>,
    ) -> HandledSource {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.vec.entry_mut(&mut obj);
        let changed = !entry.items.is_empty();
        let handler_id = entry.handlers.changed_handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        let init = if changed {
            let vec = self.vec;
            let id = self.id;
            Some(Box::new(move |state: &mut dyn State| {
                let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
                let entry = vec.entry(&obj);
                let handler = entry.handlers.changed_handlers[handler_id].0.clone();
                handler.execute(state, ());
            }) as _)
        } else {
            None
        };
        HandledSource {
            handler_id: Box::new(DepVecChangedHandledSource { handler_id, id: self.id, vec: self.vec }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DepVecItemInitialFinalSourceUpdate<Owner: DepType, ItemType: Convenient> {
    id: Owner::Id,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> Target<()> for DepVecItemInitialFinalSourceUpdate<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    fn execute(&self, state: &mut dyn State, (): ()) {
        self.vec.modify(state, self.id, DepVecModification::Update(None));
    }
}

impl<Owner: DepType, ItemType: Convenient> Holder for DepVecItemInitialFinalSourceUpdate<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    fn release(&self, state: &mut dyn State) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.vec.entry_mut(&mut obj);
        let ok = entry.handlers.item_initial_final_handler.as_mut().unwrap().update.take().is_some();
        debug_assert!(ok);
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DepVecItemSourceUpdate<Owner: DepType, ItemType: Convenient> {
    id: Owner::Id,
    vec: DepVec<Owner, ItemType>,
    handler_id: Id<ItemHandler<ItemType>>,
}

impl<Owner: DepType, ItemType: Convenient> Target<()> for DepVecItemSourceUpdate<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    fn execute(&self, state: &mut dyn State, (): ()) {
        self.vec.modify(state, self.id, DepVecModification::Update(Some(self.handler_id)));
    }
}

impl<Owner: DepType, ItemType: Convenient> Holder for DepVecItemSourceUpdate<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    fn release(&self, state: &mut dyn State) {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.vec.entry_mut(&mut obj);
        let ok = entry.handlers.item_handlers[self.handler_id].update.take().is_some();
        debug_assert!(ok);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecItemSource<Owner: DepType, ItemType: Convenient> {
    id: Owner::Id,
    vec: DepVec<Owner, ItemType>,
    update: Option<BindingBase<()>>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> Source for DepVecItemSource<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    type Value = ItemChange<ItemType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<ItemChange<ItemType>>>,
    ) -> HandledSource {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.vec.entry_mut(&mut obj);
        let items = entry.items.clone();
        let handler_id = entry.handlers.item_handlers.insert(
            |handler_id| (ItemHandler { handler, update: self.update }, handler_id)
        );
        if let Some(update) = self.update {
            update.set_target(state, Box::new(DepVecItemSourceUpdate { id: self.id, vec: self.vec, handler_id }));
            update.set_holder(state, Box::new(DepVecItemSourceUpdate { id: self.id, vec: self.vec, handler_id }));
        }
        let init = if items.is_empty() {
            None
        } else {
            let vec = self.vec;
            let id = self.id;
            Some(Box::new(move |state: &mut dyn State| {
                let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
                let entry = vec.entry(&obj);
                let handler = entry.handlers.item_handlers[handler_id].handler.clone();
                for (item, prev) in items.iter().zip(once(None).chain(items.iter().map(Some))) {
                    handler.execute(state, ItemChange {
                        action: ItemChangeAction::Insert { prev: prev.cloned() },
                        item: item.clone()
                    });
                }
            }) as _)
        };
        HandledSource {
            handler_id: Box::new(DepVecItemHandledSource { handler_id, id: self.id, vec: self.vec }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecItemInitialFinalSource<Owner: DepType, ItemType: Convenient> {
    id: Owner::Id,
    vec: DepVec<Owner, ItemType>,
    update: Option<BindingBase<()>>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> Source for DepVecItemInitialFinalSource<Owner, ItemType> where
    Owner::Id: DepObj<Owner> {

    type Value = ItemChange<ItemType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<ItemChange<ItemType>>>,
    ) -> HandledSource {
        let mut obj = <Owner::Id as DepObj<Owner>>::get_mut(state, self.id.into_raw());
        let entry = self.vec.entry_mut(&mut obj);
        let items = entry.items.clone();
        let handler = ItemHandler { handler, update: self.update };
        assert!(entry.handlers.item_initial_final_handler.replace(handler).is_none(), "duplicate initial handler");
        if let Some(update) = self.update {
            update.set_target(state, Box::new(DepVecItemInitialFinalSourceUpdate { id: self.id, vec: self.vec }));
            update.set_holder(state, Box::new(DepVecItemInitialFinalSourceUpdate { id: self.id, vec: self.vec }));
        }
        let init = if items.is_empty() {
            None
        } else {
            let vec = self.vec;
            let id = self.id;
            Some(Box::new(move |state: &mut dyn State| {
                let obj = <Owner::Id as DepObj<Owner>>::get(state, id.into_raw());
                let entry = vec.entry(&obj);
                let handler = entry.handlers.item_initial_final_handler.as_ref().unwrap().handler.clone();
                for (item, prev) in items.iter().zip(once(None).chain(items.iter().map(Some))) {
                    handler.execute(state, ItemChange {
                        action: ItemChangeAction::Insert { prev: prev.cloned() },
                        item: item.clone()
                    });
                }
            }) as _)
        };
        HandledSource {
            handler_id: Box::new(DepVecItemHandledInitialFinalSource { id: self.id, vec: self.vec }),
            init
        }
    }
}

pub struct DepObjRef<'a, Obj> {
    state_part: &'a dyn Any,
    id: RawId,
    get_raw: fn(state_part: &dyn Any, id: RawId) -> &Obj,
}

impl<'a, Obj> Deref for DepObjRef<'a, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        (self.get_raw)(self.state_part, self.id)
    }
}

pub struct DepObjMut<'a, Obj> {
    state_part: &'a mut dyn Any,
    id: RawId,
    get_raw: fn(state_part: &dyn Any, id: RawId) -> &Obj,
    get_raw_mut: fn(state_part: &mut dyn Any, id: RawId) -> &mut Obj,
}

impl<'a, Obj> Deref for DepObjMut<'a, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        (self.get_raw)(self.state_part, self.id)
    }
}

impl<'a, Obj> DerefMut for DepObjMut<'a, Obj> {
    fn deref_mut(&mut self) -> &mut Obj {
        (self.get_raw_mut)(self.state_part, self.id)
    }
}

pub trait DepObj<Type: DepType> {
    const STATE_PART: TypeId;

    fn get_raw(state_part: &dyn Any, id: RawId) -> &Type;
    fn get_raw_mut(state_part: &mut dyn Any, id: RawId) -> &mut Type;

    fn get(state: &dyn State, id: RawId) -> DepObjRef<Type> {
        DepObjRef {
            id,
            state_part: state.get_raw(Self::STATE_PART).unwrap_or_else(|| panic!("{:?} required", Self::STATE_PART)),
            get_raw: Self::get_raw,
        }
    }

    fn get_mut(state: &mut dyn State, id: RawId) -> DepObjMut<Type> {
        DepObjMut {
            id,
            state_part: state.get_mut_raw(Self::STATE_PART).unwrap_or_else(|| panic!("{:?} required", Self::STATE_PART)),
            get_raw: Self::get_raw,
            get_raw_mut: Self::get_raw_mut,
        }
    }
}

pub struct Builder<'a, T: ComponentId> {
    pub id: T,
    pub state: &'a mut dyn State,
}

impl<'a, T: ComponentId> DepObjBuilder for Builder<'a, T> {
    type Id = T;

    fn id(&self) -> T { self.id }
    fn state(&self) -> &dyn State { self.state }
    fn state_mut(&mut self) -> &mut dyn State { self.state }
}

#[doc(hidden)]
#[macro_export]
macro_rules! unexpected_token {
    () => { };
}

#[macro_export]
macro_rules! ext_builder {
    (
        $($token:tt)+
    ) => {
        $crate::generics_parse! {
            $crate::ext_builder_impl {
                @generics
            }
            $($token)+
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! ext_builder_impl {
    (
        @generics
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $base_builder:ty as $ext:ident [$Id:ty] {
            fn $fn_name:ident (
                $state:ident $(: $State:ty)?,
                $id:ident $(: $Id_:ty)?
            ) -> ($builder:ident $(in $($builder_path:tt)+)?) {
                $($e:tt)*
            }
        }
    ) => {
        $crate::generics_concat! {
            $crate::ext_builder_impl {
                @impl
                [$($g)*] [$($r)*] [$($w)*]
                [$base_builder] [$ext] [$Id] [$fn_name]
                [$builder] [$($($builder_path)+)?]
                [$state $($State)?] [$id $($Id_)?] [$($e)*]
            }
            [$($g)*] [$($r)*] [$($w)*],
            [] [] [where Self: Sized]
        }
    };
    (
        @generics
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $base_builder:ty as $ext:ident [$Id:ty] {
            $fn_name:ident -> ($builder:ident $(in $($builder_path:tt)+)?)
        }
    ) => {
        $crate::generics_concat! {
            $crate::ext_builder_impl {
                @impl
                [$($g)*] [$($r)*] [$($w)*]
                [$base_builder] [$ext] [$Id] [$fn_name]
                [$builder] [$($($builder_path)+)?]
                [_state] [_id] []
            }
            [$($g)*] [$($r)*] [$($w)*],
            [] [] [where Self: Sized]
        }
    };
    (
        @generics
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("invalid builder extension");
    };
    (
        @impl
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$base_builder:ty] [$ext:ident] [$Id:ty] [$fn_name:ident]
        [$builder:ident] [$($($builder_path:tt)+)?]
        [$state:ident $($State:ty)?] [$id:ident $($Id_:ty)?] [$($e:tt)*]
        [$($tr_g:tt)*] [$($tr_r:tt)*] [$($tr_w:tt)*]
    ) => {
        $crate::paste_paste! {
            pub trait $ext $($tr_g)* : $crate::DepObjBuilder<Id=$Id> $($tr_w)* {
                fn $fn_name (
                    self,
                    f: impl FnOnce(
                        $($($builder_path)+ ::)? [< $builder Builder >] < Self >
                    ) -> $($($builder_path)+ ::)? [< $builder Builder >] < Self >
                ) -> Self;
            }

            impl $($g)* $ext $($r)* for $base_builder $($w)* {
                fn $fn_name (
                    mut self,
                    f: impl  FnOnce(
                        $($($builder_path)+ ::)? [< $builder Builder >] < Self >
                    ) -> $($($builder_path)+ ::)? [< $builder Builder >] < Self >
                ) -> Self {
                    {
                        let $id $(: $Id_)? = <Self as $crate::DepObjBuilder>::id(&self);
                        let $state $(: $State)? = <Self as $crate::DepObjBuilder>::state_mut(&mut self);
                        $($e)*
                    }
                    f(<$($($builder_path)+ ::)? [< $builder Builder >] <Self> >::new_priv(self)).base_priv()
                }
            }
        }
    };
}

#[macro_export]
macro_rules! with_builder {
    (
    ) => {
        pub fn build(
            self,
            state: &mut dyn $crate::dyn_context_State,
            f: impl for<'builder_lt> FnOnce(
                $crate::Builder<'builder_lt, Self>
            ) -> $crate::Builder<'builder_lt, Self>
        ) -> Self {
            f($crate::Builder { state, id: self });
            self
        }
    };
    (
        $builder:ident $(in $($builder_path:tt)+)?
    ) => {
        $crate::paste_paste! {
            pub fn build(
                self,
                state: &mut dyn $crate::dyn_context_State,
                f: impl for<'builder_lt> FnOnce(
                    $($($builder_path)+ ::)? [< $builder Builder >] < $crate::Builder <'builder_lt, Self> >
                ) -> $($($builder_path)+ ::)? [< $builder Builder >] < $crate::Builder <'builder_lt, Self> >
            ) -> Self {
                let base_builder = $crate::Builder { state, id: self };
                f(
                    <
                        $($($builder_path)+ ::)? [< $builder Builder >] < $crate::Builder <'_, Self> >
                    >::new_priv(base_builder)
                );
                self
            }
        }
    };
}

/// Defines dependency type.
///
/// Accepts input in the following form:
///
/// ```ignore
/// $(#[$attr:meta])* $vis:vis struct $name:ident
/// $(
///     [$Id:ty]
/// |
///     <$generics> [$Id:ty] $(where $where_clause)?
/// )
/// {
///     $($(
///         $(#[$field_attr:meta])* $field_name:ident
///         $(
///             : $field_type:ty = $field_value:expr
///         |
///             [$vec_field_item_type:ty]
///         |
///             yield $event_field_type:ty
///         )
///     ),+ $(,)?)?
/// }
/// ```
#[macro_export]
macro_rules! dep_type {
    (
        $($token:tt)*
    ) => {
        $crate::dep_type_impl! { $($token)* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_type_impl {
    (
        $(#[$attr:meta])* $vis:vis struct $name:ident $($body:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_impl {
                @struct
                [Base]
                [$([$attr])*] [$vis] [$name]
            }
            $($body)*
        }
    };
    (
        @struct
        [$BaseBuilder:ident]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$Id:ty]
        {
            $($($(#[$inherits:tt])* $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
    ) => {
        $crate::dep_type_impl! {
            @concat_generics
            [$([$attr])*] [$vis] [$name] [id] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [$BaseBuilder]
            [$($([[$($inherits)*] $field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @struct
        [$BaseBuilder:ident]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$Id:ty]
        {
            $($($(#[$inherits:tt])* $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
        $token:tt $($tail:tt)*
    ) => {
        $crate::unexpected_token!($token);
        $crate::std_compile_error!("unexpected extra tokens after dep type definition");
    };
    (
        @struct
        [$BaseBuilder:ident]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!($crate::indoc_indoc!("
            invalid dep type definition, allowed form is

            $(#[$attr:meta])* $vis:vis struct $name:ident
            $(
                [$Id:ty]
            |
                <$generics> [$Id:ty] $(where $where_clause)?
            )
            {
                $($(
                    $(#[$field_attr:meta])* $field_name:ident
                    $(
                        : $field_type:ty = $field_value:expr
                    |
                        [$vec_field_item_type:ty]
                    |
                        yield $event_field_type:ty
                    )
                ),+ $(,)?)?
            }

        "));
    };
    (
        @concat_generics
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$BaseBuilder:ident]
        [$([[$($inherits:tt)*] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
    ) => {
        $crate::generics_concat! {
            $crate::dep_type_impl {
                @concat_generics_done
                [$BaseBuilder]
                [$([$attr])*] [$vis] [$name] [$id] [$Id]
                [$($g)*] [$($r)*] [$($w)*]
                [$([[$($inherits)*] $field $delim $($field_ty $(= $field_val)?)?])*]
            }
            [ < $BaseBuilder : $crate::DepObjBuilder <Id= $Id > > ] [ < $BaseBuilder > ] [],
            [$($g)*] [$($r)*] [$($w)*]
        }
    };
    (
        @concat_generics_done
        [$BaseBuilder:ident]
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$([[$($inherits:tt)*] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
    ) => {
        $crate::dep_type_impl! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$id] [$Id] [state] [this] [bindings] [handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] [] [] [] []
            [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*] []
            [$([[$($inherits)*] $field $delim $($field_ty $(= $field_val)?)?])*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[ref inherits] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$id] [$Id]
            [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepPropEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepPropEntry::new(&Self:: [< $field:upper _DEFAULT >] , true),
            ]
            [
                $($core_consts)*
                const [< $field:upper _DEFAULT >] : $field_ty = $field_val;
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepProp<Self, $field_ty> = {
                    unsafe {
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepProp::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
                $this . $field .binding().map(|x| $bindings.push(
                    <$crate::binding::AnyBindingBase as $crate::std_convert_From<$crate::binding::BindingBase<$field_ty>>>::from(x)
                ));
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
                $name:: [< $field:upper >] .update_parent_children_has_handlers($state, $id);
            ]
            [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
            [
                $($builder_methods)*

                $vis fn [< $field _ref >] (mut self, value: $field_ty) -> Self {
                    let id = <$BaseBuilder as $crate::DepObjBuilder>::id(&self.base);
                    let state = <$BaseBuilder as $crate::DepObjBuilder>::state_mut(&mut self.base);
                    $name:: [< $field:upper >] .set(state, id, value).immediate();
                    self
                }
            ]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[inherits ref] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$id] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepPropEntry<$field_ty>,
            ]
                [
                $($core_new)*
                $field: $crate::DepPropEntry::new(&Self:: [< $field:upper _DEFAULT >] , true),
            ]
            [
                $($core_consts)*
                const [< $field:upper _DEFAULT >] : $field_ty = $field_val;
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepProp<Self, $field_ty> = {
                    unsafe {
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepProp::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
                $this . $field .binding().map(|x| $bindings.push(
                    <$crate::binding::AnyBindingBase as $crate::std_convert_From<$crate::binding::BindingBase<$field_ty>>>::from(x)
                ));
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
                $name:: [< $field:upper >] .update_parent_children_has_handlers($state, $id);
            ]
            [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
            [
                $($builder_methods)*

                $vis fn [< $field _ref >] (mut self, value: $field_ty) -> Self {
                    let id = <$BaseBuilder as $crate::DepObjBuilder>::id(&self.base);
                    let state = <$BaseBuilder as $crate::DepObjBuilder>::state_mut(&mut self.base);
                    $name:: [< $field:upper >] .set(state, id, value).immediate();
                    self
                }
            ]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[inherits] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$id] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepPropEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepPropEntry::new(&Self:: [< $field:upper _DEFAULT >] , true),
            ]
            [
                $($core_consts)*
                const [< $field:upper _DEFAULT >] : $field_ty = $field_val;
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepProp<Self, $field_ty> = {
                    unsafe {
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepProp::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
                $this . $field .binding().map(|x| $bindings.push(
                    <$crate::binding::AnyBindingBase as $crate::std_convert_From<$crate::binding::BindingBase<$field_ty>>>::from(x)
                ));
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
                $name:: [< $field:upper >] .update_parent_children_has_handlers($state, $id);
            ]
            [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
            [
                $($builder_methods)*

                $vis fn $field(mut self, value: $field_ty) -> Self {
                    let id = <$BaseBuilder as $crate::DepObjBuilder>::id(&self.base);
                    let state = <$BaseBuilder as $crate::DepObjBuilder>::state_mut(&mut self.base);
                    $name:: [< $field:upper >] .set(state, id, value).immediate();
                    self
                }
            ]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[ref] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$id] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepPropEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepPropEntry::new(&Self:: [< $field:upper _DEFAULT >] , false),
            ]
            [
                $($core_consts)*
                const [< $field:upper _DEFAULT >] : $field_ty = $field_val;
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepProp<Self, $field_ty> = {
                    unsafe {
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepProp::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
                $this . $field .binding().map(|x| $bindings.push(
                    <$crate::binding::AnyBindingBase as $crate::std_convert_From<$crate::binding::BindingBase<$field_ty>>>::from(x)
                ));
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
            ]
            [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
            [
                $($builder_methods)*

                $vis fn [< $field _ref >] (mut self, value: $field_ty) -> Self {
                    let id = <$BaseBuilder as $crate::DepObjBuilder>::id(&self.base);
                    let state = <$BaseBuilder as $crate::DepObjBuilder>::state_mut(&mut self.base);
                    $name:: [< $field:upper >] .set(state, id, value).immediate();
                    self
                }
            ]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$id] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepPropEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepPropEntry::new(&Self:: [< $field:upper _DEFAULT >] , false),
            ]
            [
                $($core_consts)*
                const [< $field:upper _DEFAULT >] : $field_ty = $field_val;
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepProp<Self, $field_ty> = {
                    unsafe {
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepProp::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
                $this . $field .binding().map(|x| $bindings.push(
                    <$crate::binding::AnyBindingBase as $crate::std_convert_From<$crate::binding::BindingBase<$field_ty>>>::from(x)
                ));
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
            ]
            [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
            [
                $($builder_methods)*

                $vis fn $field(mut self, value: $field_ty) -> Self {
                    let id = <$BaseBuilder as $crate::DepObjBuilder>::id(&self.base);
                    let state = <$BaseBuilder as $crate::DepObjBuilder>::state_mut(&mut self.base);
                    $name:: [< $field:upper >] .set(state, id, value).immediate();
                    self
                }
            ]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[$($inherits:tt)*] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dep type property attributes: '",
            $crate::std_stringify!($(#[$inherits])*),
            "'; allowed attributes are: '#[inherits]', '#[ref]'"
        ));
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[bubble] $field:ident yield $field_ty:ty] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$id] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepEventEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepEventEntry::new(true),
            ]
            [
                $($core_consts)*
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepEvent<Self, $field_ty> = {
                    unsafe {
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepEvent::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
            ]
            [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
            [
                $($builder_methods)*
            ]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[] $field:ident yield $field_ty:ty] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$id] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepEventEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepEventEntry::new(false),
            ]
            [
                $($core_consts)*
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepEvent<Self, $field_ty> = {
                    unsafe {
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepEvent::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
            ]
            [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
            [
                $($builder_methods)*
            ]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[$($inherits:tt)*] $field:ident yield $field_ty:ty] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dep type event attributes: '",
            $crate::std_stringify!($(#[$inherits])*),
            "'; allowed attributes are: '#[bubble]'"
        ));
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[] $field:ident [$field_ty:ty]] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$id] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepVecEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepVecEntry::new(),
            ]
            [
                $($core_consts)*
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepVec<Self, $field_ty> = {
                    unsafe {
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepVec::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
                $this . $field .collect_all_bindings(&mut $bindings);
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
            ]
            [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
            [
                $($builder_methods)*
            ]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[$($inherits:tt)*] $field:ident [$field_ty:ty]] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "unexpected dep type vector property attributes: '",
            $crate::std_stringify!($(#[$inherits])*),
            "'"
        ));
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        [[[$($inherits:tt)*] $field:ident $delim:tt $field_ty:ty $(= $field_val:expr)?] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dep type field definition\n\n",
            $crate::std_stringify!($(#[$inherits])? $field $delim $field_ty $(= $field_val)?),
            "\n\n",
            $crate::indoc_indoc!("
                allowed forms are

                $(#[$field_attr:meta])* $field_name:ident : $field_type:ty = $field_value:expr

                $(#[$field_attr:meta])* $field_name:ident [$vec_field_item_type:ty]

                $(#[$field_attr:meta])* $field_name:ident yield $event_field_type:ty
            ")
        ));
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$id:ident] [$Id:ty]
        [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$($update_handlers:tt)*]
        [$BaseBuilder:ident] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        [$($builder_methods:tt)*]
        []
    ) => {
        $crate::paste_paste! {
            #[derive($crate::std_fmt_Debug)]
            struct [< $name Core >] $($g)* $($w)* {
                dep_type_core_base: $crate::BaseDepObjCore<$name $($r)*>,
                $($core_fields)*
            }

            impl $($g)* [< $name Core >] $($r)* $($w)* {
                const fn new() -> Self {
                    Self {
                        dep_type_core_base: $crate::BaseDepObjCore::new(),
                        $($core_new)*
                    }
                }

                $($core_consts)*

                fn dep_type_core_take_all_handlers(&mut self) -> $crate::std_vec_Vec<$crate::std_boxed_Box<dyn $crate::binding::AnyHandler>> {
                    let mut $handlers = $crate::std_vec_Vec::new();
                    let $this = self;
                    $($core_handlers)*
                    $handlers
                }

                fn dep_type_core_collect_all_bindings(&self) -> $crate::std_vec_Vec<$crate::binding::AnyBindingBase> {
                    let mut $bindings = self.dep_type_core_base.collect_bindings();
                    let $this = self;
                    $($core_bindings)*
                    $bindings
                }
            }

            $( #[ $attr ] )*
            $vis struct $name $($g)* $($w)* {
                core: [< $name Core >] $($r)*
            }

            impl $($g)* $name $($r)* $($w)* {
                const fn new_priv() -> Self {
                    Self { core: [< $name Core >] ::new() }
                }

                $($dep_props)*
            }

            impl $($g)* $crate::DepType for $name $($r)* $($w)* {
                type Id = $Id;

                fn core_base_priv(&self) -> &$crate::BaseDepObjCore<$name $($r)*> {
                    &self.core.dep_type_core_base
                }

                fn core_base_priv_mut(&mut self) -> &mut $crate::BaseDepObjCore<$name $($r)*> {
                    &mut self.core.dep_type_core_base
                }

                fn take_all_handlers(&mut self) -> $crate::std_vec_Vec<$crate::std_boxed_Box<dyn $crate::binding::AnyHandler>> {
                    self.core.dep_type_core_take_all_handlers()
                }

                fn collect_all_bindings(&self) -> $crate::std_vec_Vec<$crate::binding::AnyBindingBase> {
                    self.core.dep_type_core_collect_all_bindings()
                }

                #[allow(unused_variables)]
                fn update_parent_children_has_handlers($state: &mut dyn $crate::dyn_context_State, $id: $crate::components_arena_RawId) where Self: Sized {
                    $($update_handlers)*
                }
            }

            $vis struct [< $name Builder >] $($bc_g)* $($bc_w)* {
                base: $BaseBuilder,
            }

            impl $($bc_g)* $crate::DepObjBuilder for [< $name Builder >] $($bc_r)* $($bc_w)* {
                type Id = $Id;

                fn state(&self) -> &dyn $crate::dyn_context_State {
                    < $BaseBuilder as $crate::DepObjBuilder >::state(&self.base)
                }

                fn state_mut(&mut self) -> &mut dyn $crate::dyn_context_State {
                    < $BaseBuilder as $crate::DepObjBuilder >::state_mut(&mut self.base)
                }

                fn id(&self) -> $Id {
                    < $BaseBuilder as $crate::DepObjBuilder >::id(&self.base)
                }
            }

            impl $($bc_g)* [< $name Builder >] $($bc_r)* $($bc_w)* {
                fn new_priv(base: $BaseBuilder) -> Self {
                    Self { base }
                }

                #[allow(dead_code)]
                fn base_priv(self) -> $BaseBuilder { self.base }

                #[allow(dead_code)]
                fn base_priv_ref(&self) -> &$BaseBuilder { &self.base }

                #[allow(dead_code)]
                fn base_priv_mut(&mut self) -> &mut $BaseBuilder { &mut self.base }

                $($builder_methods)*
            }
        }
    };
    (
        $($token:tt)*
    ) => {
        $crate::std_compile_error!($crate::indoc_indoc!("
            invalid dep_type macro input, allowed form is

            $(#[$attr:meta])* $vis:vis struct $name:ident
            $(
                [$Id:ty]
            |
                <$generics> [$Id:ty] $(where $where_clause)?
            )
            {
                $($(
                    $(#[$field_attr:meta])* $field_name:ident
                    $(
                        : $field_type:ty = $field_value:expr
                    |
                        [$vec_field_item_type:ty]
                    |
                        yield $event_field_type:ty
                    )
                ),+ $(,)?)?
            }
        "));
    };
}

/// Specifies dependency objects list and accessing methods.
///
/// Accepts input in the following form:
///
/// ```ignore
/// $(
///     impl $Id:ty
/// |
///     impl <$generics> $Id:ty $(where $where_clause)?
/// )
/// {
///     $(
///         fn(
///             self as $this:ident,
///             $state_part:ident : $StatePart:ty
///         ) -> $(optional)? $(dyn ($tr:path) | ($ty:ty)) {
///             if mut { $field_mut:expr } else { $field:expr }
///         }
///     )*
/// }
/// ```
#[macro_export]
macro_rules! dep_obj {
    (
        impl $($token:tt)+
    ) => {
        $crate::generics_parse! {
            $crate::dep_obj_impl {
                @generics_parsed
            }
            $($token)+
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_obj_impl {
    (
        @generics_parsed
        $g:tt $r:tt $w:tt $Id:ty {
            $(
                fn(
                    self as $this:ident,
                    $state_part:ident : $StatePart:ty
                ) -> $(optional dyn($opt_tr:path))? $(dyn ($tr:path))? $(optional($opt_ty:ty))? $(($ty:ty))? {
                    if mut { $field_mut:expr } else { $field:expr }
                }
            )*
        }
    ) => {
        $crate::dep_obj_impl! {
            @drop_bindings
            $g $r $w [$Id]
            [
                $(
                    [$this] [$state_part] [$StatePart] [$($opt_tr)?] [$($tr)?] [$($opt_ty)?] [$($ty)?]
                    [$field_mut] [$field]
                )*
            ]
        }
        $(
            $crate::dep_obj_impl! {
                @impl
                $g $w [$Id]
                [$this] [$state_part] [$StatePart] [$($opt_tr)?] [$($tr)?] [$($opt_ty)?] [$($ty)?]
                [$field_mut] [$field]
            }
        )*
    };
    (
        @generics_parsed
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] $($body:tt)*
    ) => {
        $crate::std_compile_error!($crate::indoc_indoc!("
            invalid dep obj implementation, allowed form is

            $(
                impl $Id:ty
            |
                impl <$generics> $Id:ty $(where $where_clause)?
            )
            {
                $(
                    fn(
                        self as $this:ident,
                        $state_part:ident : $StatePart:ty
                    ) -> $(optional)? $(dyn ($tr:path) | ($ty:ty)) {
                        if mut { $field_mut:expr } else { $field:expr }
                    }
                )*
            }

        "));
    };
    (
        @impl
        [$($g:tt)*] [$($w:tt)*] [$Id:ty]
        [$this:ident] [$state_part:ident] [$StatePart:ty] [$opt_tr:path] [] [] []
        [$field_mut:expr] [$field:expr]
    ) => {
        $crate::dep_obj_impl! {
            @add_parameter_to_optional
            [DepObjType]
            [$($g)*] [$($w)*] [$Id]
            [$this] [$state_part] [$StatePart] [$opt_tr]
            [$field_mut] [$field]
        }
    };
    (
        @add_parameter_to_optional
        [$p:ident]
        [$($g:tt)*] [$($w:tt)*] [$Id:ty]
        [$this:ident] [$state_part:ident] [$StatePart:ty] [$opt_tr:path]
        [$field_mut:expr] [$field:expr]
    ) => {
        $crate::generics_concat! {
            $crate::dep_obj_impl {
                @impl_optional_with_parameter
                [$p] [$($g)*] [$($w)*] [$Id]
                [$this] [$state_part] [$StatePart]
                [$field_mut] [$field]
            }
            [$($g)*] [] [],
            [ < $p : $opt_tr + $crate::DepType<Id=Self> > ] [] []
        }
    };
    (
        @impl_optional_with_parameter
        [$p:ident] [$($g:tt)*] [$($w:tt)*] [$Id:ty]
        [$this:ident] [$state_part:ident] [$StatePart:ty]
        [$field_mut:expr] [$field:expr]
        [$($gp:tt)*] [] []
    ) => {
        $crate::paste_paste! {
            impl $($gp)* $crate::DepObj <$p> for $Id $($w)* {
                const STATE_PART: $crate::std_any_TypeId = $crate::std_any_TypeId::of::<$StatePart>();

                fn get_raw <'state_part_lifetime>(
                    $state_part: &'state_part_lifetime dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'state_part_lifetime $p {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $state_part = $state_part.downcast_ref::<$StatePart>().expect("invalid state part cast");
                    ($field)
                        .expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                        .downcast_ref::<DepObjType>().expect("invalid cast")
                }

                fn get_raw_mut <'state_part_lifetime>(
                    $state_part: &'state_part_lifetime mut dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'state_part_lifetime mut $p {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $state_part = $state_part.downcast_mut::<$StatePart>().expect("invalid state part cast");
                    ($field_mut)
                        .expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                        .downcast_mut::<DepObjType>().expect("invalid cast")
                }

            }
        }
    };
    (
        @impl
        [$($g:tt)*] [$($w:tt)*] [$Id:ty]
        [$this:ident] [$state_part:ident] [$StatePart:ty] [] [$tr:path] [] []
        [$field_mut:expr] [$field:expr]
    ) => {
        $crate::dep_obj_impl! {
            @add_parameter
            [DepObjType]
            [$($g)*] [$($w)*] [$Id]
            [$this] [$state_part] [$StatePart] [$tr]
            [$field_mut] [$field]
        }
    };
    (
        @add_parameter
        [$p:ident]
        [$($g:tt)*] [$($w:tt)*] [$Id:ty]
        [$this:ident] [$state_part:ident] [$StatePart:ty] [$tr:path]
        [$field_mut:expr] [$field:expr]
    ) => {
        $crate::generics_concat! {
            $crate::dep_obj_impl {
                @impl_with_parameter
                [$p] [$($g)*] [$($w)*] [$Id]
                [$this] [$state_part] [$StatePart]
                [$field_mut] [$field]
            }
            [$($g)*] [] [],
            [ < $p : $tr + $crate::DepType<Id=Self> > ] [] []
        }
    };
    (
        @impl_with_parameter
        [$p:ident] [$($g:tt)*] [$($w:tt)*] [$Id:ty]
        [$this:ident] [$state_part:ident] [$StatePart:ty]
        [$field_mut:expr] [$field:expr]
        [$($gp:tt)*] [] []
    ) => {
        $crate::paste_paste! {
            impl $($gp)* $crate::DepObj<$p> for $Id $($w)* {
                const STATE_PART: $crate::std_any_TypeId = $crate::std_any_TypeId::of::<$StatePart>();

                fn get_raw <'state_part_lifetime>(
                    $state_part: &'state_part_lifetime dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'state_part_lifetime $p {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $state_part = $state_part.downcast_ref::<$StatePart>().expect("invalid state part cast");
                    ($field).downcast_ref::<DepObjType>().expect("invalid cast")
                }

                fn get_raw_mut <'state_part_lifetime>(
                    $state_part: &'state_part_lifetime mut dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'state_part_lifetime mut $p {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $state_part = $state_part.downcast_mut::<$StatePart>().expect("invalid state part cast");
                    ($field_mut).downcast_mut::<DepObjType>().expect("invalid cast")
                }
            }
        }
    };
    (
        @impl
        [$($g:tt)*] [$($w:tt)*] [$Id:ty]
        [$this:ident] [$state_part:ident] [$StatePart:ty] [] [] [$opt_ty:ty] []
        [$field_mut:expr] [$field:expr]
    ) => {
        $crate::paste_paste! {
            impl $($g)* $crate::DepObj<$opt_ty> for $Id $($w)* {
                const STATE_PART: $crate::std_any_TypeId = $crate::std_any_TypeId::of::<$StatePart>();

                fn get_raw <'state_part_lifetime>(
                    $state_part: &'state_part_lifetime dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'state_part_lifetime $opt_ty {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $state_part = $state_part.downcast_ref::<$StatePart>().expect("invalid state part cast");
                    ($field).expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                }

                fn get_raw_mut <'state_part_lifetime>(
                    $state_part: &'state_part_lifetime mut dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'state_part_lifetime mut $opt_ty {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $state_part = $state_part.downcast_mut::<$StatePart>().expect("invalid state part cast");
                    ($field_mut).expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                }
            }
        }
    };
    (
        @impl
        [$($g:tt)*] [$($w:tt)*] [$Id:ty]
        [$this:ident] [$state_part:ident] [$StatePart:ty] [] [] [] [$ty:ty]
        [$field_mut:expr] [$field:expr]
    ) => {
        $crate::paste_paste! {
            impl $($g)* $crate::DepObj<$ty> for $Id $($w)* {
                const STATE_PART: $crate::std_any_TypeId = $crate::std_any_TypeId::of::<$StatePart>();

                fn get_raw <'state_part_lifetime>(
                    $state_part: &'state_part_lifetime dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'state_part_lifetime $ty {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $state_part = $state_part.downcast_ref::<$StatePart>().expect("invalid state part cast");
                    $field
                }

                fn get_raw_mut <'state_part_lifetime>(
                    $state_part: &'state_part_lifetime mut dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'state_part_lifetime mut $ty {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $state_part = $state_part.downcast_mut::<$StatePart>().expect("invalid state part cast");
                    $field_mut
                }
            }
        }
    };
    (
        @impl
        [$($g:tt)*] [$($w:tt)*] [$Id:ty]
        [$this:ident] [$state_part:ident] [$StatePart:ty] -> [$($opt_tr:path)?] [$($tr:path)?] [$($opt_ty:ty)?] [$($ty:ty)?]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dep obj return type\n\n",
            $crate::std_stringify!($(optional dyn($opt_tr))? $((trait $tr))? $(optional($opt_ty))? $(($ty))?),
            "\
                \n\n\
                allowed forms are \
                '($ty:ty)', \
                'dyn($tr:path)', \
                'optional($ty:ty)', and \
                'optional dyn($tr:path)'\
            "
        ));
    };
    (
        @drop_bindings
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [
            $(
                [$this:ident] [$state_part:ident] [$StatePart:ty] [$($opt_tr:path)?] [$($tr:path)?] [$($opt_ty:ty)?] [$($ty:ty)?]
                [$field_mut:expr] [$field:expr]
            )*
        ]
    ) => {
        impl $($g)* $Id $($w)* {
            fn drop_bindings_priv(self, state: &mut dyn $crate::dyn_context_State) {
                $(
                    let $this = self;
                    let $state_part: &mut $StatePart = <dyn $crate::dyn_context_State as $crate::dyn_context_StateExt>::get_mut(state);
                    $(
                        let bindings = <dyn $tr as $crate::DepType>::collect_all_bindings($field);
                    )?
                    $(
                        let bindings = if let $crate::std_option_Option::Some(f) = $field {
                            <dyn $opt_tr as $crate::DepType>::collect_all_bindings(f)
                        } else {
                            $crate::std_vec_Vec::new()
                        };
                    )?
                    $(
                        let bindings = <$ty as $crate::DepType>::collect_all_bindings($field);
                    )?
                    $(
                        let bindings = if let $crate::std_option_Option::Some(f) = $field {
                            <$opt_ty as $crate::DepType>::collect_all_bindings(f)
                        } else {
                            $crate::std_vec_Vec::new()
                        };
                    )?
                    for binding in bindings {
                        binding.drop_self(state);
                    }
                )*
                $(
                    let $this = self;
                    let $state_part: &mut $StatePart = <dyn $crate::dyn_context_State as $crate::dyn_context_StateExt>::get_mut(state);
                    $(
                        let handlers = <dyn $tr as $crate::DepType>::take_all_handlers($field_mut);
                    )?
                    $(
                        let handlers = if let $crate::std_option_Option::Some(f) = $field_mut {
                            <dyn $opt_tr as $crate::DepType>::take_all_handlers(f)
                        } else {
                            $crate::std_vec_Vec::new()
                        };
                    )?
                    $(
                        let handlers = <$ty as $crate::DepType>::take_all_handlers($field_mut);
                        if !handlers.is_empty() {
                            <$ty as $crate::DepType>::update_parent_children_has_handlers(state, <Self as $crate::components_arena_ComponentId>::into_raw(self));
                        }
                    )?
                    $(
                        let handlers = if let $crate::std_option_Option::Some(f) = $field_mut {
                            <$opt_ty as $crate::DepType>::take_all_handlers(f)
                        } else {
                            $crate::std_vec_Vec::new()
                        };
                    )?
                    for handler in handlers {
                        handler.clear(state);
                    }
                )*
            }
        }
    };
}

/// Specifies dependency objects list and accessing methods in simplified form.
///
/// Accepts input in the following form:
///
/// ```ignore
/// $(
///     $Id:ty
/// |
///     <$generics> $Id:ty $(where $where_clause)?
/// )
/// {
///     $($(
///         $(optional)? $(type $ty:ty | trait $tr:path) =>
///             $StatePart:ty $({ . $state_part_field:tt })? | . $component_field:tt
///     ),+ $(,)?)?
/// }
/// ```
#[macro_export]
macro_rules! impl_dep_obj {
    (
        $($token:tt)+
    ) => {
        $crate::generics_parse! {
            $crate::impl_dep_obj_impl {
            }
            $($token)+
        }
    };
}

#[macro_export]
macro_rules! impl_dep_obj_impl {
    (
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $Id:ty {
            $($objs:tt)*
        }
    ) => {
        $crate::impl_dep_obj_impl! {
            @objs
            [$($g)*] [$($r)*] [$($w)*] [$Id]
            [] [] [] []
            [, $($objs)*]
        }
    };
    (
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!($crate::indoc_indoc!("
            invalid dep obj implementation, allowed form is

            $(
                $Id:ty
            |
                <$generics> $Id:ty $(where $where_clause)?
            )
            {
                $($(
                    $(optional)? $(type $ty:ty | trait $tr:path) =>
                        $StatePart:ty $({ . $state_part_field:tt })? | . $component_field:tt
                ),+ $(,)?)?
            }

        "));
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [, type $Obj:ty => $StatePart:ty | . $component_field:tt $($tail:tt)* ]
    ) => {
        $crate::impl_dep_obj_impl! {
            @objs
            [$($g)*] [$($r)*] [$($w)*] [$Id]
            [$($ty)* [[$Obj] [$StatePart] [.0] [. $component_field]]] [$($opt_ty)*] [$($tr)*] [$($opt_tr)*]
            [$($tail)*]
        }
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [, type $Obj:ty => $StatePart:ty { . $state_part_field:tt } | . $component_field:tt $($tail:tt)* ]
    ) => {
        $crate::impl_dep_obj_impl! {
            @objs
            [$($g)*] [$($r)*] [$($w)*] [$Id]
            [$($ty)* [[$Obj] [$StatePart] [. $state_part_field] [. $component_field]]] [$($opt_ty)*] [$($tr)*] [$($opt_tr)*]
            [$($tail)*]
        }
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [, optional type $Obj:ty => $StatePart:ty | . $component_field:tt $($tail:tt)* ]
    ) => {
        $crate::impl_dep_obj_impl! {
            @objs
            [$($g)*] [$($r)*] [$($w)*] [$Id]
            [$($ty)*] [$($opt_ty)* [[$Obj] [$StatePart] [.0] [. $component_field]]] [$($tr)*] [$($opt_tr)*]
            [$($tail)*]
        }
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [, optional type $Obj:ty => $StatePart:ty { . $state_part_field:tt } | . $component_field:tt $($tail:tt)* ]
    ) => {
        $crate::impl_dep_obj_impl! {
            @objs
            [$($g)*] [$($r)*] [$($w)*] [$Id]
            [$($ty)*] [$($opt_ty)* [[$Obj] [$StatePart] [. $state_part_field] [. $component_field]]] [$($tr)*] [$($opt_tr)*]
            [$($tail)*]
        }
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [, trait $Obj:path => $StatePart:ty | . $component_field:tt $($tail:tt)* ]
    ) => {
        $crate::impl_dep_obj_impl! {
            @objs
            [$($g)*] [$($r)*] [$($w)*] [$Id]
            [$($ty)*] [$($opt_ty)*] [$($tr)* [[$Obj] [$StatePart] [.0] [. $component_field]]] [$($opt_tr)*]
            [$($tail)*]
        }
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [, trait $Obj:path => $StatePart:ty { . $state_part_field:tt } | . $component_field:tt $($tail:tt)* ]
    ) => {
        $crate::impl_dep_obj_impl! {
            @objs
            [$($g)*] [$($r)*] [$($w)*] [$Id]
            [$($ty)*] [$($opt_ty)*] [$($tr)* [[$Obj] [$StatePart] [. $state_part_field] [. $component_field]]] [$($opt_tr)*]
            [$($tail)*]
        }
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [, optional trait $Obj:path => $StatePart:ty | . $component_field:tt $($tail:tt)* ]
    ) => {
        $crate::impl_dep_obj_impl! {
            @objs
            [$($g)*] [$($r)*] [$($w)*] [$Id]
            [$($ty)*] [$($opt_ty)*] [$($tr)*] [$($opt_tr)* [[$Obj] [$StatePart] [.0] [. $component_field]]]
            [$($tail)*]
        }
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [, optional trait $Obj:path => $StatePart:ty { . $state_part_field:tt } | . $component_field:tt $($tail:tt)* ]
    ) => {
        $crate::impl_dep_obj_impl! {
            @objs
            [$($g)*] [$($r)*] [$($w)*] [$Id]
            [$($ty)*] [$($opt_ty)*] [$($tr)*] [$($opt_tr)* [[$Obj] [$StatePart] [. $state_part_field] [. $component_field]]]
            [$($tail)*]
        }
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$([[$ty_Obj:ty] [$ty_StatePart:ty] [$($ty_state_part_field:tt)*] [$($ty_component_field:tt)*]])*]
        [$([[$opt_ty_Obj:ty] [$opt_ty_StatePart:ty] [$($opt_ty_state_part_field:tt)*] [$($opt_ty_component_field:tt)*]])*]
        [$([[$tr_Obj:path] [$tr_StatePart:ty] [$($tr_state_part_field:tt)*] [$($tr_component_field:tt)*]])*]
        [$([[$opt_tr_Obj:path] [$opt_tr_StatePart:ty] [$($opt_tr_state_part_field:tt)*] [$($opt_tr_component_field:tt)*]])*]
        [$(,)?]
    ) => {
        $crate::dep_obj! {
            impl $($g)* $Id $($w)* {
                $(
                    fn(self as this, state_part: $ty_StatePart) -> ($ty_Obj) {
                        if mut {
                            &mut state_part $($ty_state_part_field)* [this.0] $($ty_component_field)*
                        } else {
                            &state_part $($ty_state_part_field)* [this.0] $($ty_component_field)*
                        }
                    }
                )*
                $(
                    fn(self as this, state_part: $opt_ty_StatePart) -> optional($opt_ty_Obj) {
                        if mut {
                            state_part $($opt_ty_state_part_field)* [this.0] $($opt_ty_component_field)* .as_mut()
                        } else {
                            state_part $($opt_ty_state_part_field)* [this.0] $($opt_ty_component_field)* .as_ref()
                        }
                    }
                )*
                $(
                    fn(self as this, state_part: $tr_StatePart) -> dyn($tr_Obj) {
                        if mut {
                            state_part $($tr_state_part_field)* [this.0] $($tr_component_field)* .as_mut()
                        } else {
                            state_part $($tr_state_part_field)* [this.0] $($tr_component_field)* .as_ref()
                        }
                    }
                )*
                $(
                    fn(self as this, state_part: $opt_tr_StatePart) -> optional dyn($opt_tr_Obj) {
                        if mut {
                            state_part $($opt_tr_state_part_field)* [this.0] $($opt_tr_component_field)* .as_mut().as_mut()
                        } else {
                            state_part $($opt_tr_state_part_field)* [this.0] $($opt_tr_component_field)* .as_ref().as_ref()
                        }
                    }
                )*
            }
        }
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [, $($token:tt)* ]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dep obj access function definition\n\n",
            $crate::std_stringify!($($token)*),
            "\n\n",
            $crate::indoc_indoc!("
                allowed form is

                $(optional)? $(type $ty:ty | trait $tr:path) =>
                    $StatePart:ty $({ . $state_part_field:tt })? | . $component_field:tt

            ")
        ));
    };
    (
        @objs
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] [$Id:ty]
        [$($ty:tt)*] [$($opt_ty:tt)*] [$($tr:tt)*] [$($opt_tr:tt)*]
        [$token:tt $($tail:tt)*]
    ) => {
        $crate::unexpected_token($token);
        $crate::std_compile_error!("missing comma");
    };
}

#[cfg(test)]
mod test {
}
