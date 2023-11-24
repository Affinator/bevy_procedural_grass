use bevy::{prelude::*, render::{view::NoFrustumCulling, mesh::VertexAttributeValues, render_resource::{Buffer, BufferInitDescriptor, BufferUsages}, render_asset::{RenderAsset, PrepareAssetError}, renderer::RenderDevice, texture::{ImageType, CompressedImageFormats}, primitives::Aabb, extract_component::ExtractComponent}, ecs::system::{lifetimeless::SRes, SystemParamItem}, pbr::wireframe::Wireframe, utils::HashMap};
use bevy_inspector_egui::{prelude::ReflectInspectorOptions, InspectorOptions};

use rand::Rng;

use crate::grass::extract::{GrassInstanceData, InstanceData};

use super::{extract::{GrassColorData, WindData, BladeData}, wind::{Wind, WindMap}, chunk::{GrassChunks, GrassToDraw, self}};

#[derive(Reflect, Component, InspectorOptions, Default)]
#[reflect(Component, InspectorOptions)]
pub struct Grass {
    #[reflect(ignore)]
    pub mesh: Handle<Mesh>,
    #[reflect(ignore)]
    pub grass_entity: Option<Entity>,
    #[reflect(ignore)]
    pub grass_handle: Option<Handle<GrassInstanceData>>,
    #[reflect(ignore)]
    pub wind_map_handle: Handle<Image>,
    #[reflect(ignore)]
    pub chunks: GrassChunks,
    pub density: u32,
    pub color: GrassColor,
    pub blade: Blade,
    pub wind: Wind,
    pub regenerate: bool,
}

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct Blade {
    pub length: f32,
    pub width: f32,
    pub tilt: f32,
    pub bend: f32,
}

impl Default for Blade {
    fn default() -> Self {
        Self {
            length: 1.5,
            width: 1.,
            tilt: 0.5,
            bend: 0.5,
        }
    }
}

pub fn update_grass_data(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut Grass, &Handle<Mesh>), Changed<Grass>>,
    meshes: Res<Assets<Mesh>>,
    mut grass_asset: ResMut<Assets<GrassInstanceData>>,
) {
    for (transform, mut grass, mesh_handle) in query.iter_mut() {
        if grass.regenerate {
            if let (Some(grass_entity), Some(mesh)) = (grass.grass_entity, meshes.get(mesh_handle)) {
                commands.entity(grass_entity).insert(generate_grass_data(&mut grass, transform, mesh, &mut grass_asset));
            }

            grass.regenerate = false;
        }
    }
}

pub fn update_grass_params(
    mut commands: Commands,
    query: Query<&Grass, Changed<Grass>>,
) {
    for grass in query.iter() {
        if let Some(grass_entity) = grass.grass_entity {
            commands.entity(grass_entity)
                .insert(GrassColorData::from(grass.color.clone()))
                .insert(WindData::from(grass.wind.clone()))
                .insert(BladeData::from(grass.blade.clone()));
        }
    }
}

pub fn load_grass(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Grass, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
    mut grass_asset: ResMut<Assets<GrassInstanceData>>,
) {
    for (entity, transform, mut grass, mesh_handle) in query.iter_mut() {
        spawn_grass(&mut commands, transform, &entity, &mut grass, meshes.get(mesh_handle).unwrap(), &mut grass_asset);
    }
}

pub fn generate_grass_data(
    grass: &mut Grass,
    transform: &Transform,
    mesh: &Mesh,
    grass_asset: &mut ResMut<Assets<GrassInstanceData>>,
) -> GrassChunks {
    let mut chunks: HashMap<(i32, i32, i32), GrassInstanceData> = HashMap::new();
    let chunk_size = 30.0; // Define your chunk size

    if let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            if let Some(indices) = mesh.indices() {
                let mut triangle = Vec::new();
                for index in indices.iter() {
                    triangle.push(index);
                    if triangle.len() == 3 {
                        let result: Vec<InstanceData> = {
                            // Calculate the area of the triangle
                            let v0 = Vec3::from(positions[triangle[0] as usize]) * transform.scale;
                            let v1 = Vec3::from(positions[triangle[1] as usize]) * transform.scale;
                            let v2 = Vec3::from(positions[triangle[2] as usize]) * transform.scale;

                            let normal = (v1 - v0).cross(v2 - v0).normalize();
        
                            let area = ((v1 - v0).cross(v2 - v0)).length() / 2.0;
        
                            // Scale the density by the area of the triangle
                            let scaled_density = (grass.density as f32 * area).ceil() as u32;
        
                            (0..scaled_density).filter_map(|_| {
                                let mut rng = rand::thread_rng();
        
                                // Generate random barycentric coordinates
                                let r1 = rng.gen::<f32>().sqrt();
                                let r2 = rng.gen::<f32>();
                                let barycentric = Vec3::new(1.0 - r1, r1 * (1.0 - r2), r1 * r2);
        
                                // Calculate the position of the blade using the barycentric coordinates
                                let position = v0 * barycentric.x + v1 * barycentric.y + v2 * barycentric.z;
                            
                                let uv0 = Vec2::from(uvs[triangle[0] as usize]);
                                let uv1 = Vec2::from(uvs[triangle[1] as usize]);
                                let uv2 = Vec2::from(uvs[triangle[2] as usize]);
                                let uv = uv0 * barycentric.x + uv1 * barycentric.y + uv2 * barycentric.z;

                                let chunk_coords = (
                                    (position.x / chunk_size).floor() as i32,
                                    //(position.y / chunk_size).floor() as i32,
                                    0,
                                    (position.z / chunk_size).floor() as i32,
                                );

                                let instance = InstanceData {
                                    position,
                                    normal,
                                    uv,
                                    chunk: Vec3::new(chunk_coords.0 as f32,chunk_coords.1 as f32, chunk_coords.2 as f32),
                                };

                                // Add instance to the appropriate chunk
                                chunks.entry(chunk_coords).or_insert_with(|| {GrassInstanceData(Vec::new())}).0.push(instance);

                                None
                            }).collect::<Vec<_>>()
                        };
                        triangle.clear();
                    }
                }
            }
        }
    }

    // let mut loaded_chunks = HashMap::new();
    // for (chunk_coords, instance) in &chunks {
    //     let handle = grass_asset.add(instance.clone());
    //     loaded_chunks.insert(*chunk_coords, handle);
    // }

    GrassChunks {
        chunk_size: 30.,
        chunks,
        //loaded: loaded_chunks,
        ..default()
    }
}

pub fn spawn_grass(
    commands: &mut Commands,
    transform: &Transform,
    entity: &Entity, 
    grass: &mut Grass,
    mesh: &Mesh,
    grass_asset: &mut ResMut<Assets<GrassInstanceData>>,
) {
    let grass_handles = generate_grass_data(grass, transform, mesh, grass_asset);
    commands.entity(*entity).insert(grass_handles);

    let grass_entity = commands.spawn((
        grass.mesh.clone(),
        SpatialBundle::INHERITED_IDENTITY,
        GrassToDraw::default(),
        GrassColorData::from(grass.color),
        WindData::from(grass.wind),
        BladeData::from(grass.blade),
        WindMap {
            wind_map: grass.wind_map_handle.clone(),
        },
        NoFrustumCulling,
    )).id();

    grass.grass_entity = Some(grass_entity);
}

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct GrassColor {
    pub ao: Color,
    pub color_1: Color,
    pub color_2: Color,
    pub tip: Color,
}

impl Default for GrassColor {
    fn default() -> Self {
        Self {
            ao: [0.01, 0.02, 0.05, 1.0].into(),
            color_1: [0.1, 0.23, 0.09, 1.0].into(),
            color_2: [0.12, 0.39, 0.15, 1.0].into(),
            tip: [0.7, 0.7, 0.7, 1.0].into(),
        }
    }
}


pub struct GrassDataBuffer {
    pub buffer: Buffer,
    pub length: usize,
}