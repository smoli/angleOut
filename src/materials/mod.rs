use bevy::prelude::{Color, Component, LinearRgba, Vec4};

pub mod block;
pub mod background;
pub mod arena;
pub mod force_field;

#[derive(Component)]
pub struct CustomMaterialApplied;

/// Colors are authored in sRGB but the shaders work in linear space.
pub(crate) fn linear_rgba(color: Color) -> Vec4 {
    let c: LinearRgba = color.into();
    Vec4::new(c.red, c.green, c.blue, c.alpha)
}
