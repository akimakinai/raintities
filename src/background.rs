use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};
pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BackgroundMaterial>::default())
            .register_asset_reflect::<BackgroundMaterial>();
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
#[reflect(Default, Debug)]
#[uniform(0, BackgroundMaterialUniform)]
pub struct BackgroundMaterial {
    pub scroll: f32,
    pub alpha: f32,
}

#[derive(Clone, Default, ShaderType)]
pub struct BackgroundMaterialUniform {
    pub scroll: f32,
    pub alpha: f32,
    padding_1: u32,
    padding_2: u32,
}

impl From<&BackgroundMaterial> for BackgroundMaterialUniform {
    fn from(material: &BackgroundMaterial) -> Self {
        BackgroundMaterialUniform {
            scroll: material.scroll,
            alpha: material.alpha,
            padding_1: 0,
            padding_2: 0,
        }
    }
}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/background_material.wgsl".into()
    }
}

pub type Background2dBundle = MaterialMesh2dBundle<BackgroundMaterial>;
