use crate::base::*;
use crate::binding::*;
use alloc::boxed::Box;
use alloc::collections::{TryReserveError, VecDeque};
use alloc::vec::Vec;
use components_arena::{Arena, ArenaItemsIntoValues, Component, ComponentId, Id};
use core::fmt::Debug;
use core::iter::once;
use core::mem::{replace, take};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::state::State;
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
        obj: Glob<Owner>,
        prop: DepProp<Owner, PropType>
    ) {
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
            prop.notify_children(state, obj, change);
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
}

pub trait DetachedDepObjId: ComponentId { }

impl<T: DetachedDepObjId> DepObjId for T {
    fn parent(self, _state: &dyn State) -> Option<Self> { None }

    fn next(self, _state: &dyn State) -> Self { self }

    fn first_child(self, _state: &dyn State) -> Option<Self> { None }
}

/// A dependency type.
/// Use the [`dep_type`] or the [`dep_type_with_builder`] macro
/// to create a type implementing this trait.
///
/// # Examples
///
/// ```rust
/// # #![feature(const_ptr_offset_from)]
/// use components_arena::{Arena, Component, NewtypeComponentId, Id};
/// use dep_obj::{DetachedDepObjId, dep_obj, dep_type};
/// use dep_obj::binding::{Bindings, Binding, Binding1};
/// use dyn_context::state::{State, StateExt};
/// use macro_attr_2018::macro_attr;
/// use std::any::{Any, TypeId};
///
/// dep_type! {
///     #[derive(Debug)]
///     pub struct MyDepType in MyDepTypeId as MyDepType {
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
/// pub struct MyApp {
///     bindings: Bindings,
///     my_dep_types: Arena<MyDepTypePrivateData>,
///     res: Binding<i32>,
/// }
///
/// impl State for MyApp {
///     fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
///         if ty == TypeId::of::<Bindings>() {
///             Some(&self.bindings)
///         } else if ty == TypeId::of::<MyApp>() {
///             Some(self)
///         } else {
///             None
///         }
///     }
///
///     fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
///         if ty == TypeId::of::<Bindings>() {
///             Some(&mut self.bindings)
///         } else if ty == TypeId::of::<MyApp>() {
///             Some(self)
///         } else {
///             None
///         }
///     }
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
///
///     dep_obj! {
///         pub fn obj(self as this, app: MyApp) -> (MyDepType) as MyDepType {
///             if mut {
///                 &mut app.my_dep_types[this.0].dep_data
///             } else {
///                 &app.my_dep_types[this.0].dep_data
///             }
///         }
///     }
/// }
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
///     res.set_source_1(app, &mut MyDepType::PROP_2.value_source(id.obj()));
///     assert_eq!(app.res.get_value(app), Some(10));
///     MyDepType::PROP_2.set(app, id.obj(), 5).immediate();
///     assert_eq!(app.res.get_value(app), Some(5));
///     id.drop_my_dep_type(app);
///     res.drop_self(app);
/// }
/// ```
pub trait DepType: Debug {
    type Id: DepObjId;
    type Dyn: ?Sized;

    #[doc(hidden)]
    fn core_base_priv(&self) -> &BaseDepObjCore<Self> where Self: Sized;

    #[doc(hidden)]
    fn core_base_priv_mut(&mut self) -> &mut BaseDepObjCore<Self> where Self: Sized;

    #[doc(hidden)]
    fn take_all_handlers(&mut self) -> Vec<Box<dyn AnyHandler>>;

    #[doc(hidden)]
    fn collect_all_bindings(&self) -> Vec<AnyBindingBase>;

    #[doc(hidden)]
    fn update_parent_children_has_handlers(state: &mut dyn State, obj: Glob<Self>) where Self: Sized;
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
    /// Using the [`dep_type!`](dep_type) or the [`dep_type_with_builder!`](dep_type_with_builder)
    /// macros garantees the function safe use.
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

    fn raise_raw(self, state: &mut dyn State, obj: Glob<Owner>, args: &ArgsType) -> bool {
        let obj = obj.get(state);
        let entry = self.entry(&obj);
        let bubble = entry.bubble;
        let handlers = entry.handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, args.clone());
        }
        bubble
    }

    pub fn raise<X: Convenient>(self, state: &mut dyn State, mut obj: Glob<Owner>, args: ArgsType) -> Re<X> {
        let bubble = self.raise_raw(state, obj, &args);
        if !bubble || args.handled() { return Re::Continue; }
        while let Some(parent) = obj.parent(state) {
            obj = parent;
            let bubble = self.raise_raw(state, obj, &args);
            debug_assert!(bubble);
            if args.handled() { return Re::Continue; }
        }
        Re::Continue
    }

    pub fn source(self, obj: Glob<Owner>) -> DepEventSource<Owner, ArgsType> {
        DepEventSource { obj, event: self }
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
    /// Using the [`dep_type!`](dep_type) or the [`dep_type_with_builder!`](dep_type_with_builder)
    /// macros garantees the function safe use.
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

    fn unstyled_non_local_value<T>(self, state: &dyn State, obj: Glob<Owner>, f: impl FnOnce(&PropType) -> T) -> T {
        let obj_ref = obj.get(state);
        let entry = self.entry(&obj_ref);
        if entry.inherits() {
            if let Some(parent) = obj.parent(state) {
                self.current_value(state, parent, f)
            } else {
                f(entry.default)
            }
        } else {
            f(entry.default)
        }
    }

    fn non_local_value<T>(self, state: &dyn State, obj: Glob<Owner>, f: impl FnOnce(&PropType) -> T) -> T {
        let obj_ref = obj.get(state);
        let entry = self.entry(&obj_ref);
        if let Some(value) = entry.style.as_ref() {
            f(value)
        } else {
            self.unstyled_non_local_value(state, obj, f)
        }
    }

    fn current_value<T>(self, state: &dyn State, obj: Glob<Owner>, f: impl FnOnce(&PropType) -> T) -> T {
        let obj_ref = obj.get(state);
        let entry = self.entry(&obj_ref);
        if let Some(value) = entry.local.as_ref() {
            f(value)
        } else {
            self.non_local_value(state, obj, f)
        }
    }

    #[doc(hidden)]
    pub fn update_parent_children_has_handlers(self, state: &mut dyn State, mut obj: Glob<Owner>) {
        while let Some(parent) = obj.parent(state) {
            obj = parent;
            let children_has_handlers = if let Some(first_child) = Owner::Id::from_raw(obj.id).first_child(state) {
                let mut child = first_child;
                loop {
                    let child_obj = Glob { id: child.into_raw(), descriptor: obj.descriptor };
                    let obj = child_obj.get(state);
                    let entry = self.entry(&obj);
                    debug_assert!(entry.inherits());
                    if !entry.handlers.is_empty() { break true; }
                    child = child.next(state);
                    if child == first_child { break false; }
                }
            } else {
                false
            };
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
            if children_has_handlers == entry_mut.handlers.children_has_handlers.unwrap() { return; }
            entry_mut.handlers.children_has_handlers = Some(children_has_handlers);
        }
    }

    fn notify_children(
        self,
        state: &mut dyn State,
        obj: Glob<Owner>,
        change: &Change<PropType>,
    ) {
        if let Some(first_child) = Owner::Id::from_raw(obj.id).first_child(state) {
            let mut child = first_child;
            loop {
                let child_obj = Glob { id: child.into_raw(), descriptor: obj.descriptor };
                let mut obj_mut = child_obj.get_mut(state);
                let entry_mut = self.entry_mut(&mut obj_mut);
                debug_assert!(entry_mut.inherits());
                if entry_mut.local.is_none() && entry_mut.style.is_none() {
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute(state, change, child_obj, self);
                }
                child = child.next(state);
                if child == first_child { break; }
            }
        }
    }

    fn un_set_core(self, state: &mut dyn State, obj: Glob<Owner>, value: Option<PropType>) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        let old = replace(&mut entry_mut.local, value.clone());
        if old == value { return; }
        let handlers = entry_mut.handlers.clone();
        let change = if old.is_some() && value.is_some() {
            unsafe { Change { old: old.unwrap_unchecked(), new: value.unwrap_unchecked() } }
        } else {
            if let Some(change) = self.non_local_value(state, obj, |non_local| {
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
        handlers.execute(state, &change, obj, self);
    }

    fn un_set(self, state: &mut dyn State, obj: Glob<Owner>, mut value: Option<PropType>) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        if replace(&mut entry_mut.enqueue, true) {
            let queue = entry_mut.queue.get_or_insert_with(VecDeque::new);
            queue.push_back(value);
            return;
        }
        loop {
            self.un_set_core(state, obj, value);
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
            if let Some(queue) = entry_mut.queue.as_mut() {
                if let Some(queue_head) = queue.pop_front() { value = queue_head; } else { break; }
            } else {
                break;
            }
        }
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.enqueue = false;
    }

    pub fn set<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner>, value: PropType) -> Re<X> {
        self.un_set(state, obj, Some(value));
        Re::Continue
    }

    pub fn unset<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner>) -> Re<X> {
        self.un_set(state, obj, None);
        Re::Continue
    }

    fn bind_raw(
        self,
        state: &mut dyn State,
        obj: Glob<Owner>,
        binding: BindingBase<PropType>
    ) where Owner: 'static {
        self.unbind(state, obj);
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.binding = Some(binding);
        binding.set_target(state, Box::new(DepPropSet { prop: self, obj }));
        binding.set_holder(state, Box::new(DepPropSet { prop: self, obj }));
    }

    pub fn bind(
        self,
        state: &mut dyn State,
        obj: Glob<Owner>,
        binding: impl Into<BindingBase<PropType>>
    ) where Owner: 'static {
        self.bind_raw(state, obj, binding.into());
    }

    pub fn unbind(self, state: &mut dyn State, obj: Glob<Owner>) {
        if let Some(binding) = {
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
            entry_mut.binding
        } {
            binding.drop_self(state);
        }
    }

    fn clear_binding(self, state: &mut dyn State, obj: Glob<Owner>) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        let ok = entry_mut.binding.take().is_some();
        debug_assert!(ok);
    }

    pub fn value_source(self, obj: Glob<Owner>) -> DepPropValueSource<Owner, PropType> {
        DepPropValueSource { obj, prop: self }
    }

    pub fn change_source(self, obj: Glob<Owner>) -> DepPropChangeSource<Owner, PropType> {
        DepPropChangeSource { obj, prop: self }
    }

    pub fn change_initial_source(self, obj: Glob<Owner>) -> DepPropChangeInitialSource<Owner, PropType> {
        DepPropChangeInitialSource { obj, prop: self }
    }

    pub fn change_final_source(self, obj: Glob<Owner>) -> DepPropChangeFinalSource<Owner, PropType> {
        DepPropChangeFinalSource { obj, prop: self }
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DepPropSet<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> Target<PropType> for DepPropSet<Owner, PropType> {
    fn execute(&self, state: &mut dyn State, value: PropType) {
        self.prop.set(state, self.obj, value).immediate();
    }
}

impl<Owner: DepType, PropType: Convenient> Holder for DepPropSet<Owner, PropType> {
    fn release(&self, state: &mut dyn State) {
        self.prop.clear_binding(state, self.obj);
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
    /// Using the [`dep_type!`](dep_type) or the [`dep_type_with_builder!`](dep_type_with_builder)
    /// macros garantees the function safe use.
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
        obj: Glob<Owner>,
        mut modification: DepVecModification<ItemType>,
    ) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        if replace(&mut entry_mut.enqueue, true) {
            let queue = entry_mut.queue.get_or_insert_with(VecDeque::new);
            queue.push_back(modification);
            return;
        }
        loop {
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
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
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
            if let Some(queue) = entry_mut.queue.as_mut() {
                if let Some(queue_head) = queue.pop_front() { modification = queue_head; } else { break; }
            } else {
                break;
            }
        }
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.enqueue = false;
    }

    pub fn clear<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner>) -> Re<X> {
        self.modify(state, obj, DepVecModification::Clear);
        Re::Continue
    }

    pub fn push<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner>, item: ItemType) -> Re<X> {
        self.insert(state, obj, DepVecInsertPos::AfterLastItem, item)
    }

    pub fn insert<X: Convenient>(
        self,
        state: &mut dyn State,
        obj: Glob<Owner>,
        pos: DepVecInsertPos<ItemType>,
        item: ItemType
    ) -> Re<X> {
        self.modify(state, obj, DepVecModification::Insert(pos, item));
        Re::Continue
    }

    pub fn move_<X: Convenient>(
        self,
        state: &mut dyn State,
        obj: Glob<Owner>,
        old_pos: DepVecItemPos<ItemType>,
        new_pos: DepVecInsertPos<ItemType>
    ) -> Re<X> {
        self.modify(state, obj, DepVecModification::Move(old_pos, new_pos));
        Re::Continue
    }

    pub fn remove<X: Convenient>(
        self,
        state: &mut dyn State,
        obj: Glob<Owner>,
        pos: DepVecItemPos<ItemType>
    ) -> Re<X> {
        self.modify(state, obj, DepVecModification::Remove(pos));
        Re::Continue
    }

    pub fn extend_from<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner>, other: Vec<ItemType>) -> Re<X> {
        self.modify(state, obj, DepVecModification::ExtendFrom(other));
        Re::Continue
    }

    pub fn changed_source(self, obj: Glob<Owner>) -> DepVecChangedSource<Owner, ItemType> {
        DepVecChangedSource { obj, vec: self }
    }

    pub fn item_source(self, obj: Glob<Owner>) -> DepVecItemSource<Owner, ItemType> {
        DepVecItemSource { obj, vec: self, update: None }
    }

    pub fn item_source_with_update(self, update: impl Into<BindingBase<()>>, obj: Glob<Owner>) -> DepVecItemSource<Owner, ItemType> {
        DepVecItemSource { obj, vec: self, update: Some(update.into()) }
    }

    pub fn item_initial_final_source(self, obj: Glob<Owner>) -> DepVecItemInitialFinalSource<Owner, ItemType> {
        DepVecItemInitialFinalSource { obj, vec: self, update: None }
    }

    pub fn item_initial_final_source_with_update(self, update: impl Into<BindingBase<()>>, obj: Glob<Owner>) -> DepVecItemInitialFinalSource<Owner, ItemType> {
        DepVecItemInitialFinalSource { obj, vec: self, update: Some(update.into()) }
    }
}

struct AddedBindingHolder<Owner: DepType> {
    obj: Glob<Owner>,
    binding_id: Id<AnyBindingBase>,
}

impl<Owner: DepType> Holder for AddedBindingHolder<Owner> {
    fn release(&self, state: &mut dyn State) {
        let mut obj_mut = self.obj.get_mut(state);
        obj_mut.core_base_priv_mut().added_bindings.remove(self.binding_id);
    }
}

impl<Owner: DepType> Glob<Owner> {
    pub fn parent(self, state: &dyn State) -> Option<Self> {
        Owner::Id::from_raw(self.id).parent(state).map(|id| Glob { id: id.into_raw(), descriptor: self.descriptor })
    }

    fn add_binding_raw<T: Convenient>(self, state: &mut dyn State, binding: BindingBase<T>) where Owner: 'static {
        let mut obj_mut = self.get_mut(state);
        let binding_id = obj_mut.core_base_priv_mut().added_bindings.insert(|id| (binding.into(), id));
        binding.set_holder(state, Box::new(AddedBindingHolder { obj: self, binding_id }));
    }

    pub fn add_binding<T: Convenient>(self, state: &mut dyn State, binding: impl Into<BindingBase<T>>) where Owner: 'static {
        self.add_binding_raw(state, binding.into())
    }

    pub fn apply_style(
        self,
        state: &mut dyn State,
        style: Option<Style<Owner>>,
    ) -> Option<Style<Owner>> {
        let mut on_changed = Vec::new();
        let obj = &mut self.get_mut(state);
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
        let obj = &mut self.get_mut(state);
        obj.core_base_priv_mut().style = style;
        for on_changed in on_changed {
            on_changed(state);
        }
        old
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
        obj: Glob<Owner>,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn State)>>;
}

clone_trait_object!(<Owner: DepType> AnySetter<Owner>);

impl<Owner: DepType + 'static, PropType: Convenient> AnySetter<Owner> for Setter<Owner, PropType> where Owner::Id: 'static {
    fn prop_offset(&self) -> usize { self.prop.offset }

    fn un_apply(
        &self,
        state: &mut dyn State,
        obj: Glob<Owner>,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn State)>> {
        let obj_mut = &mut obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(obj_mut);
        let value = if unapply { None } else { Some(self.value.clone()) };
        let old = replace(&mut entry_mut.style, value.clone());
        if entry_mut.local.is_some() || old == value { return None; }
        let handlers = entry_mut.handlers.clone();
        let change = if old.is_some() && value.is_some() {
            unsafe { Change { old: old.unwrap_unchecked(), new: value.unwrap_unchecked() } }
        } else {
            self.prop.unstyled_non_local_value(state, obj, |unstyled_non_local_value| {
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
        Some(Box::new(move |state: &'_ mut dyn State| handlers.execute(state, &change, obj, prop)))
    }
}

/// A dictionary mapping a subset of target type properties to the values.
/// Every dependency object can have an applied style at every moment.
/// To switch an applied style, use the [`Glob::apply_style`] function.
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
    ) -> bool where Owner: 'static {
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

pub trait DepObjBaseBuilder<OwnerId: ComponentId> {
    fn state(&self) -> &dyn State;
    fn state_mut(&mut self) -> &mut dyn State;
    fn id(&self) -> OwnerId;
}

#[derive(Educe)]
#[educe(Debug)]
struct DepEventHandledSource<Owner: DepType, ArgsType: DepEventArgs> {
    obj: Glob<Owner>,
    handler_id: Id<BoxedHandler<ArgsType>>,
    event: DepEvent<Owner, ArgsType>,
}

impl<Owner: DepType, ArgsType: DepEventArgs> HandlerId for DepEventHandledSource<Owner, ArgsType> {
    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.event.entry_mut(&mut obj);
        entry_mut.handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepEventSource<Owner: DepType, ArgsType: DepEventArgs> {
    obj: Glob<Owner>,
    event: DepEvent<Owner, ArgsType>,
}

impl<Owner: DepType + 'static, ArgsType: DepEventArgs + 'static> Source for DepEventSource<Owner, ArgsType> {
    type Value = ArgsType;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<ArgsType>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.event.entry_mut(&mut obj);
        let handler_id = entry.handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        HandledSource {
            handler_id: Box::new(DepEventHandledSource { handler_id, obj: self.obj, event: self.event }),
            init: None // TODO some events with cached value?
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledValueSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner>,
    handler_id: Id<BoxedHandler<PropType>>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledValueSource<Owner, PropType> {
    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(&mut obj);
        entry_mut.handlers.value_handlers.remove(self.handler_id);
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledChangeInitialSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledChangeInitialSource<Owner, PropType> {
    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(&mut obj);
        let handler = entry_mut.handlers.change_initial_handler.take();
        debug_assert!(handler.is_some());
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledChangeFinalSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledChangeFinalSource<Owner, PropType> {
    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(&mut obj);
        let handler = entry_mut.handlers.change_final_handler.take();
        debug_assert!(handler.is_some());
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledChangeSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner>,
    handler_id: Id<BoxedHandler<Change<PropType>>>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledChangeSource<Owner, PropType> {
    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(&mut obj);
        entry_mut.handlers.change_handlers.remove(self.handler_id);
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropValueSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropValueSource<Owner, PropType> {
    type Value = PropType;
    type Cache = ValueCache<PropType>;

    fn handle(&self, state: &mut dyn State, handler: Box<dyn Handler<PropType>>) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.prop.entry_mut(&mut obj);
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler_id = entry.handlers.value_handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
        let value = self.prop.current_value(state, self.obj, |x| x.clone());
        let prop = self.prop;
        let obj = self.obj;
        let init = Box::new(move |state: &mut dyn State| {
            let obj = obj.get(state);
            let entry = prop.entry(&obj);
            let handler = entry.handlers.value_handlers[handler_id].0.clone();
            handler.execute(state, value);
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledValueSource { handler_id, obj: self.obj, prop: self.prop }),
            init: Some(init)
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropChangeSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropChangeSource<Owner, PropType> {
    type Value = Change<PropType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<Change<PropType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.prop.entry_mut(&mut obj);
        let default_value = entry.default;
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler_id = entry.handlers.change_handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
        let change = self.prop.current_value(state, self.obj, |value| {
            if value == default_value {
                None
            } else {
                Some(Change { old: default_value.clone(), new: value.clone() })
            }
        });
        let init = change.map(|change| {
            let prop = self.prop;
            let obj = self.obj;
            Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = prop.entry(&obj);
                let handler = entry.handlers.change_handlers[handler_id].0.clone();
                handler.execute(state, change);
            }) as _
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledChangeSource { handler_id, obj: self.obj, prop: self.prop }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropChangeInitialSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropChangeInitialSource<Owner, PropType> {
    type Value = Change<PropType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<Change<PropType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.prop.entry_mut(&mut obj);
        let default_value = entry.default;
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler = entry.handlers.change_initial_handler.replace(handler);
        assert!(handler.is_none(), "duplicate initial handler");
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
        let change = self.prop.current_value(state, self.obj, |value| {
            if value == default_value {
                None
            } else {
                Some(Change { old: default_value.clone(), new: value.clone() })
            }
        });
        let init = change.map(|change| {
            let prop = self.prop;
            let obj = self.obj;
            Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = prop.entry(&obj);
                let handler = entry.handlers.change_initial_handler.clone().unwrap();
                handler.execute(state, change);
            }) as _
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledChangeInitialSource { obj: self.obj, prop: self.prop }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropChangeFinalSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropChangeFinalSource<Owner, PropType> {
    type Value = Change<PropType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<Change<PropType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.prop.entry_mut(&mut obj);
        let default_value = entry.default;
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler = entry.handlers.change_final_handler.replace(handler);
        assert!(handler.is_none(), "duplicate final handler");
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
        let change = self.prop.current_value(state, self.obj, |value| {
            if value == default_value {
                None
            } else {
                Some(Change { old: default_value.clone(), new: value.clone() })
            }
        });
        let init = change.map(|change| {
            let prop = self.prop;
            let obj = self.obj;
            Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = prop.entry(&obj);
                let handler = entry.handlers.change_final_handler.clone().unwrap();
                handler.execute(state, change);
            }) as _
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledChangeFinalSource { obj: self.obj, prop: self.prop }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecChangedHandledSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner>,
    handler_id: Id<BoxedHandler<()>>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecChangedHandledSource<Owner, ItemType> {
    fn unhandle(&self, state: &mut dyn State, _dropping_binding: AnyBindingBase) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.vec.entry_mut(&mut obj);
        entry_mut.handlers.changed_handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecItemHandledInitialFinalSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecItemHandledInitialFinalSource<Owner, ItemType> {
    fn unhandle(&self, state: &mut dyn State, dropping_binding: AnyBindingBase) {
        let mut obj = self.obj.get_mut(state);
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
    obj: Glob<Owner>,
    handler_id: Id<ItemHandler<ItemType>>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecItemHandledSource<Owner, ItemType> {
    fn unhandle(&self, state: &mut dyn State, dropping_binding: AnyBindingBase) {
        let mut obj = self.obj.get_mut(state);
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
    obj: Glob<Owner>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> Source for DepVecChangedSource<Owner, ItemType> {
    type Value = ();
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<()>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let changed = !entry.items.is_empty();
        let handler_id = entry.handlers.changed_handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        let init = if changed {
            let vec = self.vec;
            let obj = self.obj;
            Some(Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = vec.entry(&obj);
                let handler = entry.handlers.changed_handlers[handler_id].0.clone();
                handler.execute(state, ());
            }) as _)
        } else {
            None
        };
        HandledSource {
            handler_id: Box::new(DepVecChangedHandledSource { handler_id, obj: self.obj, vec: self.vec }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DepVecItemInitialFinalSourceUpdate<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> Target<()> for DepVecItemInitialFinalSourceUpdate<Owner, ItemType> {
    fn execute(&self, state: &mut dyn State, (): ()) {
        self.vec.modify(state, self.obj, DepVecModification::Update(None));
    }
}

impl<Owner: DepType, ItemType: Convenient> Holder for DepVecItemInitialFinalSourceUpdate<Owner, ItemType> {
    fn release(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let ok = entry.handlers.item_initial_final_handler.as_mut().unwrap().update.take().is_some();
        debug_assert!(ok);
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DepVecItemSourceUpdate<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner>,
    vec: DepVec<Owner, ItemType>,
    handler_id: Id<ItemHandler<ItemType>>,
}

impl<Owner: DepType, ItemType: Convenient> Target<()> for DepVecItemSourceUpdate<Owner, ItemType> {
    fn execute(&self, state: &mut dyn State, (): ()) {
        self.vec.modify(state, self.obj, DepVecModification::Update(Some(self.handler_id)));
    }
}

impl<Owner: DepType, ItemType: Convenient> Holder for DepVecItemSourceUpdate<Owner, ItemType> {
    fn release(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let ok = entry.handlers.item_handlers[self.handler_id].update.take().is_some();
        debug_assert!(ok);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecItemSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner>,
    vec: DepVec<Owner, ItemType>,
    update: Option<BindingBase<()>>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> Source for DepVecItemSource<Owner, ItemType> {
    type Value = ItemChange<ItemType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<ItemChange<ItemType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let items = entry.items.clone();
        let handler_id = entry.handlers.item_handlers.insert(
            |handler_id| (ItemHandler { handler, update: self.update }, handler_id)
        );
        if let Some(update) = self.update {
            update.set_target(state, Box::new(DepVecItemSourceUpdate { obj: self.obj, vec: self.vec, handler_id }));
            update.set_holder(state, Box::new(DepVecItemSourceUpdate { obj: self.obj, vec: self.vec, handler_id }));
        }
        let init = if items.is_empty() {
            None
        } else {
            let vec = self.vec;
            let obj = self.obj;
            Some(Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
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
            handler_id: Box::new(DepVecItemHandledSource { handler_id, obj: self.obj, vec: self.vec }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecItemInitialFinalSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner>,
    vec: DepVec<Owner, ItemType>,
    update: Option<BindingBase<()>>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> Source for DepVecItemInitialFinalSource<Owner, ItemType> {
    type Value = ItemChange<ItemType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<ItemChange<ItemType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let items = entry.items.clone();
        let handler = ItemHandler { handler, update: self.update };
        assert!(entry.handlers.item_initial_final_handler.replace(handler).is_none(), "duplicate initial handler");
        if let Some(update) = self.update {
            update.set_target(state, Box::new(DepVecItemInitialFinalSourceUpdate { obj: self.obj, vec: self.vec }));
            update.set_holder(state, Box::new(DepVecItemInitialFinalSourceUpdate { obj: self.obj, vec: self.vec }));
        }
        let init = if items.is_empty() {
            None
        } else {
            let vec = self.vec;
            let obj = self.obj;
            Some(Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
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
            handler_id: Box::new(DepVecItemHandledInitialFinalSource { obj: self.obj, vec: self.vec }),
            init
        }
    }
}

#[doc(hidden)]
pub trait NewPriv {
    fn new_priv() -> Self;
}

pub trait DepObj<Dyn> {
    type Type: DepType;

    fn descriptor() -> GlobDescriptor<Self::Type>;
}
