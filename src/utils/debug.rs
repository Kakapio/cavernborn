use crate::{
    particle::PARTICLE_SIZE,
    player::DebugMode,
    utils::coords::{center_in_screen, chunk_pos_to_screen},
    world::chunk::{self, CHUNK_SIZE},
    world::map::Map,
};
use bevy::{
    math::{Affine3A, Vec3A},
    prelude::*,
    render::primitives::{Aabb, Frustum},
    utils::HashSet,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// Plugin for debug visualization features
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>()
            .add_plugins(
                WorldInspectorPlugin::new().run_if(|debug_mode: Res<DebugMode>| debug_mode.enabled),
            )
            .add_systems(
                Update,
                (
                    toggle_debug_features,
                    update_debug_chunk_visuals,
                    update_debug_chunk_outlines,
                    cleanup_debug_visuals,
                ),
            );
    }
}

// Centralized debug state to track various debug features
#[derive(Resource, Default)]
pub struct DebugState {
    // Whether to show chunk visualization
    pub show_chunks: bool,
    // Whether to show chunk outlines
    pub show_chunk_outlines: bool,
    // Parent entity for all chunk visualization entities
    pub chunk_visuals_parent: Option<Entity>,
    // Parent entity for all chunk outline entities
    pub chunk_outlines_parent: Option<Entity>,
}

// Component for chunk visualization
#[derive(Component)]
pub struct ChunkVisual {
    pub chunk_pos: UVec2,
    pub is_active: bool,
}

// Component for chunk outline visualization
#[derive(Component)]
pub struct ChunkOutline {
    pub chunk_pos: UVec2,
    pub is_active: bool,
}

// Toggle debug features with keyboard shortcuts
fn toggle_debug_features(
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
) {
    if !debug_mode.enabled {
        return;
    }

    // Only process debug keys if debug mode is enabled
    // F4 toggles chunk visualization (both outlines and labels)
    if keyboard.just_pressed(KeyCode::F4) {
        debug_state.show_chunks = !debug_state.show_chunks;
        info!(
            "Chunk visualization: {}",
            if debug_state.show_chunks { "ON" } else { "OFF" }
        );
    }

    // F5 toggles chunk outlines
    if keyboard.just_pressed(KeyCode::F5) {
        debug_state.show_chunk_outlines = !debug_state.show_chunk_outlines;
        info!(
            "Chunk outlines: {}",
            if debug_state.show_chunk_outlines {
                "ON"
            } else {
                "OFF"
            }
        );
    }
}

// Helper function to calculate chunk dimensions and world positioning
fn get_chunk_dimensions(chunk_pos: UVec2, map: &Map) -> (Vec2, Vec2) {
    // Calculate world position for this chunk in pixels
    let chunk_pixels = chunk_pos_to_screen(chunk_pos);
    let chunk_size_pixels = (chunk::CHUNK_SIZE * PARTICLE_SIZE) as f32;

    // Adjust for world centering
    let centered_pos = center_in_screen(chunk_pixels, map.width, map.height);

    // Calculate the center position of the chunk
    let center_pos = Vec2::new(
        centered_pos.x + chunk_size_pixels / 2.0,
        centered_pos.y + chunk_size_pixels / 2.0,
    );

    (Vec2::new(chunk_size_pixels, chunk_size_pixels), center_pos)
}

// Helper function to create line segment components
fn create_line_segment(
    size: Vec2,
    position: Vec3,
    color: Color,
) -> (
    Sprite,
    Transform,
    GlobalTransform,
    Visibility,
    InheritedVisibility,
    ViewVisibility,
) {
    (
        Sprite {
            custom_size: Some(size),
            color,
            ..default()
        },
        Transform::from_translation(position),
        GlobalTransform::default(),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    )
}

// Helper function to check if a chunk is visible in camera view
fn is_chunk_visible(
    chunk_pos: UVec2,
    map: &Map,
    _camera_transform: &Transform, // Prefix with underscore since unused
    camera_frustum: Option<&Frustum>,
) -> bool {
    // If no frustum is available, consider the chunk visible
    let Some(frustum) = camera_frustum else {
        return true;
    };

    // Get chunk dimensions and world position
    let (chunk_size, center_pos) = get_chunk_dimensions(chunk_pos, map);

    // Create an AABB for the chunk
    // The chunk's min is at bottom-left, max is at top-right
    let half_size = chunk_size / 2.0;
    let center = Vec3A::new(center_pos.x, center_pos.y, 0.0);
    let half_extents = Vec3A::new(half_size.x, half_size.y, 0.1); // Small Z extent

    let aabb = Aabb {
        center,
        half_extents,
    };

    // Check if the AABB intersects with the frustum
    // Use identity transform since our AABB is already in world space
    // Check intersection with both near and far planes
    frustum.intersects_obb(&aabb, &Affine3A::IDENTITY, true, true)
}

// Update chunk visualization based on debug state
fn update_debug_chunk_visuals(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
    map: Res<Map>,
    mut chunk_visual_query: Query<(Entity, &mut ChunkVisual, &mut Sprite)>,
    camera_query: Query<(&Transform, &Camera, Option<&Frustum>)>,
) {
    // Determine if chunk visualization should be visible
    let chunks_enabled = debug_mode.enabled && debug_state.show_chunks;

    // If chunks should not be visible, clean up
    if !chunks_enabled {
        // Despawn the parent entity (which will cascade to all children)
        if let Some(parent) = debug_state.chunk_visuals_parent {
            commands.entity(parent).despawn_recursive();
            debug_state.chunk_visuals_parent = None;
        }
        return;
    }

    // Use the map's active_chunks directly - this is the source of truth for what chunks are active
    let active_chunks = &map.active_chunks;

    // Get camera for visibility check
    let camera_data = camera_query.iter().next();

    // First, create a set of all currently visible chunks
    let mut visible_chunk_positions = HashSet::new();

    // Track which chunks should have visuals
    let chunk_width = map.width.div_ceil(CHUNK_SIZE) as usize;
    let chunk_height = map.height.div_ceil(CHUNK_SIZE) as usize;

    // Determine which chunks are visible
    for cx in 0..chunk_width {
        for cy in 0..chunk_height {
            let chunk_pos = UVec2::new(cx as u32, cy as u32);

            // Check if this chunk is visible in camera view
            let is_chunk_in_view = if let Some((camera_transform, _, frustum)) = camera_data {
                is_chunk_visible(chunk_pos, &map, camera_transform, frustum)
            } else {
                true // If no camera found, default to visible
            };

            if is_chunk_in_view {
                visible_chunk_positions.insert(chunk_pos);
            }
        }
    }

    // Create a parent entity for all chunk visuals if it doesn't exist yet
    if debug_state.chunk_visuals_parent.is_none() {
        debug_state.chunk_visuals_parent = Some(
            commands
                .spawn((
                    Name::new("ChunkVisualsParent"),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    Node {
                        display: Display::Block,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                ))
                .id(),
        );
    }

    let parent_entity = debug_state.chunk_visuals_parent.unwrap();

    // Get entities that need to be removed (those that are no longer visible)
    let mut entities_to_remove = Vec::new();
    for (entity, chunk_visual, _) in chunk_visual_query.iter() {
        if !visible_chunk_positions.contains(&chunk_visual.chunk_pos) {
            entities_to_remove.push(entity);
        }
    }

    // Despawn entities that are no longer visible
    for entity in entities_to_remove {
        commands.entity(entity).despawn_recursive();
    }

    // Now handle the visible chunks
    let mut existing_chunks = vec![vec![Entity::PLACEHOLDER; chunk_height]; chunk_width];
    for (entity, chunk_visual, _) in chunk_visual_query.iter() {
        if visible_chunk_positions.contains(&chunk_visual.chunk_pos) {
            existing_chunks[chunk_visual.chunk_pos.x as usize][chunk_visual.chunk_pos.y as usize] =
                entity;
        }
    }

    // Only process chunks that are visible to the camera
    for (cx, col) in existing_chunks.iter_mut().enumerate() {
        for (cy, entity) in col.iter_mut().enumerate() {
            let chunk_pos = UVec2::new(cx as u32, cy as u32);

            // Skip if this chunk isn't visible
            if !visible_chunk_positions.contains(&chunk_pos) {
                continue;
            }

            let chunk_entity = if *entity != Entity::PLACEHOLDER {
                // Update existing chunk visual
                if let Ok(entry) = chunk_visual_query.get_mut(*entity) {
                    let (_, mut visual_comp, mut sprite) = entry;

                    // Check if the chunk is active based on the map's active_chunks set
                    let is_active = active_chunks.contains(&chunk_pos);
                    visual_comp.is_active = is_active;

                    // Update the sprite color based on active state
                    if is_active {
                        sprite.color = Color::srgba(0.0, 1.0, 0.0, 0.2); // Active chunks: green tint
                    } else {
                        sprite.color = Color::srgba(1.0, 0.0, 0.0, 0.2); // Inactive chunks: red tint
                    }
                }

                *entity
            } else {
                // Get chunk dimensions and position
                let (chunk_size, center_pos) = get_chunk_dimensions(chunk_pos, &map);

                // Check if chunk is active
                let is_active = active_chunks.contains(&chunk_pos);

                // Set color based on active state
                let color = if is_active {
                    Color::srgba(0.0, 1.0, 0.0, 0.2) // Active chunks: green tint
                } else {
                    Color::srgba(1.0, 0.0, 0.0, 0.2) // Inactive chunks: red tint
                };

                // Spawn the new chunk entity as a child of the parent
                let chunk_entity = commands
                    .spawn((
                        Name::new(format!("ChunkVisual({},{})", chunk_pos.x, chunk_pos.y)),
                        Sprite {
                            color,
                            custom_size: Some(chunk_size),
                            ..default()
                        },
                        Transform::from_xyz(center_pos.x, center_pos.y, 10.0),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        ChunkVisual {
                            chunk_pos,
                            is_active,
                        },
                    ))
                    .with_children(|parent| {
                        // Add text label as a child entity
                        parent.spawn((
                            Text2d::from(format!("{},{}", chunk_pos.x, chunk_pos.y)),
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                        ));
                    })
                    .id();

                // Add as child to parent
                commands.entity(parent_entity).add_child(chunk_entity);

                *entity = chunk_entity;
                chunk_entity
            };

            // Update the grid entry to point to the chunk entity
            *entity = chunk_entity;
        }
    }
}

// Update chunk outlines based on debug state
fn update_debug_chunk_outlines(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
    map: Res<Map>,
    mut chunk_outline_query: Query<(Entity, &mut ChunkOutline)>,
    mut outline_sprites_query: Query<(&Parent, &mut Sprite)>,
    camera_query: Query<(&Transform, &Camera, Option<&Frustum>)>,
) {
    // Determine if chunk outlines should be visible
    let outlines_enabled = debug_mode.enabled && debug_state.show_chunk_outlines;

    // If outlines should not be visible, clean up
    if !outlines_enabled {
        // Despawn the parent entity (which will cascade to all children)
        if let Some(parent) = debug_state.chunk_outlines_parent {
            commands.entity(parent).despawn_recursive();
            debug_state.chunk_outlines_parent = None;
        }
        return;
    }

    // Use the map's active_chunks directly
    let active_chunks = &map.active_chunks;

    // Get camera for visibility check
    let camera_data = camera_query.iter().next();

    // First, create a set of all currently visible chunks
    let mut visible_chunk_positions = HashSet::new();

    // Track which chunks should have outlines
    let chunk_width = map.width.div_ceil(CHUNK_SIZE) as usize;
    let chunk_height = map.height.div_ceil(CHUNK_SIZE) as usize;

    // Determine which chunks are visible
    for cx in 0..chunk_width {
        for cy in 0..chunk_height {
            let chunk_pos = UVec2::new(cx as u32, cy as u32);

            // Check if this chunk is visible in camera view
            let is_chunk_in_view = if let Some((camera_transform, _, frustum)) = camera_data {
                is_chunk_visible(chunk_pos, &map, camera_transform, frustum)
            } else {
                true // If no camera found, default to visible
            };

            if is_chunk_in_view {
                visible_chunk_positions.insert(chunk_pos);
            }
        }
    }

    // Create a parent entity for all chunk outlines if it doesn't exist yet
    if debug_state.chunk_outlines_parent.is_none() {
        debug_state.chunk_outlines_parent = Some(
            commands
                .spawn((
                    Name::new("ChunkOutlinesParent"),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                ))
                .id(),
        );
    }

    let parent_entity = debug_state.chunk_outlines_parent.unwrap();

    // Get entities that need to be removed (those that are no longer visible)
    let mut entities_to_remove = Vec::new();
    for (entity, chunk_outline) in chunk_outline_query.iter() {
        if !visible_chunk_positions.contains(&chunk_outline.chunk_pos) {
            entities_to_remove.push(entity);
        }
    }

    // Despawn entities that are no longer visible
    for entity in entities_to_remove {
        commands.entity(entity).despawn_recursive();
    }

    // Now handle the visible chunks
    let mut existing_chunks = vec![vec![Entity::PLACEHOLDER; chunk_height]; chunk_width];
    for (entity, chunk_outline) in chunk_outline_query.iter() {
        if visible_chunk_positions.contains(&chunk_outline.chunk_pos) {
            existing_chunks[chunk_outline.chunk_pos.x as usize]
                [chunk_outline.chunk_pos.y as usize] = entity;
        }
    }

    // Only process chunks that are visible to the camera
    for (cx, col) in existing_chunks.iter_mut().enumerate() {
        for (cy, entity) in col.iter_mut().enumerate() {
            let chunk_pos = UVec2::new(cx as u32, cy as u32);

            // Skip if this chunk isn't visible
            if !visible_chunk_positions.contains(&chunk_pos) {
                continue;
            }

            let outline_entity = if *entity != Entity::PLACEHOLDER {
                // Update existing chunk outline
                if let Ok((_, mut outline_comp)) = chunk_outline_query.get_mut(*entity) {
                    // Check if the chunk is active based on the map's active_chunks set
                    let is_active = active_chunks.contains(&chunk_pos);
                    outline_comp.is_active = is_active;
                }

                *entity
            } else {
                // Get chunk dimensions and position
                let (chunk_size, center_pos) = get_chunk_dimensions(chunk_pos, &map);

                // Check if chunk is active
                let is_active = active_chunks.contains(&chunk_pos);

                // Create the line thickness relative to chunk size
                let line_thickness = chunk_size.x * 0.02;

                // Create chunk outline entity with four line segments (top, right, bottom, left)
                let outline_entity = commands
                    .spawn((
                        Name::new(format!("ChunkOutline({})", chunk_pos)),
                        Transform::from_xyz(center_pos.x, center_pos.y, 11.0),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        ChunkOutline {
                            chunk_pos,
                            is_active,
                        },
                    ))
                    .id();

                // Create child line segments
                let outline_color = if is_active {
                    Color::srgb(0.0, 1.0, 0.2) // Bright green for active
                } else {
                    Color::srgb(1.0, 0.2, 0.2) // Bright red for inactive
                };

                // Half dimensions (from center to edge)
                let half_width = chunk_size.x / 2.0;
                let half_height = chunk_size.y / 2.0;

                // Top line (horizontal)
                let top_line = commands
                    .spawn(create_line_segment(
                        Vec2::new(chunk_size.x, line_thickness),
                        Vec3::new(0.0, half_height - line_thickness / 2.0, 0.0),
                        outline_color,
                    ))
                    .id();

                // Right line (vertical)
                let right_line = commands
                    .spawn(create_line_segment(
                        Vec2::new(line_thickness, chunk_size.y),
                        Vec3::new(half_width - line_thickness / 2.0, 0.0, 0.0),
                        outline_color,
                    ))
                    .id();

                // Bottom line (horizontal)
                let bottom_line = commands
                    .spawn(create_line_segment(
                        Vec2::new(chunk_size.x, line_thickness),
                        Vec3::new(0.0, -half_height + line_thickness / 2.0, 0.0),
                        outline_color,
                    ))
                    .id();

                // Left line (vertical)
                let left_line = commands
                    .spawn(create_line_segment(
                        Vec2::new(line_thickness, chunk_size.y),
                        Vec3::new(-half_width + line_thickness / 2.0, 0.0, 0.0),
                        outline_color,
                    ))
                    .id();

                // Add all lines as children of the outline entity
                commands.entity(outline_entity).add_child(top_line);
                commands.entity(outline_entity).add_child(right_line);
                commands.entity(outline_entity).add_child(bottom_line);
                commands.entity(outline_entity).add_child(left_line);

                // Add outline entity as child to parent
                commands.entity(parent_entity).add_child(outline_entity);

                *entity = outline_entity;
                outline_entity
            };

            // Update the grid entry to point to the outline entity
            *entity = outline_entity;
        }
    }

    // Update the colors of all outline segments based on active state
    for (parent, mut sprite) in outline_sprites_query.iter_mut() {
        if let Ok((_, outline)) = chunk_outline_query.get(parent.get()) {
            let outline_color = if outline.is_active {
                Color::srgb(0.0, 1.0, 0.2) // Bright green for active
            } else {
                Color::srgb(1.0, 0.2, 0.2) // Bright red for inactive
            };
            sprite.color = outline_color;
        }
    }
}

// Clean up debug visuals when needed
fn cleanup_debug_visuals(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
) {
    if debug_mode.enabled {
        return;
    }

    // Only run this cleanup when debug mode is turned off
    // Clean up all chunk visualization entities
    if let Some(parent) = debug_state.chunk_visuals_parent {
        commands.entity(parent).despawn_recursive();
        debug_state.chunk_visuals_parent = None;
    }

    // Clean up all chunk outline entities
    if let Some(parent) = debug_state.chunk_outlines_parent {
        commands.entity(parent).despawn_recursive();
        debug_state.chunk_outlines_parent = None;
    }
}
