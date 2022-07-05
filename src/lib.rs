mod container;
mod context;
mod others;
mod response;
mod ui;
mod widget;
pub use container::*;
pub use context::*;
pub use egui;
pub use others::*;
pub use response::*;
use tealr::{
    mlu::{
        mlua::{self, Lua},
        TealData,
    },
    MluaTealDerive, TypeWalker,
};
pub use ui::*;
pub use widget::*;

#[derive(Clone, Default, MluaTealDerive)]
pub struct EguiProxy;
impl TealData for EguiProxy {}
pub fn register_egui_lua_bindings(lua: &Lua) -> Result<(), mlua::Error> {
    let egui_proxy = lua.create_table()?;

    egui_proxy.set("color32", lua.create_proxy::<Color32>()?)?;
    egui_proxy.set("ctx", lua.create_proxy::<Context>()?)?;
    egui_proxy.set("galley", lua.create_proxy::<Galley>()?)?;
    egui_proxy.set("response", lua.create_proxy::<Response>()?)?;
    egui_proxy.set("rich_text", lua.create_proxy::<RichText>()?)?;
    egui_proxy.set("ui_docs", lua.create_proxy::<Ui>()?)?;
    egui_proxy.set("widget_text", lua.create_proxy::<WidgetText>()?)?;
    lua.globals().set("Egui", egui_proxy)?;
    Ok(())
}

#[macro_export]
macro_rules! lua_registry_scoped_ui {
    ( $lua:expr, $from_ui:expr, |$ui:ident| $code:expr) => {{
        use crate::ui::Ui;
        $lua.scope(|scope| {
            let $ui = scope.create_nonstatic_userdata(Ui::from($from_ui))?;
            let response: MultiValue = $code?;
            $lua.create_registry_value(response.into_vec())
        })
    }};
}

#[macro_export]
macro_rules! lua_registry_scoped_ui_extract {
    ( $lua:expr, $from_ui:expr, |$ui:ident| $code:expr) => {{
        use crate::ui::Ui;
        let key = $lua
            .scope(|scope| {
                let $ui = scope
                    .create_nonstatic_userdata(Ui::from($from_ui))
                    .expect("failed to create non static userdata with Ui");
                let response: MultiValue = $code.expect("ui function returned error");
                $lua.create_registry_value(response.into_vec())
            })
            .expect("failed to get registry key");

        let value: Vec<Value> = $lua
            .registry_value(&key)
            .expect("failed to get registry value");
        $lua.remove_registry_value(key)
            .expect("failed to remove registry value");
        MultiValue::from_vec(value)
    }};
}

pub fn get_all_types() -> TypeWalker {
    tealr::TypeWalker::new()
        .process_type::<Ui>()
        .process_type::<Context>()
        .process_type::<Response>()
        .process_type::<Spacing>()
        .process_type::<Visuals>()
        .process_type::<TextStyle>()
        .process_type::<Painter>()
        .process_type::<Layout>()
        .process_type::<Rect>()
        .process_type::<LayerId>()
        .process_type::<Color32>()
        .process_type::<Id>()
        .process_type::<RichText>()
        .process_type::<WidgetText>()
        .process_type::<TextureId>()
        .process_type::<Vec2>()
        .process_type::<Sense>()
        .process_type::<Align>()
        .process_type::<Galley>()
        .process_type::<TextureHandle>()
        .process_type::<Style>()
}

#[macro_export]
macro_rules! from_impl {
    ($name:ident $etype:path) => {
        impl From<$name> for $etype {
            fn from(x: $name) -> Self {
                x.0
            }
        }
        impl From<&$name> for $etype {
            fn from(x: &$name) -> Self {
                x.clone().0
            }
        }
        impl From<$etype> for $name {
            fn from(x: $etype) -> Self {
                Self(x)
            }
        }
        impl From<&$etype> for $name {
            fn from(x: &$etype) -> Self {
                Self(x.clone())
            }
        }
    };
}

#[macro_export]
macro_rules! wrapper {
    ( $name:ident  $etype:path) => {
        #[derive(Clone, AsRef, AsMut, Deref, DerefMut, tealr::MluaTealDerive)]
        pub struct $name(pub $etype);

        $crate::from_impl!($name $etype);
    };
    ( copy $name:ident  $etype:path) => {
        #[derive(Clone, Copy, AsRef, AsMut, Deref, DerefMut, tealr::MluaTealDerive)]
        pub struct $name(pub $etype);

        $crate::from_impl!($name $etype);
    };
    ( default $name:ident  $etype:path) => {
        #[derive(Clone, Default, AsRef, AsMut, Deref, DerefMut, tealr::MluaTealDerive)]
        pub struct $name(pub $etype);

        $crate::from_impl!($name $etype);
    };
    ( copy default $name:ident  $etype:path) => {
        #[derive(Clone, Default, Copy, AsRef, AsMut, Deref, DerefMut, tealr::MluaTealDerive)]
        pub struct $name(pub $etype);

        $crate::from_impl!($name $etype);
    };

}
/// this macro can be used to do the recurring task of wrapping methods for lua
/// Args:
/// * $methods : the name of the `&mut T: UserDataMethods` struct we are given in the impl of `TealData` for `add_methods` function
/// * $method_id : the name of the method to call on `&Self` which we are wrapping for lua
/// * ($parameter_types) : the tupe of types that the function will take as arguments
#[macro_export]
macro_rules! add_method {
    ($methods:ident, $method_id:ident) => {
        $methods.add_method(stringify!($method_id), |_, self_ref, ()| {
            Ok(self_ref.$method_id())
        });
    };
    ($methods:ident, $method_id:ident, (), $ret_type:ty) => {
        $methods.add_method(stringify!($method_id), |_, self_ref, ()| {
            Ok(<$ret_type>::from(self_ref.$method_id()))
        });
    };
    ($methods:ident, $method_id:ident, $arg_type:ty) => {
        $methods.add_method(stringify!($method_id), |_, self_ref, a0: $arg_type| {
            Ok(self_ref.$method_id(a0.into()))
        });
    };
    ($methods:ident, $method_id:ident, $arg_type:ty ; $arg_type2:ty) => {
        $methods.add_method(
            stringify!($method_id),
            |_, self_ref, (a0, a1): ($arg_type, $arg_type2)| {
                Ok(self_ref.$method_id(a0.into(), a1.into()))
            },
        );
    };
    ($methods:ident, $method_id:ident, $arg_type:ty, $ret_type:ty) => {
        $methods.add_method(stringify!($method_id), |_, self_ref, a0: $arg_type| {
            Ok(<$ret_type>::from(self_ref.$method_id(a0.into())))
        });
    };
    ($methods:ident, $method_id:ident, $arg_type:ty, $ret_type:ty ; $ret_type2:ty) => {
        $methods.add_method(stringify!($method_id), |_, self_ref, a0: $arg_type| {
            let result = self_ref.$method_id(a0.into());
            Ok((<$ret_type>::from(result.0), <$ret_type2>::from(result.1)))
        });
    };
    ($methods:ident, $method_id:ident, $arg_type:ty ; $arg_type2:ty, $ret_type:ty) => {
        $methods.add_method(
            stringify!($method_id),
            |_, self_ref, (a0, a1): ($arg_type, $arg_type2)| {
                Ok(<$ret_type>::from(self_ref.$method_id(a0.into(), a1.into())))
            },
        );
    };
    ($methods:ident, $method_id:ident, $arg_type:ty ; $arg_type2:ty, $ret_type:ty ; $ret_type2:ty) => {
        $methods.add_method(
            stringify!($method_id),
            |_, self_ref, (a0, a1): ($arg_type, $arg_type2)| {
                let result = self_ref.$method_id(a0.into(), a1.into());

                Ok((<$ret_type>::from(result.0), <$ret_type2>::from(result.1)))
            },
        );
    };
    ($methods:ident, $method_id:ident, $arg_type:ty ; $arg_type2:ty ; $arg_type3:ty, $ret_type:ty) => {
        $methods.add_method(
            stringify!($method_id),
            |_, self_ref, (a0, a1, a2): ($arg_type, $arg_type2, $arg_type3)| {
                Ok(<$ret_type>::from(self_ref.$method_id(
                    a0.into(),
                    a1.into(),
                    a2.into(),
                )))
            },
        );
    };
}
#[macro_export]
macro_rules! add_method_mut {
    ($methods:ident, $method_id:ident) => {
        $methods.add_method_mut(stringify!($method_id), |_, self_ref, ()| {
            Ok(self_ref.$method_id())
        });
    };
    ($methods:ident, $method_id:ident, (), $ret_type:ty) => {
        $methods.add_method_mut(stringify!($method_id), |_, self_ref, ()| {
            Ok(<$ret_type>::from(self_ref.$method_id()))
        });
    };
    ($methods:ident, $method_id:ident, $arg_type:ty) => {
        $methods.add_method_mut(stringify!($method_id), |_, self_ref, a0: $arg_type| {
            self_ref.$method_id(a0.into());
            Ok(())
        });
    };
    ($methods:ident, $method_id:ident,  $arg_type:ty,  $ret_type:ty) => {
        $methods.add_method_mut(stringify!($method_id), |_, self_ref, a0: $arg_type| {
            Ok(<$ret_type>::from(self_ref.$method_id(a0.into())))
        });
    };
    ($methods:ident, $method_id:ident, $arg_type:ty, $ret_type:ty ; $ret_type2:ty) => {
        $methods.add_method_mut(stringify!($method_id), |_, self_ref, a0: $arg_type| {
            let result = self_ref.$method_id(a0.into());
            Ok((<$ret_type>::from(result.0), <$ret_type2>::from(result.1)))
        });
    };
    ($methods:ident, $method_id:ident, $arg_type:ty ; $arg_type2:ty, $ret_type:ty) => {
        $methods.add_method_mut(
            stringify!($method_id),
            |_, self_ref, (a0, a1): ($arg_type, $arg_type2)| {
                Ok(<$ret_type>::from(self_ref.$method_id(a0.into(), a1.into())))
            },
        );
    };
    ($methods:ident, $method_id:ident, $arg_type:ty ; $arg_type2:ty, $ret_type:ty ; $ret_type2:ty) => {
        $methods.add_method_mut(
            stringify!($method_id),
            |_, self_ref, (a0, a1): ($arg_type, $arg_type2)| {
                let result = self_ref.$method_id(a0.into(), a1.into());

                Ok((<$ret_type>::from(result.0), <$ret_type2>::from(result.1)))
            },
        );
    };
    ($methods:ident, $method_id:ident, $arg_type:ty ; $arg_type2:ty ; $arg_type3:ty, $ret_type:ty) => {
        $methods.add_method_mut(
            stringify!($method_id),
            |_, self_ref, (a0, a1, a2): ($arg_type, $arg_type2, $arg_type3)| {
                Ok(<$ret_type>::from(self_ref.$method_id(
                    a0.into(),
                    a1.into(),
                    a2.into(),
                )))
            },
        );
    };
}
