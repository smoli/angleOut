use bevy::prelude::{AlphaMode, Color, Component, Image, Material, Vec4};
use bevy::reflect::TypeUuid;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType};

#[derive(Component)]
pub struct CustomMaterialApplied;


// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
#[uniform(0, CustomMaterialUniform)]
pub struct CustomMaterial {
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
pub struct CustomMaterialUniform {
    pub color1: Vec4,
    pub color2: Vec4,
    pub time: f32

}


impl AsBindGroupShaderType<CustomMaterialUniform> for CustomMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<Image>) -> CustomMaterialUniform {
        CustomMaterialUniform {
            color1: self.color1.as_linear_rgba_f32().into(),
            color2: self.color2.as_linear_rgba_f32().into(),
            time: self.time
        }
    }
}


/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
