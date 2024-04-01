pub mod display;
pub mod instance;
pub mod mesh;
pub mod model;
pub mod render_target;
pub mod state;
pub mod text;
pub mod texture;

mod renderer;

pub use display::*;
pub use mesh::MeshRef;
pub use render_target::*;
pub use renderer::*;
pub use state::{PipelineRef, RenderPass, RenderState};
pub use texture::*;

#[macro_export]
macro_rules! user_render_bindings {
    (@variants $name:ident [$($processed:tt)*] texture(depth) $variant:ident, $($rest:tt)*) => {
        user_render_bindings!{ @variants $name [
            $($processed)*
            $variant(&'a ::rust_game_engine::renderer::state::BoundTexture),
        ] $($rest)*}
    };
    (@variants $name:ident [$($processed:tt)*] texture $variant:ident, $($rest:tt)*) => {
        user_render_bindings!{ @variants $name [
            $($processed)*
            $variant(&'a ::rust_game_engine::renderer::state::BoundTexture),
        ] $($rest)*}
    };
    (@variants $name:ident [$($processed:tt)*] uniform $variant:ident: $tp:ty, $($rest:tt)*) => {
        user_render_bindings!{ @variants $name [
            $($processed)*
            $variant(&'a ::rust_game_engine::renderer::UniformBindGroup<$tp>),
        ] $($rest)*}
    };
    (@variants $name:ident [$($processed:tt)*] uniform $variant:ident, $($rest:tt)*) => {
        user_render_bindings!{ @variants $name [
            $($processed)*
            $variant(&'a ::rust_game_engine::renderer::UniformBindGroup<$variant>),
        ] $($rest)*}
    };
    (@variants $name:ident [$($processed:tt)*] direct $variant:ident($_e:expr, $tp:ty), $($rest:tt)*) => {
        user_render_bindings!{ @variants $name [
            $($processed)*
            $variant(&'a $tp),
        ] $($rest)*}
    };
    (@variants $name:ident [$($processed:tt)*]) => {
        enum $name<'a> {
            $($processed)*
        }
    };

    (@types [$($processed:tt)*] uniform $_:ident, $($rest:tt)*) => {
        user_render_bindings!{ @types [
            $($processed)*
            ::rust_game_engine::renderer::state::BindingType::Uniform,
        ] $($rest)* }
    };
    (@types [$($processed:tt)*] uniform $_v:ident: $_t:ty, $($rest:tt)*) => {
        user_render_bindings!{ @types [
            $($processed)*
            ::rust_game_engine::renderer::state::BindingType::Uniform,
        ] $($rest)* }
    };
    (@types [$($processed:tt)*] texture $_v:ident, $($rest:tt)*) => {
        user_render_bindings!{ @types [
            $($processed)*
            ::rust_game_engine::renderer::state::BindingType::Texture { depth: false },
        ] $($rest)* }
    };
    (@types [$($processed:tt)*] texture(depth) $_v:ident, $($rest:tt)*) => {
        user_render_bindings!{ @types [
            $($processed)*
            ::rust_game_engine::renderer::state::BindingType::Texture { depth: true },
        ] $($rest)* }
    };
    (@types [$($processed:tt)*] direct $_v:ident($e:expr, $_tp:ty), $($rest:tt)*) => {
        user_render_bindings!{ @types [
            $($processed)*
            ::rust_game_engine::renderer::state::BindingType::Direct($e),
        ] $($rest)* }
    };
    (@types [$($processed:tt)*]) => {
        vec![
            $($processed)*
        ]
    };

    (@slots $self:ident ($acc:expr) [$($processed:tt)*] uniform $v:ident, $($rest:tt)*) => {
        user_render_bindings!{ @slots $self ($acc + 1) [
            $($processed)*
            Self::$v(_) => ($acc),
        ] $($rest)* }
    };
    (@slots $self:ident ($acc:expr) [$($processed:tt)*] uniform $v:ident: $_t:ty, $($rest:tt)*) => {
        user_render_bindings!{ @slots $self ($acc + 1) [
            $($processed)*
            Self::$v(_) => ($acc),
        ] $($rest)* }
    };
    (@slots $self:ident ($acc:expr) [$($processed:tt)*] texture $v:ident, $($rest:tt)*) => {
        user_render_bindings!{ @slots $self ($acc + 1) [
            $($processed)*
            Self::$v(_) => ($acc),
        ] $($rest)* }
    };
    (@slots $self:ident ($acc:expr) [$($processed:tt)*] texture(depth) $v:ident, $($rest:tt)*) => {
        user_render_bindings!{ @slots $self ($acc + 1) [
            $($processed)*
            Self::$v(_) => ($acc),
        ] $($rest)* }
    };
    (@slots $self:ident ($acc:expr) [$($processed:tt)*] direct $v:ident($_e:expr, $_:ty), $($rest:tt)*) => {
        user_render_bindings!{ @slots $self ($acc + 1) [
            $($processed)*
            Self::$v(_) => ($acc),
        ] $($rest)* }
    };
    (@slots $self:ident ($_:expr) [$($processed:tt)*]) => {
        match $self {
            $($processed)*
        }
    };

    (@values $self:ident [$($processed:tt)*] uniform $v:ident, $($rest:tt)*) => {
        user_render_bindings!{ @values $self [
            $($processed)*
            Self::$v(v) => v.bind_group(),
        ] $($rest)* }
    };
    (@values $self:ident [$($processed:tt)*] uniform $v:ident: $_t:ty, $($rest:tt)*) => {
        user_render_bindings!{ @values $self [
            $($processed)*
            Self::$v(v) => v.bind_group(),
        ] $($rest)* }
    };
    (@values $self:ident [$($processed:tt)*] texture $v:ident, $($rest:tt)*) => {
        user_render_bindings!{ @values $self [
            $($processed)*
            Self::$v(v) => v.bind_group(),
        ] $($rest)* }
    };
    (@values $self:ident [$($processed:tt)*] texture(depth) $v:ident, $($rest:tt)*) => {
        user_render_bindings!{ @values $self [
            $($processed)*
            Self::$v(v) => v.bind_group(),
        ] $($rest)* }
    };
    (@values $self:ident [$($processed:tt)*] direct $v:ident($_:ty), $($rest:tt)*) => {
        user_render_bindings!{ @values $self [
            $($processed)*
            Self::$v(v) => v.bind_group(),
        ] $($rest)* }
    };
    (@values $self:ident [$($processed:tt)*]) => {
        match $self {
            $($processed)*
        }
    };

    ($name:ident { $($inner:tt)* }) => {
        user_render_bindings!{ @variants $name [] $($inner)* }

        impl ::rust_game_engine::renderer::state::Bindings for $name<'_> {
            fn types() -> Vec<::rust_game_engine::renderer::state::BindingType> {
                user_render_bindings!{ @types [] $($inner)* }
            }
        }

        impl ::rust_game_engine::renderer::state::BindingSlot for $name<'_> {
            fn slot(&self) -> u32 {
                user_render_bindings!{ @slots self (0) [] $($inner)* }
            }

            fn value(&self) -> &::std::sync::Arc<::wgpu::BindGroup> {
                user_render_bindings!{ @values self [] $($inner)* }
            }
        }
    };
}
