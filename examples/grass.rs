use bevy::{prelude::*, window::PresentMode, pbr::wireframe::WireframePlugin, diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin}, render::mesh::VertexAttributeValues};
use bevy_flycam::PlayerPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use procedural_grass::{ProceduralGrassPlugin, grass::{grass::{GrassBundle, GrassGeneration, GrassColor, Blade}, mesh::GrassMesh, wind::{GrassWind, WindMap, Wind}, chunk::GrassChunks}, render::extract::{GrassColorData, WindData, BladeData}};

use noise::NoiseFn;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }),
        WireframePlugin,
        PlayerPlugin,
        WorldInspectorPlugin::new(),
        ProceduralGrassPlugin,
        LogDiagnosticsPlugin::default(),
        FrameTimeDiagnosticsPlugin,
    ))
    .add_systems(Startup, setup)
    .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let mut terrain_mesh = Mesh::from(shape::Plane { size: 1.0, subdivisions: 100 });
    if let Some(positions) = terrain_mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        if let VertexAttributeValues::Float32x3(positions) = positions {
            for position in positions.iter_mut() {
                let y = noise::Perlin::new(1).get([((position[0]) * 5.) as f64, ((position[2]) * 5.) as f64]) as f32;
                position[1] += y;
            }
        }
    }


    let wind_map = asset_server.add(GrassWind::generate_wind_map(512));
    commands.insert_resource(GrassWind {
        wind_map: wind_map.clone(),
        ..default()
    });

    let terrain = commands.spawn((
        PbrBundle {
            mesh: meshes.add(terrain_mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.05, 0.0),
                reflectance: 0.0,
                ..Default::default()
            }),
            transform: Transform::from_scale(Vec3::new(100.0, 3.0, 100.0)),
            ..Default::default()
        }, 
    )).id();

    commands.spawn(GrassBundle {
        mesh: meshes.add(GrassMesh::mesh()),
        grass_generation: GrassGeneration {
            entity: Some(terrain.clone()),
            density: 25,
        },
        grass_color: GrassColorData::from(GrassColor::default()),
        wind_data: WindData::from(Wind::default()),
        blade_data: BladeData::from(Blade::default()),
        wind_map: WindMap {
            wind_map: wind_map.clone(),
        },
        ..default()
    });
     
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight::default(),
        transform: Transform::from_rotation(Quat::from_xyzw(
            -0.4207355,
            -0.4207355,
            0.22984886,
            0.77015114,
        )),
        ..default()
    });
}