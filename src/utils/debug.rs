use crate::{
    particle::PARTICLE_SIZE,
    player::DebugMode,
    utils::coords::{center_in_screen, chunk_to_pixels},
    world::chunk::{self, CHUNK_SIZE},
    world::map::Map,
};
use bevy::{
    math::{Affine3A, Vec3A},
    prelude::*,
    render::primitives::{Aabb, Frustum},
    utils::{HashMap, HashSet},
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
    // Track chunk visualization entities for cleanup
    pub chunk_entities: HashSet<Entity>,
    // Track chunk outline entities for cleanup
    pub chunk_outline_entities: HashSet<Entity>,
    // Previous visibility state to detect changes
    pub chunks_visible_last_frame: bool,
    // Previous outline visibility state to detect changes
    pub outlines_visible_last_frame: bool,
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
    let chunk_pixels = chunk_to_pixels(chunk_pos);
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

    // If visualization state changed from visible to hidden, clear all visualizations
    if debug_state.chunks_visible_last_frame && !chunks_enabled {
        for entity in debug_state.chunk_entities.drain() {
            commands.entity(entity).despawn_recursive();
        }
        debug_state.chunks_visible_last_frame = false;
        return;
    }

    // Skip if debug mode or chunk visualization is disabled
    if !chunks_enabled {
        return;
    }

    // Mark the visualization as visible for this frame
    debug_state.chunks_visible_last_frame = true;

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
        debug_state.chunk_entities.remove(&entity);
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

                    // Update color based on active state
                    if is_active {
                        sprite.color = Color::srgba(0.0, 1.0, 0.0, 0.3); // Green for active
                    } else {
                        sprite.color = Color::srgba(1.0, 0.0, 0.0, 0.3); // Red for inactive
                    }
                }

                *entity
            } else {
                // Get chunk dimensions and position
                let (chunk_size, center_pos) = get_chunk_dimensions(chunk_pos, &map);

                // Check if chunk is active
                let is_active = active_chunks.contains(&chunk_pos);

                // Create chunk visualization
                let chunk_entity = commands
                    .spawn((
                        Name::new(format!("ChunkHighlight({})", chunk_pos)),
                        Sprite {
                            custom_size: Some(chunk_size),
                            color: if is_active {
                                Color::srgba(0.0, 1.0, 0.0, 0.3) // Green for active
                            } else {
                                Color::srgba(1.0, 0.0, 0.0, 0.3) // Red for inactive
                            },
                            ..default()
                        },
                        Transform::from_xyz(center_pos.x, center_pos.y, 10.0),
                        ChunkVisual {
                            chunk_pos,
                            is_active,
                        },
                        // Add Visibility components for frustum culling
                        Visibility::Inherited,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
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

                // Track the new entity
                debug_state.chunk_entities.insert(chunk_entity);

                chunk_entity
            };

            // Ensure it's in our tracking set
            debug_state.chunk_entities.insert(chunk_entity);
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

    // If state changed from visible to hidden, clear all outlines
    if debug_state.outlines_visible_last_frame && !outlines_enabled {
        for entity in debug_state.chunk_outline_entities.drain() {
            commands.entity(entity).despawn_recursive();
        }
        debug_state.outlines_visible_last_frame = false;
        return;
    }

    // Skip if debug mode or chunk outlines are disabled
    if !outlines_enabled {
        return;
    }

    // Mark the outlines as visible for this frame
    debug_state.outlines_visible_last_frame = true;

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
        debug_state.chunk_outline_entities.remove(&entity);
    }

    // Update sprite colors for existing outlines based on active status
    // This fixes the issue with outlines not changing color when a chunk becomes active
    let mut outline_entities = HashMap::new();
    for (entity, outline) in chunk_outline_query.iter() {
        outline_entities.insert(entity, outline.is_active);
    }

    for (parent, mut sprite) in outline_sprites_query.iter_mut() {
        let parent_entity = parent.get();
        if let Ok((_, outline)) = chunk_outline_query.get(parent_entity) {
            let is_active = active_chunks.contains(&outline.chunk_pos);

            // Only update color if active state has changed
            if outline.is_active != is_active {
                // The color will be updated when we process this entity below
                // Just marking that we detected a state change
                if let Some(active_state) = outline_entities.get_mut(&parent_entity) {
                    *active_state = is_active;
                }
            }

            // Update child sprite colors based on parent's active state
            let outline_color = if is_active {
                Color::srgb(0.0, 1.0, 0.2) // Bright green for active
            } else {
                Color::srgb(1.0, 0.2, 0.2) // Bright red for inactive
            };
            sprite.color = outline_color;
        }
    }

    // Track existing and new chunk outlines
    let mut existing_outlines = vec![vec![Entity::PLACEHOLDER; chunk_height]; chunk_width];
    for (entity, chunk_outline) in chunk_outline_query.iter() {
        if visible_chunk_positions.contains(&chunk_outline.chunk_pos) {
            existing_outlines[chunk_outline.chunk_pos.x as usize]
                [chunk_outline.chunk_pos.y as usize] = entity;
        }
    }

    // Only process chunks that are visible to the camera
    for (cx, col) in existing_outlines.iter_mut().enumerate() {
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
                    .with_children(|parent| {
                        // Colors based on active state
                        let outline_color = if is_active {
                            Color::srgb(0.0, 1.0, 0.2) // Bright green for active
                        } else {
                            Color::srgb(1.0, 0.2, 0.2) // Bright red for inactive
                        };

                        // Half dimensions (from center to edge)
                        let half_width = chunk_size.x / 2.0;
                        let half_height = chunk_size.y / 2.0;

                        // Top line (horizontal)
                        parent.spawn(create_line_segment(
                            Vec2::new(chunk_size.x, line_thickness),
                            Vec3::new(0.0, half_height - line_thickness / 2.0, 0.0),
                            outline_color,
                        ));

                        // Right line (vertical)
                        parent.spawn(create_line_segment(
                            Vec2::new(line_thickness, chunk_size.y),
                            Vec3::new(half_width - line_thickness / 2.0, 0.0, 0.0),
                            outline_color,
                        ));

                        // Bottom line (horizontal)
                        parent.spawn(create_line_segment(
                            Vec2::new(chunk_size.x, line_thickness),
                            Vec3::new(0.0, -half_height + line_thickness / 2.0, 0.0),
                            outline_color,
                        ));

                        // Left line (vertical)
                        parent.spawn(create_line_segment(
                            Vec2::new(line_thickness, chunk_size.y),
                            Vec3::new(-half_width + line_thickness / 2.0, 0.0, 0.0),
                            outline_color,
                        ));
                    })
                    .id();

                // Track the new entity
                debug_state.chunk_outline_entities.insert(outline_entity);

                outline_entity
            };

            // Ensure it's in our tracking set
            debug_state.chunk_outline_entities.insert(outline_entity);
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
    if !debug_state.chunk_entities.is_empty() {
        for entity in debug_state.chunk_entities.drain() {
            commands.entity(entity).despawn_recursive();
        }
        debug_state.chunks_visible_last_frame = false;
    }

    // Clean up all chunk outline entities
    if !debug_state.chunk_outline_entities.is_empty() {
        for entity in debug_state.chunk_outline_entities.drain() {
            commands.entity(entity).despawn_recursive();
        }
        debug_state.outlines_visible_last_frame = false;
    }
}
