use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

const TEXTURE_SIZE: (u32, u32) = (300, 300);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cavernborn".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, update_texture)
        .run();
}

#[derive(Component)]
struct CustomTexture;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Add a 2D camera
    commands.spawn(Camera2dBundle::default());

    // Create a new image
    let mut image = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE.0,
            height: TEXTURE_SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;

    let image_handle = images.add(image);

    commands.spawn((
        SpriteBundle {
            texture: image_handle,
            sprite: Sprite {
                custom_size: Some(Vec2::new(TEXTURE_SIZE.0 as f32, TEXTURE_SIZE.1 as f32)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        CustomTexture,
    ));

    info!("Starting game...");
}

fn update_texture(
    mut images: ResMut<Assets<Image>>,
    custom_texture: Query<&Handle<Image>, With<CustomTexture>>,
    time: Res<Time>,
) {
    if let Ok(texture_handle) = custom_texture.get_single() {
        if let Some(texture) = images.get_mut(texture_handle) {
            let t = time.elapsed_seconds();

            // Example: Create a moving pattern
            for y in 0..TEXTURE_SIZE.1 {
                for x in 0..TEXTURE_SIZE.0 {
                    let offset = 4 * (y * TEXTURE_SIZE.0 + x) as usize;
                    let r = (((x as f32 + t * 50.0) / 10.0).sin() * 0.5 + 0.5) * 255.0;
                    let g = (((y as f32 + t * 30.0) / 10.0).cos() * 0.5 + 0.5) * 255.0;
                    let b = ((((x + y) as f32 + t * 70.0) / 20.0).sin() * 0.5 + 0.5) * 255.0;

                    texture.data[offset] = r as u8;
                    texture.data[offset + 1] = g as u8;
                    texture.data[offset + 2] = b as u8;
                    texture.data[offset + 3] = 255;
                }
            }
        }
    }
}
