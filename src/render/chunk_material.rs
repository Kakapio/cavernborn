use bevy::asset::AssetId;
use bevy::asset::{load_internal_asset, Asset, AssetApp, Assets, Handle};
use bevy::color::{Color, LinearRgba};
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::{render_asset::RenderAssets, render_resource::*, texture::GpuImage};
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};

use crate::world::chunk::CHUNK_SIZE;

pub const CHUNK_MATERIAL_SHADER_HANDLE: Handle<Shader> = Handle::Weak(AssetId::Uuid {
    uuid: uuid::uuid!("6b97a3bd-ab32-45a2-9e87-b20bab5d5878"),
});

pub const INDICE_BUFFER_SIZE: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;

#[derive(Default)]
pub struct ChunkMaterialPlugin;

impl Plugin for ChunkMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            CHUNK_MATERIAL_SHADER_HANDLE,
            "..\\..\\assets\\shaders\\chunk_material.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(Material2dPlugin::<ChunkMaterial>::default())
            .register_asset_reflect::<ChunkMaterial>();

        // Initialize the default material handle.
        app.world_mut()
            .resource_mut::<Assets<ChunkMaterial>>()
            .insert(
                &Handle::<ChunkMaterial>::default(),
                ChunkMaterial {
                    color: Color::srgb(1.0, 0.0, 1.0),
                    ..Default::default()
                },
            );
    }
}

/// A [2d material](Material2d) that renders [2d meshes](crate::Mesh2d) with a texture tinted by a uniform color
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[reflect(Default, Debug)]
#[uniform(0, ChunkMaterialUniform)]
pub struct ChunkMaterial {
    pub color: Color,
    pub alpha_mode: AlphaMode2d,
    pub uv_transform: Affine2,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    #[uniform(3)]
    pub indices: [UVec4; INDICE_BUFFER_SIZE / 4],
}

impl ChunkMaterial {
    pub fn from_indices(texture: Handle<Image>, indices: [UVec4; INDICE_BUFFER_SIZE / 4]) -> Self {
        Self {
            color: Color::WHITE,
            alpha_mode: AlphaMode2d::Opaque,
            uv_transform: Affine2::default(),
            texture: Some(texture),
            indices,
        }
    }
}

impl Default for ChunkMaterial {
    fn default() -> Self {
        ChunkMaterial {
            color: Color::WHITE,
            // TODO should probably default to AlphaMask once supported?
            alpha_mode: AlphaMode2d::Blend,
            uv_transform: Affine2::default(),
            texture: None,
            indices: [UVec4::ZERO; INDICE_BUFFER_SIZE / 4],
        }
    }
}

impl From<Color> for ChunkMaterial {
    fn from(color: Color) -> Self {
        ChunkMaterial {
            color,
            alpha_mode: if color.alpha() < 1.0 {
                AlphaMode2d::Blend
            } else {
                AlphaMode2d::Opaque
            },
            ..Default::default()
        }
    }
}

impl From<Handle<Image>> for ChunkMaterial {
    fn from(texture: Handle<Image>) -> Self {
        ChunkMaterial {
            texture: Some(texture),
            ..Default::default()
        }
    }
}

// NOTE: These must match the bit flags in bevy_sprite/src/mesh2d/color_material.wgsl!
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ChunkMaterialFlags: u32 {
        const TEXTURE                    = 1 << 0;
        /// Bitmask reserving bits for the [`AlphaMode2d`]
        /// Values are just sequential values bitshifted into
        /// the bitmask, and can range from 0 to 3.
        const ALPHA_MODE_RESERVED_BITS   = Self::ALPHA_MODE_MASK_BITS << Self::ALPHA_MODE_SHIFT_BITS;
        const ALPHA_MODE_OPAQUE          = 0 << Self::ALPHA_MODE_SHIFT_BITS;
        const ALPHA_MODE_MASK            = 1 << Self::ALPHA_MODE_SHIFT_BITS;
        const ALPHA_MODE_BLEND           = 2 << Self::ALPHA_MODE_SHIFT_BITS;
        const NONE                       = 0;
        const UNINITIALIZED              = 0xFFFF;
    }
}

impl ChunkMaterialFlags {
    const ALPHA_MODE_MASK_BITS: u32 = 0b11;
    const ALPHA_MODE_SHIFT_BITS: u32 = 32 - Self::ALPHA_MODE_MASK_BITS.count_ones();
}

/// The GPU representation of the uniform data of a [`ColorMaterial`].
#[derive(Clone, Default, ShaderType)]
pub struct ChunkMaterialUniform {
    pub color: Vec4,
    pub uv_transform: Mat3,
    pub flags: u32,
    pub alpha_cutoff: f32,
    pub chunk_size: f32,
}

impl AsBindGroupShaderType<ChunkMaterialUniform> for ChunkMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> ChunkMaterialUniform {
        let mut flags = ChunkMaterialFlags::NONE;
        if self.texture.is_some() {
            flags |= ChunkMaterialFlags::TEXTURE;
        }

        // Defaults to 0.5 like in 3d
        let mut alpha_cutoff = 0.5;
        match self.alpha_mode {
            AlphaMode2d::Opaque => flags |= ChunkMaterialFlags::ALPHA_MODE_OPAQUE,
            AlphaMode2d::Mask(c) => {
                alpha_cutoff = c;
                flags |= ChunkMaterialFlags::ALPHA_MODE_MASK;
            }
            AlphaMode2d::Blend => flags |= ChunkMaterialFlags::ALPHA_MODE_BLEND,
        };
        ChunkMaterialUniform {
            color: LinearRgba::from(self.color).to_f32_array().into(),
            uv_transform: self.uv_transform.into(),
            flags: flags.bits(),
            alpha_cutoff,
            chunk_size: CHUNK_SIZE as f32,
        }
    }
}

impl Material2d for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        CHUNK_MATERIAL_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        self.alpha_mode
    }
}
