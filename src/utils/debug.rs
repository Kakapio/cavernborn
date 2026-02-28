use crate::{player::DebugMode, utils::coords, world::chunk::CHUNK_SIZE, world::map::Map};
use bevy::{
    math::{Affine3A, Vec3A},
    prelude::*,
    render::primitives::{Aabb, Frustum},
    utils::HashSet,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

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
                    (
                        update_debug_overlay::<ChunkVisual>,
                        update_debug_overlay::<ChunkOutline>,
                    ),
                    (sync_visual_colors, sync_outline_colors),
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
pub struct DebugState {
    pub show_chunks: bool,
    pub show_chunk_outlines: bool,
    pub chunk_visuals_parent: Option<Entity>,
    pub chunk_outlines_parent: Option<Entity>,
}

#[derive(Component)]
pub struct ChunkVisual {
    pub chunk_pos: UVec2,
}

#[derive(Component)]
pub struct ChunkOutline {
    pub chunk_pos: UVec2,
}

trait ChunkOverlay: Component {
    fn chunk_pos(&self) -> UVec2;
    fn is_enabled(debug_state: &DebugState, debug_mode: &DebugMode) -> bool;
    fn get_parent(debug_state: &DebugState) -> Option<Entity>;
    fn set_parent(debug_state: &mut DebugState, entity: Option<Entity>);
    fn parent_name() -> &'static str;
    fn spawn_overlay(
        commands: &mut Commands,
        parent: Entity,
        chunk_pos: UVec2,
        chunk_size: Vec2,
        center_pos: Vec2,
        is_active: bool,
    ) -> Entity;
}

impl ChunkOverlay for ChunkVisual {
    fn chunk_pos(&self) -> UVec2 {
        self.chunk_pos
    }

    fn is_enabled(debug_state: &DebugState, debug_mode: &DebugMode) -> bool {
        debug_mode.enabled && debug_state.show_chunks
    }

    fn get_parent(debug_state: &DebugState) -> Option<Entity> {
        debug_state.chunk_visuals_parent
    }

    fn set_parent(debug_state: &mut DebugState, entity: Option<Entity>) {
        debug_state.chunk_visuals_parent = entity;
    }

    fn parent_name() -> &'static str {
        "ChunkVisualsParent"
    }

    fn spawn_overlay(
        commands: &mut Commands,
        parent: Entity,
        chunk_pos: UVec2,
        chunk_size: Vec2,
        center_pos: Vec2,
        is_active: bool,
    ) -> Entity {
        let color = if is_active {
            Color::srgba(0.0, 1.0, 0.0, 0.2)
        } else {
            Color::srgba(1.0, 0.0, 0.0, 0.2)
        };

        let entity = commands
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
                ChunkVisual { chunk_pos },
            ))
            .with_children(|builder| {
                builder.spawn(Text2d::from(format!("{},{}", chunk_pos.x, chunk_pos.y)));
            })
            .id();

        commands.entity(parent).add_child(entity);
        entity
    }
}

impl ChunkOverlay for ChunkOutline {
    fn chunk_pos(&self) -> UVec2 {
        self.chunk_pos
    }

    fn is_enabled(debug_state: &DebugState, debug_mode: &DebugMode) -> bool {
        debug_mode.enabled && debug_state.show_chunk_outlines
    }

    fn get_parent(debug_state: &DebugState) -> Option<Entity> {
        debug_state.chunk_outlines_parent
    }

    fn set_parent(debug_state: &mut DebugState, entity: Option<Entity>) {
        debug_state.chunk_outlines_parent = entity;
    }

    fn parent_name() -> &'static str {
        "ChunkOutlinesParent"
    }

    fn spawn_overlay(
        commands: &mut Commands,
        parent: Entity,
        chunk_pos: UVec2,
        chunk_size: Vec2,
        center_pos: Vec2,
        is_active: bool,
    ) -> Entity {
        let line_thickness = chunk_size.x * 0.02;
        let outline_color = if is_active {
            Color::srgb(0.0, 1.0, 0.2)
        } else {
            Color::srgb(1.0, 0.2, 0.2)
        };

        let half_width = chunk_size.x / 2.0;
        let half_height = chunk_size.y / 2.0;

        let entity = commands
            .spawn((
                Name::new(format!("ChunkOutline({})", chunk_pos)),
                Transform::from_xyz(center_pos.x, center_pos.y, 11.0),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                ChunkOutline { chunk_pos },
            ))
            .with_children(|builder| {
                builder.spawn(create_line_segment(
                    Vec2::new(chunk_size.x, line_thickness),
                    Vec3::new(0.0, half_height - line_thickness / 2.0, 0.0),
                    outline_color,
                ));
                builder.spawn(create_line_segment(
                    Vec2::new(line_thickness, chunk_size.y),
                    Vec3::new(half_width - line_thickness / 2.0, 0.0, 0.0),
                    outline_color,
                ));
                builder.spawn(create_line_segment(
                    Vec2::new(chunk_size.x, line_thickness),
                    Vec3::new(0.0, -half_height + line_thickness / 2.0, 0.0),
                    outline_color,
                ));
                builder.spawn(create_line_segment(
                    Vec2::new(line_thickness, chunk_size.y),
                    Vec3::new(-half_width + line_thickness / 2.0, 0.0, 0.0),
                    outline_color,
                ));
            })
            .id();

        commands.entity(parent).add_child(entity);
        entity
    }
}

fn toggle_debug_features(
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
) {
    if !debug_mode.enabled {
        return;
    }

    if keyboard.just_pressed(KeyCode::F4) {
        debug_state.show_chunks = !debug_state.show_chunks;
        info!(
            "Chunk visualization: {}",
            if debug_state.show_chunks { "ON" } else { "OFF" }
        );
    }

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

fn get_chunk_dimensions(chunk_pos: UVec2, map: &Map) -> (Vec2, Vec2) {
    coords::chunk_screen_rect(chunk_pos, map.width, map.height)
}

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

fn is_chunk_visible(chunk_pos: UVec2, map: &Map, camera_frustum: Option<&Frustum>) -> bool {
    let Some(frustum) = camera_frustum else {
        return true;
    };

    let (chunk_size, center_pos) = get_chunk_dimensions(chunk_pos, map);

    let half_size = chunk_size / 2.0;
    let center = Vec3A::new(center_pos.x, center_pos.y, 0.0);
    let half_extents = Vec3A::new(half_size.x, half_size.y, 0.1);

    let aabb = Aabb {
        center,
        half_extents,
    };

    frustum.intersects_obb(&aabb, &Affine3A::IDENTITY, true, true)
}

fn compute_visible_chunks(map: &Map, camera_frustum: Option<&Frustum>) -> HashSet<UVec2> {
    let chunk_width = map.width.div_ceil(CHUNK_SIZE) as usize;
    let chunk_height = map.height.div_ceil(CHUNK_SIZE) as usize;
    let mut visible = HashSet::new();

    for cx in 0..chunk_width {
        for cy in 0..chunk_height {
            let chunk_pos = UVec2::new(cx as u32, cy as u32);
            if is_chunk_visible(chunk_pos, map, camera_frustum) {
                visible.insert(chunk_pos);
            }
        }
    }

    visible
}

fn update_debug_overlay<T: ChunkOverlay>(
    mut commands: Commands,
    debug_mode: Res<DebugMode>,
    mut debug_state: ResMut<DebugState>,
    map: Res<Map>,
    overlay_query: Query<(Entity, &T)>,
    camera_query: Query<(&Transform, &Camera, Option<&Frustum>)>,
) {
    if !T::is_enabled(&debug_state, &debug_mode) {
        if let Some(parent) = T::get_parent(&debug_state) {
            commands.entity(parent).despawn_recursive();
            T::set_parent(&mut debug_state, None);
        }
        return;
    }

    let camera_frustum = camera_query.iter().next().and_then(|(_, _, f)| f);
    let visible_chunks = compute_visible_chunks(&map, camera_frustum);

    if T::get_parent(&debug_state).is_none() {
        let parent = commands
            .spawn((
                Name::new(T::parent_name()),
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .id();
        T::set_parent(&mut debug_state, Some(parent));
    }
    let parent_entity = T::get_parent(&debug_state).unwrap();

    // Despawn overlays for non-visible chunks
    for (entity, overlay) in overlay_query.iter() {
        if !visible_chunks.contains(&overlay.chunk_pos()) {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Collect existing overlay positions
    let existing: HashSet<UVec2> = overlay_query
        .iter()
        .filter(|(_, o)| visible_chunks.contains(&o.chunk_pos()))
        .map(|(_, o)| o.chunk_pos())
        .collect();

    // Spawn new overlays for visible chunks that don't have one yet
    for &chunk_pos in &visible_chunks {
        if existing.contains(&chunk_pos) {
            continue;
        }
        let (chunk_size, center_pos) = get_chunk_dimensions(chunk_pos, &map);
        let is_active = map.active_chunks.contains(&chunk_pos);
        T::spawn_overlay(
            &mut commands,
            parent_entity,
            chunk_pos,
            chunk_size,
            center_pos,
            is_active,
        );
    }
}

fn sync_visual_colors(map: Res<Map>, mut query: Query<(&ChunkVisual, &mut Sprite)>) {
    for (visual, mut sprite) in query.iter_mut() {
        let is_active = map.active_chunks.contains(&visual.chunk_pos);
        sprite.color = if is_active {
            Color::srgba(0.0, 1.0, 0.0, 0.2)
        } else {
            Color::srgba(1.0, 0.0, 0.0, 0.2)
        };
    }
}

fn sync_outline_colors(
    map: Res<Map>,
    outline_query: Query<&ChunkOutline>,
    mut sprite_query: Query<(&Parent, &mut Sprite)>,
) {
    for (parent, mut sprite) in sprite_query.iter_mut() {
        if let Ok(outline) = outline_query.get(parent.get()) {
            let is_active = map.active_chunks.contains(&outline.chunk_pos);
            sprite.color = if is_active {
                Color::srgb(0.0, 1.0, 0.2)
            } else {
                Color::srgb(1.0, 0.2, 0.2)
            };
        }
    }
}
