use bevy::{prelude::*, pbr::{MeshPipeline, MeshPipelineKey}, render::{render_resource::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, BufferBindingType, SpecializedMeshPipeline, RenderPipelineDescriptor, SpecializedMeshPipelineError, VertexBufferLayout, VertexStepMode, VertexAttribute, VertexFormat}, renderer::RenderDevice, mesh::MeshVertexBufferLayout}};

use super::grass::InstanceData;

#[derive(Resource)]
pub struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    pub color_layout: BindGroupLayout,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("shaders/instancing.wgsl");
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let mesh_pipeline = world.resource::<MeshPipeline>();

        let color_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("grass_color_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        CustomPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
            color_layout,
        }
    }
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        descriptor
            .vertex
            .shader_defs
            .push("MESH_BINDGROUP_1".into());

        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 3,
                },
            ],
        });
        descriptor.layout.push(self.color_layout.clone());

        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.primitive.cull_mode = None;
        Ok(descriptor)
    }
}