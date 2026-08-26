use bevy::asset::{Asset, Handle};
use bevy::math::Vec4;
use bevy::pbr::Material;
use bevy::prelude::{AlphaMode, Color, Image};
use bevy::reflect::TypePath;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderType};
use bevy::render::texture::GpuImage;
use bevy::shader::ShaderRef;

use crate::materials::linear_rgba;

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, BlockMaterialUniform)]
pub struct BlockMaterial {
    pub color1: Color,
    pub color2: Color,
    pub damage: f32,
    pub time: f32,
    pub top_bottom_split: bool,

    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,

    pub alpha_mode: AlphaMode,
}

impl Default for BlockMaterial {
    fn default() -> Self {
        return BlockMaterial {
            color1: Color::WHITE,
            color2: Color::WHITE,
            damage: 0.0,
            time: 0.0,
            top_bottom_split: false,
            color_texture: None,
            alpha_mode: AlphaMode::Blend,
        }
    }
}

#[derive(ShaderType, Clone, Default)]
pub struct BlockMaterialUniform {
    pub color1: Vec4,
    pub color2: Vec4,
    pub damage: f32,
    pub time: f32,
    pub top_bottom_split: u32
}


impl AsBindGroupShaderType<BlockMaterialUniform> for BlockMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> BlockMaterialUniform {
        BlockMaterialUniform {
            color1: linear_rgba(self.color1),
            color2: linear_rgba(self.color2),
            damage: self.damage,
            time: self.time,
            top_bottom_split: if self.top_bottom_split { 1 } else { 0 }

        }
    }
}


/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for BlockMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/block_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
