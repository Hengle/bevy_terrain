use crate::{
    render::{layouts::CONFIG_BUFFER_SIZE, resources::TerrainResources, PersistentComponents},
    GpuNodeAtlas, TerrainRenderPipeline,
};
use bevy::{
    prelude::*,
    render::{render_resource::*, renderer::RenderDevice},
};

pub struct TerrainRenderData {
    pub(crate) indirect_buffer: Buffer,
    pub(crate) terrain_data_bind_group: BindGroup,
    pub(crate) patch_list_bind_group: BindGroup,
}

impl TerrainRenderData {
    fn new(
        device: &RenderDevice,
        resources: &TerrainResources,
        gpu_node_atlas: &GpuNodeAtlas,
        terrain_pipeline: &mut TerrainRenderPipeline,
    ) -> Self {
        let terrain_data_layout = Self::create_terrain_data_layout(device, gpu_node_atlas);
        let terrain_data_bind_group = Self::create_terrain_data_bind_group(
            device,
            resources,
            gpu_node_atlas,
            &terrain_data_layout,
        );
        let patch_list_bind_group = Self::create_patch_list_bind_group(
            device,
            resources,
            &terrain_pipeline.patch_list_layout,
        );

        terrain_pipeline
            .terrain_data_layouts
            .push(terrain_data_layout);

        Self {
            indirect_buffer: resources.indirect_buffer.clone(),
            terrain_data_bind_group,
            patch_list_bind_group,
        }
    }

    fn create_terrain_data_layout(
        device: &RenderDevice,
        gpu_node_atlas: &GpuNodeAtlas,
    ) -> BindGroupLayout {
        let mut entries = vec![
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(CONFIG_BUFFER_SIZE),
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Uint,
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ];

        entries.extend(
            gpu_node_atlas
                .atlas_attachments
                .values()
                .map(|attachment| attachment.layout()),
        );

        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        })
    }

    fn create_terrain_data_bind_group(
        device: &RenderDevice,
        resources: &TerrainResources,
        gpu_node_atlas: &GpuNodeAtlas,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let mut entries = vec![
            BindGroupEntry {
                binding: 0,
                resource: resources.config_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&resources.atlas_map_view),
            },
        ];

        entries.extend(
            gpu_node_atlas
                .atlas_attachments
                .values()
                .map(|attachment| attachment.binding()),
        );

        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            entries: &entries,
            layout,
        })
    }

    fn create_patch_list_bind_group(
        device: &RenderDevice,
        resources: &TerrainResources,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: resources.patch_buffer.as_entire_binding(),
            }],
            layout,
        })
    }
}

/// Runs in queue.
pub(crate) fn initialize_terrain_render_data(
    device: Res<RenderDevice>,
    mut terrain_pipeline: ResMut<TerrainRenderPipeline>,
    gpu_node_atlases: Res<PersistentComponents<GpuNodeAtlas>>,
    mut terrain_render_data: ResMut<PersistentComponents<TerrainRenderData>>,
    terrain_query: Query<(Entity, &TerrainResources)>,
) {
    for (entity, resources) in terrain_query.iter() {
        let gpu_node_atlas = gpu_node_atlases.get(&entity).unwrap();

        terrain_render_data.insert(
            entity,
            TerrainRenderData::new(&device, &resources, gpu_node_atlas, &mut terrain_pipeline),
        );
    }
}