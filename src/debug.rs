use crate::{
    chunk::{self, CHUNK_SIZE},
    map::Map,
    particle::PARTICLE_SIZE,
    player::DebugMode,
    utils::coords::{center_in_screen, chunk_to_pixels},
};
use bevy::{prelude::*, utils::HashSet};

// Plugin for debug visualization features
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>().add_systems(
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
    // Only process debug keys if debug mode is enabled
    if debug_mode.enabled {
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

// Update chunk visualization based on debug state
fn update_debug_chunk_visuals(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
    map: Res<Map>,
    mut chunk_visual_query: Query<(Entity, &mut ChunkVisual, &mut Sprite)>,
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

    // Track existing and new chunk visuals
    let chunk_width = map.width.div_ceil(CHUNK_SIZE) as usize;
    let chunk_height = map.height.div_ceil(CHUNK_SIZE) as usize;

    let mut existing_chunks = vec![vec![Entity::PLACEHOLDER; chunk_height]; chunk_width];
    for (entity, chunk_visual, _) in chunk_visual_query.iter() {
        existing_chunks[chunk_visual.chunk_pos.x as usize][chunk_visual.chunk_pos.y as usize] =
            entity;
    }

    for (cx, col) in existing_chunks.iter_mut().enumerate() {
        for (cy, entity) in col.iter_mut().enumerate() {
            let chunk_pos = UVec2::new(cx as u32, cy as u32);
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
                    ))
                    .with_children(|parent| {
                        // Add text label as a child entity
                        parent.spawn((
                            Text::from(format!("{},{}", chunk_pos.x, chunk_pos.y)),
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

            // Mark this position as processed
            *entity = chunk_entity;
        }
    }

    // Create a copy of debug_state.chunk_entities to track which ones we've seen
    let mut entities_to_remove = debug_state.chunk_entities.clone();

    // Remove the entities we just processed from the removal set
    for chunk_row in existing_chunks.iter().take(chunk_width) {
        for &entity in chunk_row.iter().take(chunk_height) {
            if entity != Entity::PLACEHOLDER {
                entities_to_remove.remove(&entity);
            }
        }
    }

    // Now remove any entities that weren't processed this frame
    for entity in entities_to_remove {
        commands.entity(entity).despawn_recursive();
        debug_state.chunk_entities.remove(&entity);
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

    // Update existing outline colors based on active status
    for (parent, mut sprite) in outline_sprites_query.iter_mut() {
        if let Ok((_, chunk_outline)) = chunk_outline_query.get(parent.get()) {
            let is_active = active_chunks.contains(&chunk_outline.chunk_pos);
            let outline_color = if is_active {
                Color::srgb(0.0, 1.0, 0.2) // Bright green for active
            } else {
                Color::srgb(1.0, 0.2, 0.2) // Bright red for inactive
            };
            sprite.color = outline_color;
        }
    }

    // Track existing and new chunk outlines
    let chunk_width = map.width.div_ceil(CHUNK_SIZE) as usize;
    let chunk_height = map.height.div_ceil(CHUNK_SIZE) as usize;

    let mut existing_outlines = vec![vec![Entity::PLACEHOLDER; chunk_height]; chunk_width];
    for (entity, chunk_outline) in chunk_outline_query.iter() {
        existing_outlines[chunk_outline.chunk_pos.x as usize][chunk_outline.chunk_pos.y as usize] =
            entity;
    }

    for (cx, col) in existing_outlines.iter_mut().enumerate() {
        for (cy, entity) in col.iter_mut().enumerate() {
            let chunk_pos = UVec2::new(cx as u32, cy as u32);
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
                        // Use direct components instead of SpatialBundle
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

            // Mark this position as processed
            *entity = outline_entity;
        }
    }

    // Create a copy of debug_state.chunk_outline_entities to track which ones we've seen
    let mut entities_to_remove = debug_state.chunk_outline_entities.clone();

    // Remove the entities we just processed from the removal set
    for chunk_row in existing_outlines.iter().take(chunk_width) {
        for &entity in chunk_row.iter().take(chunk_height) {
            if entity != Entity::PLACEHOLDER {
                entities_to_remove.remove(&entity);
            }
        }
    }

    // Now remove any entities that weren't processed this frame
    for entity in entities_to_remove {
        commands.entity(entity).despawn_recursive();
        debug_state.chunk_outline_entities.remove(&entity);
    }
}

// Clean up debug visuals when needed
fn cleanup_debug_visuals(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
) {
    // Only run this cleanup when debug mode is turned off
    if !debug_mode.enabled {
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
}
