use crate::base::*;
use alloc::boxed::Box;
use components_arena::{Component, ComponentId, Id, Arena, NewtypeComponentId, RawId};
use core::any::{Any, TypeId};
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::state::{SelfState, State, StateExt};
use educe::Educe;
use macro_attr_2018::macro_attr;
use panicking::panicking;
use phantom_type::PhantomType;

#[derive(Educe)]
#[educe(Debug)]
struct ParamDescriptor<Obj> {
    arena: TypeId,
    #[educe(Debug(ignore))]
    get_raw: fn(arena: &dyn Any, id: RawId) -> &Obj,
    #[educe(Debug(ignore))]
    get_raw_mut: fn(arena: &mut dyn Any, id: RawId) -> &mut Obj
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct Param<Obj: 'static> {
    id: RawId,
    descriptor: &'static ParamDescriptor<Obj>,
}

pub struct ParamRef<'a, Obj: 'static> {
    arena: &'a dyn Any,
    param: Param<Obj>,
}

impl<'a, Obj> Deref for ParamRef<'a, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        (self.param.descriptor.get_raw)(self.arena.deref(), self.param.id)
    }
}

pub struct ParamMut<'a, Obj: 'static> {
    arena: &'a mut dyn Any,
    param: Param<Obj>,
}

impl<'a, Obj> Deref for ParamMut<'a, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        (self.param.descriptor.get_raw)(self.arena.deref(), self.param.id)
    }
}

impl<'a, Obj> DerefMut for ParamMut<'a, Obj> {
    fn deref_mut(&mut self) -> &mut Obj {
        (self.param.descriptor.get_raw_mut)(self.arena.deref_mut(), self.param.id)
    }
}

impl<Obj> Param<Obj> {
    pub fn get(self, state: &dyn State) -> ParamRef<Obj> {
        let arena = self.descriptor.arena;
        ParamRef {
            arena: state.get_raw(arena).unwrap_or_else(|| panic!("{:?} required", arena)),
            param: self
        }
    }

    pub fn get_mut(self, state: &mut dyn State) -> ParamMut<Obj> {
        let arena = self.descriptor.arena;
        ParamMut {
            arena: state.get_mut_raw(arena).unwrap_or_else(|| panic!("{:?} required", arena)),
            param: self
        }
    }
}


/// A helper struct, assisting to keep reactive code reentrant.
///
/// Dropping this value (e.g. with `let _ = ...`) most likely is a reentrancy violation.
/// Functions returning `Re` could not be called in a row in general case
/// without special precautions resulting in the adding a queue.
///
/// If you need to "split" control flow and perform several actions,
/// use intermediate dependecy property/event. Each dependency property/event has
/// an inner queue and support multiply listeners.
#[must_use]
pub struct Re<T: Convenient>(Option<T>);

impl<T: Convenient> Re<T> {
    /// Wraps `value` into the `Re` helper struct, allowing
    /// to return it from reactive callback.
    ///
    /// Such callback are required, for example,
    /// by `BindingExt` constructor's `dispatch` argument
    /// (see [`BindingExt1::new`], [`BindingExt2::new`], ...).
    #[allow(non_snake_case)]
    pub fn Yield(value: T) -> Re<T> {
        Re(Some(value))
    }

    /// An instance with no value.
    ///
    /// It indends to prevent remaining processing of current reactive action.
    /// For example, it can be used in `BindingExt` constructor's `dispatch` argument 
    /// (see [`BindingExt1::new`], [`BindingExt2::new`], ...)
    /// to prevent going value into the binding target.
    ///
    /// Also it can be used as `Re<!>` instance in places where no further reactive
    /// processing supposed, such as in a [`BindingBase::dispatch`] /  [`Binding::dispatch`]
    /// callback.
    #[allow(non_upper_case_globals)]
    pub const Continue: Re<T> = Re(None);
}

impl Re<!> {
    /// Explicitely drops `Re` with no value, allowing to call
    /// functions returning `Re` in any context.
    ///
    /// This could breaks code piece reenterability, so it
    /// is supposed to use in places, where it is garanteed to
    /// be not a problem, e.g. in `main` function (or any other
    /// place which executes once).
    pub fn immediate(self) {
        let _ = self;
    }
}

/// An object which can recieve values from a binding.
pub trait Target<T: Convenient>: DynClone {
    fn execute(&self, state: &mut dyn State, value: T);
}

clone_trait_object!(<T: Convenient> Target<T>);

/// An object controlling binding lifetime.
pub trait Holder {
    fn release(&self, state: &mut dyn State);
}

#[derive(Educe)]
#[educe(Clone)]
struct DispatchTarget<Context: Clone, T: Convenient> {
    context: Context,
    execute: fn(state: &mut dyn State, context: Context, value: T) -> Re<!>,
}

impl<Context: Clone, T: Convenient> Target<T> for DispatchTarget<Context, T> {
    fn execute(&self, state: &mut dyn State, value: T) {
        let _ = (self.execute)(state, self.context.clone(), value);
    }
}

#[derive(Educe)]
#[educe(Clone)]
struct FnTarget<Context: Clone, T: Convenient> {
    context: Context,
    execute: fn(state: &mut dyn State, context: Context, value: T)
}

impl<Context: Clone, T: Convenient> Target<T> for FnTarget<Context, T> {
    fn execute(&self, state: &mut dyn State, value: T) {
        (self.execute)(state, self.context.clone(), value);
    }
}

/// Base non-generic part of the [`Handler`] trait.
pub trait AnyHandler: Debug {
    fn clear(&self, state: &mut dyn State);
}

/// A value stream handler of any nature.
pub trait Handler<T>: Debug + DynClone + Send + Sync {
    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler>;
    fn execute(&self, state: &mut dyn State, args: T);
}

clone_trait_object!(<T> Handler<T>);

/// A value caching strategy.
pub trait SourceCache<T: Convenient>: Default + Debug {
    type Value: Convenient;
    fn update(&mut self, value: T);
    fn get(&self, current: Option<T>) -> Option<Self::Value>;
}

/// Simple straightforward value caching strategy.
#[derive(Educe)]
#[educe(Debug)]
pub struct ValueCache<T: Convenient>(Option<T>);

impl<T: Convenient> Default for ValueCache<T> {
    fn default() -> Self { ValueCache(None) }
}

impl<T: Convenient> SourceCache<T> for ValueCache<T> {
    type Value = T;

    fn update(&mut self, value: T) { self.0 = Some(value); }

    fn get(&self, current: Option<T>) -> Option<T> {
        current.map_or_else(|| self.0.clone(), |current| Some(current))
    }
}

/// Disabled caching.
#[derive(Debug, Default)]
pub struct NoCache(());

impl<T: Convenient> SourceCache<T> for NoCache {
    type Value = Option<T>;

    fn update(&mut self, _: T) { }

    fn get(&self, current: Option<T>) -> Option<Option<T>> { Some(current) }
}

/// An object which can send values to a binding.
pub trait Source: Debug {
    type Value: Convenient;
    type Cache: SourceCache<Self::Value>;
    fn handle(&self, state: &mut dyn State, handler: Box<dyn Handler<Self::Value>>) -> HandledSource;
}

/// An id of a [`Source`] handler, which can be used to unsubscribe the handler from source.
pub trait HandlerId: Debug {
    fn unhandle(&self, state: &mut dyn State, dropping_binding: AnyBindingBase);
}

/// The [`Source::handle`] method result.
#[derive(Educe)]
#[educe(Debug)]
pub struct HandledSource {
    pub handler_id: Box<dyn HandlerId>,
    #[educe(Debug(ignore))]
    pub init: Option<Box<dyn FnOnce(&mut dyn State)>>,
}

trait AnyBindingNode: Downcast {
    fn unhandle_sources_and_release_holder(&mut self, state: &mut dyn State, dropping_binding: AnyBindingBase);
}

impl_downcast!(AnyBindingNode);

macro_attr! {
    #[derive(Component!)]
    struct BoxedBindingNode(Box<dyn AnyBindingNode>);
}

/// An arena holding all bindings data.
/// There almost always will be only one object of that type in application.
pub struct Bindings(Arena<BoxedBindingNode>);

impl SelfState for Bindings { }

impl Bindings {
    pub const fn new() -> Self { Bindings(Arena::new()) }
}

impl Drop for Bindings {
    fn drop(&mut self) {
        if !panicking() {
            debug_assert!(self.0.items().is_empty(), "there are non-dropped bindings (count: {})", self.0.items().len());
        }
    }
}

trait AnyBindingNodeSources: Downcast {
    type Value: Convenient;
    fn unhandle(&mut self, state: &mut dyn State, dropping_binding: AnyBindingBase);
    fn get_value(&self) -> Option<Self::Value>;
    fn is_empty(&self) -> bool;
}

impl_downcast!(AnyBindingNodeSources assoc Value where Value: Convenient);

struct BindingNode<T: Convenient> {
    sources: Box<dyn AnyBindingNodeSources<Value=T>>,
    target: Option<Box<dyn Target<T>>>,
    holder: Option<Box<dyn Holder>>,
}

impl<T: Convenient> AnyBindingNode for BindingNode<T> {
    fn unhandle_sources_and_release_holder(&mut self, state: &mut dyn State, dropping_binding: AnyBindingBase) {
        self.holder.as_ref().map(|x| x.release(state));
        self.sources.unhandle(state, dropping_binding);
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!, Component!)]
    #[educe(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct AnyBindingBase(Id<BoxedBindingNode>);
}

impl AnyBindingBase {
    pub fn drop_self(self, state: &mut dyn State) {
        let bindings: &mut Bindings = state.get_mut();
        let mut node = bindings.0.remove(self.0);
        node.0.unhandle_sources_and_release_holder(state, self);
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct BindingBase<T: Convenient>(Id<BoxedBindingNode>, PhantomType<T>);
}

impl<T: Convenient> BindingBase<T> {
    pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
        let bindings: &mut Bindings = state.get_mut();
        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
        node.target = Some(target);
        assert!(node.sources.is_empty(), "set_target should be called before any set_source_*");
    }

    pub fn set_holder(self, state: &mut dyn State, holder: Box<dyn Holder>) {
        let bindings: &mut Bindings = state.get_mut();
        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
        node.holder = Some(holder);
        assert!(node.sources.is_empty(), "set_holder should be called before any set_source_*");
    }

    pub fn dispatch<Context: Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: T) -> Re<!>
    ) {
        self.set_target(state, Box::new(DispatchTarget { context, execute }));
    }

    pub fn set_target_fn<Context: Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: T)
    ) {
        self.set_target(state, Box::new(FnTarget { context, execute }));
    }

    pub fn drop_self(self, state: &mut dyn State) {
        AnyBindingBase::from(self).drop_self(state);
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct Binding<T: Convenient>(Id<BoxedBindingNode>, PhantomType<T>);
}

impl<T: Convenient> From<Binding<T>> for BindingBase<T> {
    fn from(binding: Binding<T>) -> Self {
        BindingBase(binding.0, PhantomType::new())
    }
}

impl<T: Convenient> From<Binding<T>> for AnyBindingBase {
    fn from(v: Binding<T>) -> AnyBindingBase {
        AnyBindingBase(v.0)
    }
}

impl<T: Convenient> Binding<T> {
    pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
        BindingBase::from(self).set_target(state, target);
    }

    pub fn set_holder(self, state: &mut dyn State, holder: Box<dyn Holder>) {
        BindingBase::from(self).set_holder(state, holder);
    }

    pub fn dispatch<Context: Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: T) -> Re<!>
    ) {
        BindingBase::from(self).dispatch(state, context, execute);
    }

    pub fn set_target_fn<Context: Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: T)
    ) {
        BindingBase::from(self).set_target_fn(state, context, execute);
    }

    pub fn drop_self(self, state: &mut dyn State) {
        AnyBindingBase::from(self).drop_self(state);
    }

    pub fn get_value(self, state: &dyn State) -> Option<T> {
        let bindings: &Bindings = state.get();
        let node = bindings.0[self.0].0.downcast_ref::<BindingNode<T>>().unwrap();
        node.sources.get_value()
    }
}

impl<T: Convenient> From<BindingBase<T>> for AnyBindingBase {
    fn from(v: BindingBase<T>) -> AnyBindingBase {
        AnyBindingBase(v.0)
    }
}

pub use n::Binding0;
pub use n::Binding1;
pub use n::Binding2;
pub use n::Binding3;
pub use n::BindingExt0;
pub use n::BindingExt1;
pub use n::BindingExt2;
pub use n::BindingExt3;

macro_rules! binding_n {
    ($n:tt; $($i:tt),* $(,)?) => {
        binding_n! { @unwrap [] [$n] [$($i)*] [$($i)*] }
    };
    (@unwrap [$($r:tt)*] [$n:tt] [] [$($j:tt)*]) => {
        binding_n! { @done [$n] $($r)* }
    };
    (@unwrap [$($r:tt)*] [$n:tt] [$i0:tt $($i:tt)*] [$($j:tt)*]) => {
        binding_n! { @unwrap [$($r)* [$i0 $($j)+]] [$n] [$($i)*] [$($j)*] }
    };
    (@done [$n:tt] $([$i:tt $($j:tt)*])*) => {
        $crate::paste_paste! {
            #[derive(Educe)]
            #[educe(Debug)]
            struct [< BindingExt $n NodeSources >] <P: Debug + 'static, $( [< S $i >] : Source, )* T: Convenient> {
                param: P,
                $(
                    [< source_ $i >] : Option<(Box<dyn HandlerId>, [< S $i >] ::Cache )>,
                )*
                #[allow(dead_code)]
                #[educe(Debug(ignore))]
                dispatch: fn(
                    &mut dyn State,
                    Param <  P >,
                    $( < < [< S $i >] as Source > ::Cache as SourceCache< [< S $i >] ::Value > >::Value ),*
                ) -> Re<T>,
            }

            impl<
                P: Debug + 'static,
                $( [< S $i >] : Source + 'static, )*
                T: Convenient
            > AnyBindingNodeSources for [< BindingExt $n NodeSources >] <P, $( [< S $i >] , )* T> {
                type Value = T;

                fn is_empty(&self) -> bool {
                    $(
                        if self. [< source_ $i >] .is_some() {
                            return false;
                        }
                    )*
                    true
                }

                #[allow(unused_variables)]
                fn unhandle(&mut self, state: &mut dyn State, dropping_binding: AnyBindingBase) {
                    $(
                        if let Some(source) = self. [< source_ $i >] .take() {
                            source.0.unhandle(state, dropping_binding);
                        }
                    )*
                }

                fn get_value(&self) -> Option<T> {
                    unreachable!()
                }
            }

            macro_attr! {
                #[derive(Educe, NewtypeComponentId!)]
                #[educe(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
                pub struct [< BindingExt $n >] <P, $( [< S $i >] : Source, )* T: Convenient>(
                    Id<BoxedBindingNode>,
                    PhantomType<(P, ($( [< S $i >] ,)* ), T)>
                );
            }

            impl<
                P: Debug + 'static,
                $( [< S $i >] : Source + 'static, )*
                T: Convenient
            > [< BindingExt $n >] <P, $( [< S $i >] , )* T> {
                pub fn new(
                    state: &mut dyn State,
                    param: P,
                    dispatch: fn(
                        &mut dyn State,
                        Param < P >,
                        $( < < [< S $i >] as Source > ::Cache as SourceCache< [< S $i >] ::Value > >::Value ),*
                    ) -> Re<T>,
                ) -> Self {
                    let bindings: &mut Bindings = state.get_mut();
                    let id = bindings.0.insert(|id| {
                        let sources: [< BindingExt $n NodeSources >] <P, $( [< S $i >] ,)* T> = [< BindingExt $n NodeSources >] {
                            param,
                            $(
                                [< source_ $i >] : None,
                            )*
                            dispatch,
                        };
                        let node: BindingNode<T> = BindingNode {
                            sources: Box::new(sources),
                            target: None,
                            holder: None,
                        };
                        (BoxedBindingNode(Box::new(node)), id)
                    });
                    [< BindingExt $n >] (id, PhantomType::new())
                }

                fn param_ref(arena: &dyn Any, id: RawId) -> &P {
                    let bindings = arena.downcast_ref::<Bindings>().unwrap();
                    let node = bindings.0[Id::from_raw(id)].0.downcast_ref::<BindingNode<T>>().unwrap();
                    let sources = node.sources.downcast_ref::< [< BindingExt $n NodeSources >] <P, $( [< S $i >] ,)* T>>().unwrap();
                    &sources.param
                }

                fn param_mut(arena: &mut dyn Any, id: RawId) -> &mut P {
                    let bindings = arena.downcast_mut::<Bindings>().unwrap();
                    let node = bindings.0[Id::from_raw(id)].0.downcast_mut::<BindingNode<T>>().unwrap();
                    let sources = node.sources.downcast_mut::< [< BindingExt $n NodeSources >] <P, $( [< S $i >] ,)* T>>().unwrap();
                    &mut sources.param
                }

                #[allow(dead_code)]
                const PARAM_DESCRIPTOR: ParamDescriptor<P> = ParamDescriptor {
                    arena: TypeId::of::<Bindings>(),
                    get_raw: Self::param_ref,
                    get_raw_mut: Self::param_mut,
                };

                pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
                    BindingBase::from(self).set_target(state, target);
                }

                pub fn set_holder(self, state: &mut dyn State, holder: Box<dyn Holder>) {
                    BindingBase::from(self).set_holder(state, holder);
                }

                pub fn dispatch<Context: Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T) -> Re<!>
                ) {
                    BindingBase::from(self).dispatch(state, context, execute);
                }

                pub fn set_target_fn<Context: Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T)
                ) {
                    BindingBase::from(self).set_target_fn(state, context, execute);
                }

                pub fn drop_self(self, state: &mut dyn State) {
                    AnyBindingBase::from(self).drop_self(state);
                }

                $(
                    pub fn [< set_source_ $i >] (self, state: &mut dyn State, source: &mut [< S $i >] ) {
                        let handler: [< BindingExt $n Source $i Handler >] ::<P, $( [< S $j >] ,)* T>  = [< BindingExt $n Source $i Handler >] {
                            binding: self.0,
                            phantom: PhantomType::new()
                        };
                        let source = source.handle(
                            state,
                            Box::new(handler)
                        );
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< BindingExt $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        if sources. [< source_ $i >] .replace((source.handler_id, [< S $i >] ::Cache::default() )).is_some() {
                            panic!("duplicate source");
                        }
                        source.init.map(|x| x(state));
                    }
                )*
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< BindingExt $n >] <P, $( [< S $i >] , )* T> > for BindingBase<T> {
                fn from(v: [< BindingExt $n >] <P, $( [< S $i >] , )* T> ) -> BindingBase<T> {
                    BindingBase(v.0, PhantomType::new())
                }
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< BindingExt $n >] <P, $( [< S $i >] , )* T> > for AnyBindingBase {
                fn from(v: [< BindingExt $n >] <P, $( [< S $i >] , )* T> ) -> AnyBindingBase {
                    AnyBindingBase(v.0)
                }
            }

            $(
                #[derive(Educe)]
                #[educe(Debug, Clone)]
                struct [< BindingExt $n Source $i Handler >] <
                    P,
                    $( [< S $j >] : Source, )*
                    T: Convenient
                > {
                    binding: Id<BoxedBindingNode>,
                    phantom: PhantomType<(P, $( [< S $j >] ,)* T)>
                }

                impl<
                    P: Debug + 'static,
                    $( [< S $j >] : Source + 'static, )*
                    T: Convenient
                > AnyHandler for [< BindingExt $n Source $i Handler >] <P, $( [< S $j >] , )* T >  {
                    fn clear(&self, state: &mut dyn State) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< BindingExt $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        sources. [< source_ $i >] .take();
                    }
                }

                impl<
                    P: Debug + 'static,
                    $( [< S $j >] : Source + 'static, )*
                    T: Convenient
                > Handler< [< S $i >] ::Value > for [< BindingExt $n Source $i Handler >] <P, $( [< S $j >] , )* T >  {
                    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler> {
                        self
                    }

                    fn execute(&self, state: &mut dyn State, value: [< S $i >] ::Value ) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< BindingExt $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        sources. [< source_ $i >] .as_mut().unwrap().1.update(value.clone());
                        $(
                            #[allow(unused_assignments, unused_mut)]
                            let mut [< current_ $j >] = None;
                        )*
                        [< current_ $i >] = Some(value);
                        $(
                            let [< value_ $j >] ;
                            if let Some(source) = sources. [< source_ $j >] .as_ref() {
                                if let Some(source) = source.1.get( [< current_ $j >] ) {
                                    [< value_ $j >] = source;
                                } else {
                                    return;
                                }
                            } else {
                                return;
                            }
                        )*

                        let target = node.target.clone();
                        let param = Param {
                            id: self.binding.into_raw(),
                            descriptor: & < [< BindingExt $n >] <P, $( [< S $j >] ,)* T> > ::PARAM_DESCRIPTOR
                        };
                        if let Re(Some(value)) = (sources.dispatch)(state, param, $( [< value_ $j >] ),*) {
                            target.map(|x| x.execute(state, value));
                        }
                    }
                }
            )*

            #[derive(Educe)]
            #[educe(Debug(bound="P: Debug"))]
            struct [< Binding $n NodeSources >] <P, $( [< S $i >] : Source, )* T: Convenient> {
                param: P,
                $(
                    [< source_ $i >] : Option<(Box<dyn HandlerId>, [< S $i >] ::Cache )>,
                )*
                #[educe(Debug(ignore))]
                filter_map: fn(P, $( < < [< S $i >] as Source > ::Cache as SourceCache< [< S $i >] ::Value > >::Value ),* ) -> Option<T>,
            }

            impl<
                P: Clone + 'static,
                $( [< S $i >] : Source + 'static, )*
                T: Convenient
            > AnyBindingNodeSources for [< Binding $n NodeSources >] <P, $( [< S $i >] , )* T> {
                type Value = T;

                fn is_empty(&self) -> bool {
                    $(
                        if self. [< source_ $i >] .is_some() {
                            return false;
                        }
                    )*
                    true
                }

                #[allow(unused_variables)]
                fn unhandle(&mut self, state: &mut dyn State, dropping_binding: AnyBindingBase) {
                    $(
                        if let Some(source) = self. [< source_ $i >] .take() {
                            source.0.unhandle(state, dropping_binding);
                        }
                    )*
                }

                fn get_value(&self) -> Option<T> {
                    $(
                        let [< value_ $i >] ;
                        if let Some(source) = self. [< source_ $i >] .as_ref() {
                            if let Some(source) = source.1.get(None) {
                                [< value_ $i >] = source;
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    )*
                    (self.filter_map)(self.param.clone(), $( [< value_ $i >] ),*)
                }
            }

            macro_attr! {
                #[derive(Educe, NewtypeComponentId!)]
                #[educe(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
                pub struct [< Binding $n >] <P, $( [< S $i >] : Source, )* T: Convenient>(
                    Id<BoxedBindingNode>,
                    PhantomType<(P, ($( [< S $i >] ,)* ), T)>
                );
            }

            impl<
                P: Clone + 'static,
                $( [< S $i >] : Source + 'static, )*
                T: Convenient
            > [< Binding $n >] <P, $( [< S $i >] , )* T> {
                pub fn new(
                    state: &mut dyn State,
                    param: P,
                    filter_map: fn(P, $( < < [< S $i >] as Source > ::Cache as SourceCache< [< S $i >] ::Value > >::Value ),* ) -> Option<T>,
                ) -> Self {
                    let bindings: &mut Bindings = state.get_mut();
                    let id = bindings.0.insert(|id| {
                        let sources: [< Binding $n NodeSources >] <P, $( [< S $i >] ,)* T> = [< Binding $n NodeSources >] {
                            param,
                            $(
                                [< source_ $i >] : None,
                            )*
                            filter_map,
                        };
                        let node: BindingNode<T> = BindingNode {
                            sources: Box::new(sources),
                            target: None,
                            holder: None,
                        };
                        (BoxedBindingNode(Box::new(node)), id)
                    });
                    [< Binding $n >] (id, PhantomType::new())
                }

                pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
                    BindingBase::from(self).set_target(state, target);
                }

                pub fn set_holder(self, state: &mut dyn State, holder: Box<dyn Holder>) {
                    BindingBase::from(self).set_holder(state, holder);
                }

                pub fn dispatch<Context: Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T) -> Re<!>
                ) {
                    BindingBase::from(self).dispatch(state, context, execute);
                }

                pub fn set_target_fn<Context: Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T)
                ) {
                    BindingBase::from(self).set_target_fn(state, context, execute);
                }

                pub fn drop_self(self, state: &mut dyn State) {
                    AnyBindingBase::from(self).drop_self(state);
                }

                pub fn get_value(self, state: &dyn State) -> Option<T> {
                    Binding::from(self).get_value(state)
                }

                $(
                    pub fn [< set_source_ $i >] (self, state: &mut dyn State, source: &mut [< S $i >] ) {
                        let handler: [< Binding $n Source $i Handler >] ::<P, $( [< S $j >] ,)* T>  = [< Binding $n Source $i Handler >] {
                            binding: self.0,
                            phantom: PhantomType::new()
                        };
                        let source = source.handle(
                            state,
                            Box::new(handler)
                        );
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< Binding $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        if sources. [< source_ $i >] .replace((source.handler_id, [< S $i >] ::Cache::default() )).is_some() {
                            panic!("duplicate source");
                        }
                        source.init.map(|x| x(state));
                    }
                )*
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< Binding $n >] <P, $( [< S $i >] , )* T> > for Binding<T> {
                fn from(v: [< Binding $n >] <P, $( [< S $i >] , )* T> ) -> Binding<T> {
                    Binding(v.0, PhantomType::new())
                }
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< Binding $n >] <P, $( [< S $i >] , )* T> > for BindingBase<T> {
                fn from(v: [< Binding $n >] <P, $( [< S $i >] , )* T> ) -> BindingBase<T> {
                    BindingBase(v.0, PhantomType::new())
                }
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< Binding $n >] <P, $( [< S $i >] , )* T> > for AnyBindingBase {
                fn from(v: [< Binding $n >] <P, $( [< S $i >] , )* T> ) -> AnyBindingBase {
                    AnyBindingBase(v.0)
                }
            }

            $(
                #[derive(Educe)]
                #[educe(Debug, Clone)]
                struct [< Binding $n Source $i Handler >] <
                    P,
                    $( [< S $j >] : Source, )*
                    T: Convenient
                > {
                    binding: Id<BoxedBindingNode>,
                    phantom: PhantomType<(P, $( [< S $j >] ,)* T)>
                }

                impl<
                    P: Clone + 'static,
                    $( [< S $j >] : Source + 'static, )*
                    T: Convenient
                > AnyHandler for [< Binding $n Source $i Handler >] <P, $( [< S $j >] , )* T >  {
                    fn clear(&self, state: &mut dyn State) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< Binding $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        sources. [< source_ $i >] .take();
                    }
                }

                impl<
                    P: Clone + 'static,
                    $( [< S $j >] : Source + 'static, )*
                    T: Convenient
                > Handler< [< S $i >] ::Value > for [< Binding $n Source $i Handler >] <P, $( [< S $j >] , )* T >  {
                    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler> {
                        self
                    }

                    fn execute(&self, state: &mut dyn State, value: [< S $i >] ::Value ) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< Binding $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        sources. [< source_ $i >] .as_mut().unwrap().1.update(value.clone());
                        $(
                            #[allow(unused_assignments, unused_mut)]
                            let mut [< current_ $j >] = None;
                        )*
                        [< current_ $i >] = Some(value);
                        $(
                            let [< value_ $j >] ;
                            if let Some(source) = sources. [< source_ $j >] .as_ref() {
                                if let Some(source) = source.1.get( [< current_ $j >] ) {
                                    [< value_ $j >] = source;
                                } else {
                                    return;
                                }
                            } else {
                                return;
                            }
                        )*

                        if let Some(value) = (sources.filter_map)(sources.param.clone(), $( [< value_ $j >] ),*) {
                            if let Some(target) = node.target.clone() {
                                target.execute(state, value);
                            }
                        }
                    }
                }
            )*
        }
    };
}

pub mod n {
    use crate::binding::*;

    binding_n!(0;);
    binding_n!(1; 1);
    binding_n!(2; 1, 2);
    binding_n!(3; 1, 2, 3);
    binding_n!(4; 1, 2, 3, 4);
    binding_n!(5; 1, 2, 3, 4, 5);
    binding_n!(6; 1, 2, 3, 4, 5, 6);
    binding_n!(7; 1, 2, 3, 4, 5, 6, 7);
    binding_n!(8; 1, 2, 3, 4, 5, 6, 7, 8);
    binding_n!(9; 1, 2, 3, 4, 5, 6, 7, 8, 9);
    binding_n!(10; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
    binding_n!(11; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
    binding_n!(12; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
    binding_n!(13; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
    binding_n!(14; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
    binding_n!(15; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
    binding_n!(16; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
}
