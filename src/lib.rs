#![feature(const_mut_refs)]
#![feature(const_ptr_offset_from)]
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

mod fw;
pub use fw::*;

pub mod binding;
pub mod templates;

#[cfg(docsrs)]
pub mod example {
    //! The [`dep_type`] and [`dep_obj`] macro expansion example.
    //!
    //! ```ignore
    //! dep_type! {
    //!     #[derive(Debug)]
    //!     pub struct MyDepType in MyDepTypeId {
    //!         prop_1: bool = false,
    //!         prop_2: i32 = 10,
    //!     }
    //! }
    //!
    //! macro_attr! {
    //!     #[derive(Component!, Debug)]
    //!     struct MyDepTypePrivateData {
    //!         dep_data: MyDepType,
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
    //! macro_attr! {
    //!     #[derive(State!, Debug)]
    //!     pub struct MyApp {
    //!         my_dep_types: Arena<MyDepTypePrivateData>,
    //!     }
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
    //!     pub fn drop_my_dep_type(self, state: &mut dyn State) {
    //!         self.drop_bindings_priv(state);
    //!         let app: &mut MyApp = state.get_mut();
    //!         app.my_dep_types.remove(self.0);
    //!     }
    //!
    //!     dep_obj! {
    //!         pub fn obj(self as this, app: MyApp) -> (MyDepType) {
    //!             if mut {
    //!                 &mut app.my_dep_types[this.0].dep_data
    //!             } else {
    //!                 &app.my_dep_types[this.0].dep_data
    //!             }
    //!         }
    //!     }
    //! }

    use crate::{DetachedDepObjId, dep_obj, dep_type};
    use components_arena::{Arena, Component, Id, NewtypeComponentId};
    use dyn_context::state::{SelfState, State, StateExt};

    dep_type! {
        #[derive(Debug)]
        pub struct MyDepType in MyDepTypeId {
            prop_1: bool = false,
            prop_2: i32 = 10,
        }
    }

    #[derive(Debug)]
    struct MyDepTypePrivateData {
        dep_data: MyDepType,
    }

    Component!(() struct MyDepTypePrivateData { .. });

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct MyDepTypeId(Id<MyDepTypePrivateData>);

    NewtypeComponentId!(() pub struct MyDepTypeId(Id<MyDepTypePrivateData>););

    impl DetachedDepObjId for MyDepTypeId { }

    #[derive(Debug)]
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

        pub fn drop_my_dep_type(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let app: &mut MyApp = state.get_mut();
            app.my_dep_types.remove(self.0);
        }

        dep_obj! {
            pub fn obj(self as this, app: MyApp) -> (MyDepType) {
                if mut {
                    &mut app.my_dep_types[this.0].dep_data
                } else {
                    &app.my_dep_types[this.0].dep_data
                }
            }
        }
    }
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
pub use dyn_context::state::State as dyn_context_state_State;
#[doc(hidden)]
pub use dyn_context::state::StateExt as dyn_context_state_StateExt;
#[doc(hidden)]
pub use generics::concat as generics_concat;
#[doc(hidden)]
pub use generics::parse as generics_parse;
#[doc(hidden)]
pub use memoffset::offset_of as memoffset_offset_of;
#[doc(hidden)]
pub use paste::paste as paste_paste;

#[macro_export]
macro_rules! dep_type_with_builder {
    (
        $($token:tt)*
    ) => {
        $crate::dep_type_with_builder_impl! { $($token)* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_type_with_builder_impl {
    (
        type BaseBuilder $($token:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @type BaseBuilder
            }
        }
        $($token)*
    };
    (
        @type BaseBuilder
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BaseBuilder:ty;

        $(#[$attr:meta])* $vis:vis struct $name:ident $($body:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @struct
                [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
                [$([$attr])*] [$vis] [$name]
            }
            $($body)*
        }
    };
    (
        @type BaseBuilder
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BaseBuilder:ty;

        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition; allowed form is \
            '$(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? \
            become $obj in $Id as $Dyn { ... }'\
        ");
    };
    (
        @type BaseBuilder
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type base builder definition; allowed form is \
            'type BaseBuilder $(<$generics> $($where_clause)?)? = $base_builder_type;\
        ");
    };
    (
        $(#[$attr:meta])* $vis:vis struct $name:ident $($body:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @struct
                []
                [$([$attr])*] [$vis] [$name]
            }
            $($body)*
        }
    };
    (
        @struct
        [[$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty as $Dyn:ty
        {
            $($($(#[$inherits:tt])* $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
    ) => {
        $crate::dep_type_with_builder_impl! {
            @concat_generics
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn]
            [$($g)*] [$($r)*] [$($w)*]
            [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
            [$($([[$($inherits)*] $field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @struct
        [[$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty as $Dyn:ty
        {
            $($($(#[$inherits:tt])* $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
        $($token:tt)+
    ) => {
        $crate::std_compile_error!("unexpected extra tokens after dep type definition body");
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty as $Dyn:ty
        {
            $($($(#[$inherits:tt])* $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }

        type BaseBuilder $($token:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @type BaseBuilder after
                [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn]
                [$($g)*] [$($r)*] [$($w)*]
                [$($([[$($inherits)*] $field $delim $($field_ty $(= $field_val)?)?])+)?]
            }
            $($token)*
        }
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty as $Dyn:ty
        {
            $($($(#[$inherits:tt])* $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
    ) => {
        $crate::std_compile_error!("\
            missing dep type base builder definition; add the definition in the following form \
            before or after dep type definition: \
            'type BaseBuilder $(<$generics> $($where_clause)?)? = $base_builder_type;\
        ");
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty as $Dyn:ty
        {
            $($($(#[$inherits:tt])* $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }

        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type base builder definition; allowed form is \
            'type BaseBuilder $(<$generics> $(where $where_clause)?)? = $base_builder_type;
        ");
    };
    (
        @struct
        [$([$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*])?]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? become $obj in $Id as $Dyn {\n\
                $(#[$field_1_attr])* $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $(#[$field_2_attr])* $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
    (
        @type BaseBuilder after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([[$($inherits:tt)*] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BaseBuilder:ty;
    ) => {
        $crate::dep_type_with_builder_impl! {
            @concat_generics
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn]
            [$($g)*] [$($r)*] [$($w)*]
            [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
            [$($([[$($inherits)*] $field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @type BaseBuilder after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([[$($inherits:tt)*] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BaseBuilder:ty;

        $($token:tt)*
    ) => {
        $crate::std_compile_error!("unexpected extra tokens after dep type base builder definition");
    };
    (
        @type BaseBuilder after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([[$($inherits:tt)*] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type base builder definition; allowed form is \
            'type BaseBuilder $(<$generics> $(where $where_clause)?)? = $base_builder_type;
        ");
    };
    (
        @concat_generics
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [[$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
        [$([[$($inherits:tt)*] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
    ) => {
        $crate::generics_concat! {
            $crate::dep_type_with_builder_impl {
                @concat_generics_done
                [$BaseBuilder]
                [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn]
                [$($g)*] [$($r)*] [$($w)*]
                [$([[$($inherits)*] $field $delim $($field_ty $(= $field_val)?)?])*]
            }
            [$($g)*] [$($r)*] [$($w)*],
            [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
        }
    };
    (
        @concat_generics_done
        [$BaseBuilder:ty]
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$([[$($inherits:tt)*] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn] [state] [this] [bindings] [handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] [] [] [] []
            [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*] []]
            [$([[$($inherits)*] $field $delim $($field_ty $(= $field_val)?)?])*]
        }
    };
    (
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? become $obj in $Id as $Dyn {\n\
                $(#[$field_1_attr])* $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $(#[$field_2_attr])* $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
}

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
                []
                [$([$attr])*] [$vis] [$name]
            }
            $($body)*
        }
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        in $Id:ty as $Dyn:ty
        {
            $($($(#[$inherits:tt])* $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [obj] [$Id] [$Dyn] [state] [this] [bindings] [handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] [] [] [] []
            []
            [$($([[$($inherits)*] $field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        in $Id:ty as $Dyn:ty
        {
            $($($(#[$inherits:tt])* $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
        $($token:tt)+
    ) => {
        $crate::std_compile_error!("unexpected extra tokens after dep type definition body");
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? in $Id as $Dyn {\n\
                $(#[$field_1_attr])* $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $(#[$field_2_attr])* $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
    (
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? in $Id as $Dyn {\n\
                $(#[$field_1_attr])* $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $(#[$field_2_attr])* $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_type_impl_raw {
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[ref inherits] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn]
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
                $name:: [< $field:upper >] .update_parent_children_has_handlers($state, $obj);
            ]
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*

                    $vis fn [< $field _ref >] (mut self, value: $field_ty) -> Self {
                        let id = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::id(&self.base);
                        let state = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::state_mut(&mut self.base);
                        $name:: [< $field:upper >] .set(state, id.$obj(), value).immediate();
                        self
                    }
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[inherits ref] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn] [$state] [$this] [$bindings] [$handlers]
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
                $name:: [< $field:upper >] .update_parent_children_has_handlers($state, $obj);
            ]
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*

                    $vis fn [< $field _ref >] (mut self, value: $field_ty) -> Self {
                        let id = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::id(&self.base);
                        let state = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::state_mut(&mut self.base);
                        $name:: [< $field:upper >] .set(state, id.$obj(), value).immediate();
                        self
                    }
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[inherits] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn] [$state] [$this] [$bindings] [$handlers]
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
                $name:: [< $field:upper >] .update_parent_children_has_handlers($state, $obj);
            ]
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*

                    $vis fn $field(mut self, value: $field_ty) -> Self {
                        let id = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::id(&self.base);
                        let state = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::state_mut(&mut self.base);
                        $name:: [< $field:upper >] .set(state, id.$obj(), value).immediate();
                        self
                    }
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[ref] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn] [$state] [$this] [$bindings] [$handlers]
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
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*

                    $vis fn [< $field _ref >] (mut self, value: $field_ty) -> Self {
                        let id = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::id(&self.base);
                        let state = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::state_mut(&mut self.base);
                        $name:: [< $field:upper >] .set(state, id.$obj(), value).immediate();
                        self
                    }
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn] [$state] [$this] [$bindings] [$handlers]
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
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*

                    $vis fn $field(mut self, value: $field_ty) -> Self {
                        let id = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::id(&self.base);
                        let state = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::state_mut(&mut self.base);
                        $name:: [< $field:upper >] .set(state, id.$obj(), value).immediate();
                        self
                    }
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[$($inherits:tt)*] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dep type property attributes: '",
            $crate::std_stringify!($(#[$inherits])*),
            "; allowed attributes are: '#[inherits]', '#[ref]'"
        ));
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[bubble] $field:ident yield $field_ty:ty] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn] [$state] [$this] [$bindings] [$handlers]
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
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[] $field:ident yield $field_ty:ty] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn] [$state] [$this] [$bindings] [$handlers]
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
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[$($inherits:tt)*] $field:ident yield $field_ty:ty] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dep type event attributes: '",
            $crate::std_stringify!($(#[$inherits])*),
            "; allowed attributes are: '#[bubble]'"
        ));
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[] $field:ident [$field_ty:ty]] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$Dyn] [$state] [$this] [$bindings] [$handlers]
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
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
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
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[$($inherits:tt)*] $field:ident $delim:tt $field_ty:ty $(= $field_val:expr)?] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!("\
            invalid dep type field definition\n\
            \n\
        ",
            $crate::std_stringify!($(#[$inherits])? $field $delim $field_ty $(= $field_val)?),
        "\
            \n\n\
            allowed forms are \
            '$(#[$field_attr])* $field_name : $field_type = $field_value', \
            '$field_name [$field_type]', and \
            '$(#[$field_attr])* $field_name yield $field_type'\
        "));
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$Dyn:ty]
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
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
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

            impl $($g)* $crate::NewPriv for $name $($r)* $($w)* {
                fn new_priv() -> Self {
                    Self::new_priv()
                }
            }

            impl $($g)* $name $($r)* $($w)* {
                const fn new_priv() -> Self {
                    Self { core: [< $name Core >] ::new() }
                }

                $($dep_props)*
            }

            impl $($g)* $crate::DepType for $name $($r)* $($w)* {
                type Id = $Id;
                type Dyn = $Dyn;

                #[doc(hidden)]
                fn core_base_priv(&self) -> &$crate::BaseDepObjCore<$name $($r)*> {
                    &self.core.dep_type_core_base
                }

                #[doc(hidden)]
                fn core_base_priv_mut(&mut self) -> &mut $crate::BaseDepObjCore<$name $($r)*> {
                    &mut self.core.dep_type_core_base
                }

                #[doc(hidden)]
                fn take_all_handlers(&mut self) -> $crate::std_vec_Vec<$crate::std_boxed_Box<dyn $crate::binding::AnyHandler>> {
                    self.core.dep_type_core_take_all_handlers()
                }

                #[doc(hidden)]
                fn collect_all_bindings(&self) -> $crate::std_vec_Vec<$crate::binding::AnyBindingBase> {
                    self.core.dep_type_core_collect_all_bindings()
                }

                #[doc(hidden)]
                #[allow(unused_variables)]
                fn update_parent_children_has_handlers($state: &mut dyn $crate::dyn_context_state_State, $obj: $crate::Glob < $name $($r)* >) where Self: Sized {
                    $($update_handlers)*
                }
            }

            $(
                $vis struct [< $name Builder >] $($bc_g)* $($bc_w)* {
                    base: $BaseBuilder,
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
            )?
        }
    };
}

#[macro_export]
macro_rules! dep_obj {
    (
        impl $($token:tt)+
    ) => {
        $crate::generics_parse! {
            $crate::dep_obj_impl {
                @impl
            }
            $($token)+
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_obj_drop_bindings {
    (
        @impl [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] $t:ty {
            $(
                $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> $(optional(trait $opt_tr:tt))? $((trait $tr:tt))? $(optional($opt_ty:ty))? $(($ty:ty))? as $Dyn:ty {
                    if mut { $field_mut:expr } else { $field:expr }
                }
            )*
        }
    ) => {
        impl $($g)* $t $($w)* {
            fn drop_bindings_priv(self, state: &mut dyn $crate::dyn_context_state_State) {
                $(
                    let $this = self;
                    let $arena: &mut $Arena = <dyn $crate::dyn_context_state_State as $crate::dyn_context_state_StateExt>::get_mut(state);
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
                    let $arena: &mut $Arena = <dyn $crate::dyn_context_state_State as $crate::dyn_context_state_StateExt>::get_mut(state);
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
                            <$ty as $crate::DepType>::update_parent_children_has_handlers(state, self.$name());
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

#[doc(hidden)]
#[macro_export]
macro_rules! dep_obj_impl {
    (
        @impl $g:tt $r:tt $w:tt $t:ty {
            $(
                $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> $(optional(trait $opt_tr:tt))? $((trait $tr:tt))? $(optional($opt_ty:ty))? $(($ty:ty))? as $Dyn:ty {
                    if mut { $field_mut:expr } else { $field:expr }
                }
            )*
        }
    ) => {
        $crate::dep_obj_drop_bindings! {
            @impl $g $r $w $t {
                $(
                    $vis fn $name (self as $this, $arena: $Arena) -> $(optional(trait $opt_tr))? $((trait $tr))? $(optional($opt_ty))? $(($ty))? as $Dyn{
                        if mut { $field_mut} else { $field}
                    }
                )*
            }
        }
        $(
            $crate::dep_obj_impl_raw! {
                $g $w [$t]
                $vis fn $name (self as $this, $arena : $Arena) -> $(optional(trait $opt_tr))? $((trait $tr))? $(optional($opt_ty))? $(($ty))? as $Dyn {
                    if mut { $field_mut } else { $field }
                }
            }
        )*
    };
    (
        @impl [$($g:tt)*] [$($r:tt)*] [$($w:tt)*] $($body:tt)*
    ) => {
        $crate::std_compile_error!($crate::std_concat!("\
            invalid dep obj implementation, allowed form is\n\
            \n\
            impl $generics $name $(where $where_clause)? {\n\
                $(\n\
                    $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> $(optional(trait $opt_tr:tt))? $((trait $tr:tt))? $(optional($opt_ty:ty))? $(($ty:ty))? as $Dyn:ty {\n\
                        if mut { $field_mut:expr } else { $field:expr }\n\
                    }\n\
                )*\n\
            }\n\
        "));
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_obj_impl_raw {
    (
        [$($g:tt)*] [$($w:tt)*] [$t:ty]
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> optional(trait $ty:tt) as $Dyn:ty {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            impl $($g)* $t $($w)* {
                fn [< $name _ref >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                    $arena: &'arena_lifetime dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'arena_lifetime DepObjType {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $arena = $arena.downcast_ref::<$Arena>().expect("invalid arena cast");
                    ($field)
                        .expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                        .downcast_ref::<DepObjType>().expect("invalid cast")
                }

                fn [< $name _mut >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                    $arena: &'arena_lifetime mut dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'arena_lifetime mut DepObjType {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $arena = $arena.downcast_mut::<$Arena>().expect("invalid arena cast");
                    ($field_mut)
                        .expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                        .downcast_mut::<DepObjType>().expect("invalid cast")
                }

                $vis fn [< $name _descriptor >] <DepObjType: $ty + $crate::DepType<Id=Self>>(
                ) -> $crate::GlobDescriptor<DepObjType> {
                    $crate::GlobDescriptor {
                        arena: $crate::std_any_TypeId::of::<$Arena>(),
                        field_ref: Self:: [< $name _ref >] ,
                        field_mut: Self:: [< $name _mut >] ,
                    }
                }

                $vis fn $name <DepObjType: $ty + $crate::DepType<Id=Self>>(
                    self
                ) -> $crate::Glob<DepObjType> {
                    $crate::Glob {
                        id: <Self as $crate::components_arena_ComponentId>::into_raw(self),
                        descriptor: Self:: [< $name _descriptor >]
                    }
                }
            }
        }
    };
    (
        [$($g:tt)*] [$($w:tt)*] [$t:ty]
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> (trait $ty:tt) as $Dyn:ty {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            impl $($g)* $t $($w)* {
                fn [< $name _ref >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                    $arena: &'arena_lifetime dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'arena_lifetime DepObjType {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $arena = $arena.downcast_ref::<$Arena>().expect("invalid arena cast");
                    ($field).downcast_ref::<DepObjType>().expect("invalid cast")
                }

                fn [< $name _mut >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                    $arena: &'arena_lifetime mut dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'arena_lifetime mut DepObjType {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $arena = $arena.downcast_mut::<$Arena>().expect("invalid arena cast");
                    ($field_mut).downcast_mut::<DepObjType>().expect("invalid cast")
                }

                $vis fn [< $name _descriptor >] <DepObjType: $ty + $crate::DepType<Id=Self>>(
                ) -> $crate::GlobDescriptor<DepObjType> {
                    $crate::GlobDescriptor {
                        arena: $crate::std_any_TypeId::of::<$Arena>(),
                        field_ref: Self:: [< $name _ref >] ,
                        field_mut: Self:: [< $name _mut >] ,
                    }
                }

                $vis fn $name <DepObjType: $ty + $crate::DepType<Id=Self>>(
                    self
                ) -> $crate::Glob<DepObjType> {
                    $crate::Glob {
                        id: <Self as $crate::components_arena_ComponentId>::into_raw(self),
                        descriptor: Self:: [< $name _descriptor >]
                    }
                }
            }
        }
    };
    (
        [$($g:tt)*] [$($w:tt)*] [$t:ty]
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident: $Arena:ty) -> optional($ty:ty) as $Dyn:ty {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            impl $($g)* $t $($w)* {
                fn [< $name _ref >] <'arena_lifetime>(
                    $arena: &'arena_lifetime dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'arena_lifetime $ty {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $arena = $arena.downcast_ref::<$Arena>().expect("invalid arena cast");
                    ($field).expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                }

                fn [< $name _mut >] <'arena_lifetime>(
                    $arena: &'arena_lifetime mut dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'arena_lifetime mut $ty {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $arena = $arena.downcast_mut::<$Arena>().expect("invalid arena cast");
                    ($field_mut).expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                }

                $vis fn [< $name _descriptor >] () -> $crate::GlobDescriptor<$ty> {
                    $crate::GlobDescriptor {
                        arena: $crate::std_any_TypeId::of::<$Arena>(),
                        field_ref: Self:: [< $name _ref >] ,
                        field_mut: Self:: [< $name _mut >] ,
                    }
                }

                $vis fn $name (
                    self
                ) -> $crate::Glob<$ty> {
                    $crate::Glob {
                        id: <Self as $crate::components_arena_ComponentId>::into_raw(self),
                        descriptor: Self:: [< $name _descriptor >]
                    }
                }
            }
        }
    };
    (
        [$($g:tt)*] [$($w:tt)*] [$t:ty]
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident: $Arena:ty) -> ($ty:ty) as $Dyn:ty {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            impl $($g)* $t $($w)* {
                fn [< $name _ref >] <'arena_lifetime>(
                    $arena: &'arena_lifetime dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'arena_lifetime $ty {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $arena = $arena.downcast_ref::<$Arena>().expect("invalid arena cast");
                    $field
                }

                fn [< $name _mut >] <'arena_lifetime>(
                    $arena: &'arena_lifetime mut dyn $crate::std_any_Any,
                    $this: $crate::components_arena_RawId,
                ) -> &'arena_lifetime mut $ty {
                    let $this = <Self as $crate::components_arena_ComponentId>::from_raw($this);
                    let $arena = $arena.downcast_mut::<$Arena>().expect("invalid arena cast");
                    $field_mut
                }

                $vis fn [< $name _descriptor >] () -> $crate::GlobDescriptor<$ty> {
                    $crate::GlobDescriptor {
                        arena: $crate::std_any_TypeId::of::<$Arena>(),
                        field_ref: Self:: [< $name _ref >] ,
                        field_mut: Self:: [< $name _mut >] ,
                    }
                }

                $vis fn $name (
                    self
                ) -> $crate::Glob<$ty> {
                    $crate::Glob {
                        id: <Self as $crate::components_arena_ComponentId>::into_raw(self),
                        descriptor: Self:: [< $name _descriptor >]
                    }
                }
            }
        }
    };
    (
        [$($g:tt)*] [$($w:tt)*] [$t:ty]
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> $(optional(trait $opt_tr:tt))? $(trait $tr:tt)? $(optional($opt_ty:ty))? $($ty:ty)? as $Dyn:ty {
        }
    ) => {
        $crate::std_compile_error!($crate::std_concat!("\
            invalid dep obj return type\n\
            \n\
        ",
            $crate::std_stringify!($(dyn $tr)? $($ty)?),
        "\
            \n\n\
            allowed form are \
            '$ty:ty', \
            'trait $trait:tt', \
            'optional($ty:ty)', and \
            'optional(trait $trait:tt)'\
        "));
    };
}
