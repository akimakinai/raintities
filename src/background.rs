use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

pub const BACKGROUND_MATERIAL_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x0039459badf94126ac7c936336ebb5c0);

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            BACKGROUND_MATERIAL_SHADER_HANDLE,
            "background_material.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins(Material2dPlugin::<BackgroundMaterial>::default())
            .register_asset_reflect::<BackgroundMaterial>();
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
#[reflect(Default, Debug)]
#[uniform(0, BackgroundMaterialUniform)]
pub struct BackgroundMaterial {
    pub scroll: f32,
}

#[derive(Clone, Default, ShaderType)]
pub struct BackgroundMaterialUniform {
    pub scroll: f32,
}

impl From<&BackgroundMaterial> for BackgroundMaterialUniform {
    fn from(material: &BackgroundMaterial) -> Self {
        BackgroundMaterialUniform {
            scroll: material.scroll,
        }
    }
}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        BACKGROUND_MATERIAL_SHADER_HANDLE.into()
    }
}

pub type Background2dBundle = MaterialMesh2dBundle<BackgroundMaterial>;
