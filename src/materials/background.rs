use bevy::asset::Asset;
use bevy::math::Vec4;
use bevy::pbr::Material;
use bevy::prelude::{AlphaMode, Color};
use bevy::reflect::TypePath;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderType};
use bevy::render::texture::GpuImage;
use bevy::shader::ShaderRef;

use crate::materials::linear_rgba;

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, BackgroundMaterialUniform)]
pub struct BackgroundMaterial {
    pub color1: Color,
    pub color2: Color,
    pub time: f32,

/*    #[texture(2)]
    #[sampler(3)]
    pub color_texture: Option<Handle<Image>>,
*/
    pub alpha_mode: AlphaMode,
}

#[derive(ShaderType, Clone, Default)]
pub struct BackgroundMaterialUniform {
    pub color1: Vec4,
    pub color2: Vec4,
    pub time: f32

}


impl AsBindGroupShaderType<BackgroundMaterialUniform> for BackgroundMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> BackgroundMaterialUniform {
        BackgroundMaterialUniform {
            color1: linear_rgba(self.color1),
            color2: linear_rgba(self.color2),
            time: self.time
        }
    }
}


/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/background_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
