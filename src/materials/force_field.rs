use bevy::asset::Handle;
use bevy::math::Vec4;
use bevy::pbr::{AlphaMode, Material};
use bevy::prelude::{Color, Image, Vec3};
use bevy::reflect::TypeUuid;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType};

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "38d3a2e0-3d5b-44bc-8ec9-f9366eae6006"]
#[uniform(0, ForceFieldMaterialUniform)]
pub struct ForceFieldMaterial {
    pub color1: Color,
    pub hit_position: Vec3,
    pub hit_time: f32,
    pub time: f32,

    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,

    pub alpha_mode: AlphaMode,
}

impl Default for ForceFieldMaterial {
    fn default() -> Self {
        return ForceFieldMaterial {
            color1: Color::WHITE,
            hit_position:Vec3::ZERO,
            hit_time: 0.0,
            time: 0.0,
            color_texture: None,
            alpha_mode: AlphaMode::Blend,
        }
    }
}

#[derive(ShaderType, Clone, Default)]
pub struct ForceFieldMaterialUniform {
    pub color1: Vec4,
    pub hit_position: Vec3,
    hit_time: f32,
    pub time: f32,
}


impl AsBindGroupShaderType<ForceFieldMaterialUniform> for ForceFieldMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> ForceFieldMaterialUniform {
        ForceFieldMaterialUniform {
            color1: self.color1.as_linear_rgba_f32().into(),
            hit_position: self.hit_position,
            hit_time: self.hit_time,
            time: self.time,
        }
    }
}


/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for ForceFieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/force_field_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
