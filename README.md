# bevy_procedural_grass

A plugin for `bevy 0.12` that generates grass on top of any mesh.

## Usage

```rust
use bevy::prelude::*;
use procedural_grass::{ProceduralGrassPlugin, grass::{grass::{GrassBundle, Grass}, mesh::GrassMesh}};

use noise::NoiseFn;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        ProceduralGrassPlugin::default(), // add grass plugin
    ))
    .add_systems(Startup, setup)
    .run();
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let plane = commands.spawn(
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.05, 0.0),
                reflectance: 0.0,
                
                ..default()
            }),
            transform: Transform::from_scale(Vec3::new(100.0, 3.0, 100.0)),
            ..default()
        }, 
    ).id();

    // spawn grass
    commands.spawn(GrassBundle {
        mesh: meshes.add(GrassMesh::mesh()),
        grass: Grass {
            entity: Some(terrain.clone()), // set entity that grass will generate on top of.
            ..default()
        },
        ..default()
    });
}
```

## TODO
- Lighting for point and spot lights (Currently only supporting directional lights).
- Improve Animation.
- Grass Clumping for less uniform grass generation.
- Grass Interaction, allow grass to move out of the way of other entites.
- Density Map.
- Local wind, allow separate grass entities to have their own wind.
- LOD
- Compute Shaders, use compute shaders to generate grass instance data each frame to optimize memory usage.

## Resources
- [Modern Foliage Rendering by Acerola](https://www.youtube.com/watch?v=jw00MbIJcrk)
- [Procedural Grass in 'Ghost of Tsushima' by GDC](https://www.youtube.com/watch?v=Ibe1JBF5i5Y)
- [Unity-Grass by cainrademan](https://github.com/cainrademan/Unity-Grass/)
- [warbler_grass by EmiOnGit](https://github.com/EmiOnGit/warbler_grass/)

#### 
