#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::view,
}

#ifdef TONEMAP_IN_SHADER
#import bevy_core_pipeline::tonemapping
#endif

struct ChunkMaterial {
    color: vec4<f32>,
    uv_transform: mat3x3<f32>,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    flags: u32,
    alpha_cutoff: f32,
};

const CHUNK_MATERIAL_FLAGS_TEXTURE_BIT: u32              = 1u;
const CHUNK_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS: u32 = 3221225472u; // (0b11u32 << 30)
const CHUNK_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE: u32        = 0u;          // (0u32 << 30)
const CHUNK_MATERIAL_FLAGS_ALPHA_MODE_MASK: u32          = 1073741824u; // (1u32 << 30)
const CHUNK_MATERIAL_FLAGS_ALPHA_MODE_BLEND: u32         = 2147483648u; // (2u32 << 30)

@group(2) @binding(0) var<uniform> material: ChunkMaterial;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var texture_sampler: sampler;
@group(2) @binding(3) var<uniform> indices: array<vec4<f32>, 1024>; // Size is CHUNK_SIZE * CHUNK_SIZE = 1024

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    var output_color: vec4<f32> = material.color;

#ifdef VERTEX_COLORS
    output_color = output_color * mesh.color;
#endif

    // Calculate which cell in the 32x32 grid we're in based on UV coordinates
    // Use floor instead of direct casting to ensure consistent rounding behavior
    let grid_x = u32(floor(mesh.uv.x * 32.0));
    // Flip Y coordinate since chunks are built from bottom-left (0,0)
    // In UV space, 0,0 is bottom-left, but we need to convert to grid space where 0,0 is bottom-left
    let grid_y = u32(floor((1.0 - mesh.uv.y) * 32.0));
    
    // Clamp to valid range to prevent out-of-bounds access
    let safe_grid_x = min(grid_x, 31u);
    let safe_grid_y = min(grid_y, 31u);
    let index = safe_grid_y * 32u + safe_grid_x;
    
    // Get the index value from our indices array
    let sprite_index = u32(indices[index].x);
    
    // Transform UVs to sample the correct part of the texture
    let uv = (material.uv_transform * vec3(mesh.uv, 1.0)).xy;
        
    // The texture is 1 pixel tall with 5 pixels wide (indices 0-4)
    // Calculate texture coordinates with a small inset to avoid edge artifacts
    let sprite_width = 1.0 / 5.0;
    let inset = 0.001; // Small inset to avoid sampling at exact texture boundaries
    
    // Calculate the texture coordinates with inset to avoid edge artifacts
    var tex_uv = vec2<f32>(
        (f32(sprite_index) * sprite_width) + inset,
        0.5
    );
    
    // For sprites other than the empty sprite (index 0), adjust the sampling area
    if (sprite_index > 0u) {
        // Add a small offset to avoid sampling exactly at texture boundaries
        tex_uv.x = (f32(sprite_index) * sprite_width) + (sprite_width * 0.5);
    }
    
    if ((material.flags & CHUNK_MATERIAL_FLAGS_TEXTURE_BIT) != 0u) {
        output_color = output_color * textureSample(texture, texture_sampler, tex_uv);
    }
    

    output_color = alpha_discard(material, output_color);

#ifdef TONEMAP_IN_SHADER
    output_color = tonemapping::tone_mapping(output_color, view.color_grading);
#endif
    return output_color;
}

fn alpha_discard(material: ChunkMaterial, output_color: vec4<f32>) -> vec4<f32> {
    var color = output_color;
    let alpha_mode = material.flags & CHUNK_MATERIAL_FLAGS_ALPHA_MODE_RESERVED_BITS;
    if alpha_mode == CHUNK_MATERIAL_FLAGS_ALPHA_MODE_OPAQUE {
        // NOTE: If rendering as opaque, alpha should be ignored so set to 1.0
        color.a = 1.0;
    }
#ifdef MAY_DISCARD
    else if alpha_mode == CHUNK_MATERIAL_FLAGS_ALPHA_MODE_MASK {
       if color.a >= material.alpha_cutoff {
            // NOTE: If rendering as masked alpha and >= the cutoff, render as fully opaque
            color.a = 1.0;
        } else {
            // NOTE: output_color.a < in.material.alpha_cutoff should not be rendered
            discard;
        }
    }
#endif // MAY_DISCARD

    return color;
}