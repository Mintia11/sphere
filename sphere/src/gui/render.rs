use std::{
    collections::{HashMap, HashSet},
    mem::offset_of,
    sync::Arc,
};

use egui::{TextureId, epaint::Primitive};
use etna::{
    Device, Image, Swapchain,
    command_buffer::CommandBuffer,
    dynamic_buffer::DynamicBuffer,
    error::Error,
    gpu_allocator::MemoryLocation,
    vk::{self, TaggedStructure},
};

use crate::gui::RenderData;

pub struct EguiRenderer {
    device: Arc<Device>,

    sampler: vk::Sampler,
    images: HashMap<TextureId, Image>,

    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,

    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    texture_descriptors: HashMap<TextureId, vk::DescriptorSet>,

    vertex_buffer: DynamicBuffer,
    index_buffer: DynamicBuffer,
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct EguiVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

struct DrawCmd {
    texture_id: TextureId,
    clip_rect: egui::Rect,
    index_offset: u32,
    index_count: u32,
    vertex_offset: i32,
}

impl EguiRenderer {
    pub fn new(device: &Arc<Device>, swapchain: &Swapchain) -> Result<Self, Error> {
        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let descriptor_set_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(&binding));

        let descriptor_set_layout = unsafe {
            device
                .handle()
                .create_descriptor_set_layout(&descriptor_set_layout_info, None)?
        };

        let push_constant_range = vk::PushConstantRange::default()
            .offset(0)
            .size(size_of::<[f32; 2]>() as u32)
            .stage_flags(vk::ShaderStageFlags::VERTEX);
        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(std::slice::from_ref(&push_constant_range))
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));

        let layout = unsafe { device.handle().create_pipeline_layout(&layout_info, None)? };

        let vert = device.create_shader(
            "main",
            vk::ShaderStageFlags::VERTEX,
            include_bytes!("egui.vert.spv"),
        )?;
        let frag = device.create_shader(
            "main",
            vk::ShaderStageFlags::FRAGMENT,
            include_bytes!("egui.frag.spv"),
        )?;
        let stages = [vert.pipeline_shader_stage(), frag.pipeline_shader_stage()];

        let vertex_attribute_descriptions = [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .location(0)
                .offset(offset_of!(EguiVertex, pos) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .location(1)
                .offset(offset_of!(EguiVertex, uv) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .location(2)
                .offset(offset_of!(EguiVertex, color) as u32),
        ];
        let vertex_binding_descriptions = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX)
            .stride(size_of::<EguiVertex>() as u32)];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&vertex_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_binding_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let rasterization_state =
            vk::PipelineRasterizationStateCreateInfo::default().line_width(1.0);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .scissor_count(1)
            .viewport_count(1);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA);
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let format = swapchain.current_format();
        let mut rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(std::slice::from_ref(&format));

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .multisample_state(&multisample_state)
            .rasterization_state(&rasterization_state)
            .dynamic_state(&dynamic_state)
            .viewport_state(&viewport_state)
            .layout(layout)
            .color_blend_state(&color_blend_state)
            .push(&mut rendering_info);

        let pipeline = unsafe {
            device
                .handle()
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .map_err(|(_, e)| e)?
        };

        let pool_size = vk::DescriptorPoolSize::default()
            .descriptor_count(1)
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER);
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(4096)
            .pool_sizes(std::slice::from_ref(&pool_size));
        let descriptor_pool = unsafe { device.handle().create_descriptor_pool(&pool_info, None)? };

        let sampler_info = vk::SamplerCreateInfo::default();
        let sampler = unsafe { device.handle().create_sampler(&sampler_info, None)? };

        Ok(Self {
            device: device.clone(),

            sampler,
            images: HashMap::new(),

            pipeline: pipeline[0],
            layout,

            descriptor_pool,
            descriptor_set_layout,
            texture_descriptors: HashMap::new(),

            vertex_buffer: DynamicBuffer::new(
                device,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                4096 * 1024,
            )?,
            index_buffer: DynamicBuffer::new(
                device,
                vk::BufferUsageFlags::INDEX_BUFFER,
                4096 * 1024,
            )?,
        })
    }

    pub fn draw(
        &mut self,
        command_buffer: &CommandBuffer,
        image: &Image,
        mut data: RenderData,
    ) -> Result<(), Error> {
        for (&id, delta) in &data.textures_delta.set {
            for delta in delta {
                self.update_textures(id, delta)?;
            }
        }

        data.textures_delta.clear();

        unsafe {
            self.device.handle().cmd_bind_pipeline(
                command_buffer.handle(),
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            command_buffer.image_pipeline_barrier(
                image,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
                vk::PipelineStageFlags2::CLEAR,
                vk::PipelineStageFlags2::ALL_GRAPHICS,
            );

            let mut used_textures = HashSet::new();
            for prim in &data.clipped_primitives {
                let texture = match &prim.primitive {
                    Primitive::Mesh(mesh) => mesh.texture_id,
                    _ => continue,
                };

                used_textures.insert(texture);
            }

            for texture in used_textures {
                let image = &self.images[&texture];
                if image.current_layout() != vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
                    command_buffer.image_pipeline_barrier(
                        &self.images[&texture],
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        vk::AccessFlags2::TRANSFER_WRITE,
                        vk::AccessFlags2::SHADER_SAMPLED_READ,
                        vk::PipelineStageFlags2::COPY,
                        vk::PipelineStageFlags2::FRAGMENT_SHADER,
                    );
                }
            }

            let framebuffer = vk::RenderingAttachmentInfo::default()
                .load_op(vk::AttachmentLoadOp::LOAD)
                .store_op(vk::AttachmentStoreOp::STORE)
                .image_view(image.view())
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
            let rendering_info = vk::RenderingInfo::default()
                .color_attachments(std::slice::from_ref(&framebuffer))
                .layer_count(1)
                .render_area(
                    vk::Rect2D::default().extent(
                        vk::Extent2D::default()
                            .width(image.extent().width)
                            .height(image.extent().height),
                    ),
                );
            self.device
                .handle()
                .cmd_begin_rendering(command_buffer.handle(), &rendering_info);

            let mut vertex_bytes = Vec::new();
            let mut index_bytes = Vec::new();
            let mut draw_cmds = Vec::new();

            let mut vertex_cursor = 0i32;
            let mut index_cursor = 0u32;

            for prim in &data.clipped_primitives {
                let Primitive::Mesh(mesh) = &prim.primitive else {
                    continue;
                };

                for v in &mesh.vertices {
                    let vertex = EguiVertex {
                        pos: [v.pos.x, v.pos.y],
                        uv: [v.uv.x, v.uv.y],
                        color: v.color.to_normalized_gamma_f32(),
                    };
                    vertex_bytes.extend_from_slice(bytemuck::bytes_of(&vertex));
                }
                for &i in &mesh.indices {
                    index_bytes.extend_from_slice(&i.to_ne_bytes());
                }

                draw_cmds.push(DrawCmd {
                    texture_id: mesh.texture_id,
                    clip_rect: prim.clip_rect,
                    index_offset: index_cursor,
                    index_count: mesh.indices.len() as u32,
                    vertex_offset: vertex_cursor,
                });

                vertex_cursor += mesh.vertices.len() as i32;
                index_cursor += mesh.indices.len() as u32;
            }

            self.vertex_buffer
                .ensure_capacity(&self.device, vertex_bytes.len())?;
            self.index_buffer
                .ensure_capacity(&self.device, index_bytes.len())?;
            self.vertex_buffer.write(0, &vertex_bytes)?;
            self.index_buffer.write(0, &index_bytes)?;

            self.device.handle().cmd_bind_vertex_buffers(
                command_buffer.handle(),
                0,
                &[self.vertex_buffer.handle()],
                &[0],
            );
            self.device.handle().cmd_bind_index_buffer(
                command_buffer.handle(),
                self.index_buffer.handle(),
                0,
                vk::IndexType::UINT32,
            );

            self.device.handle().cmd_push_constants(
                command_buffer.handle(),
                self.layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                bytemuck::bytes_of(&data.screen_size),
            );
            self.device.handle().cmd_set_viewport(
                command_buffer.handle(),
                0,
                std::slice::from_ref(
                    &vk::Viewport::default()
                        .width(image.extent().width as f32)
                        .height(image.extent().height as f32),
                ),
            );

            for cmd in &draw_cmds {
                // scissor from cmd.clip_rect (finally using it correctly)
                let scissor =
                    clip_rect_to_scissor(cmd.clip_rect, data.pixels_per_point, image.extent());
                self.device
                    .handle()
                    .cmd_set_scissor(command_buffer.handle(), 0, &[scissor]);

                let descriptor_set = self.texture_descriptors[&cmd.texture_id];
                self.device.handle().cmd_bind_descriptor_sets(
                    command_buffer.handle(),
                    vk::PipelineBindPoint::GRAPHICS,
                    self.layout,
                    0,
                    &[descriptor_set],
                    &[],
                );

                self.device.handle().cmd_draw_indexed(
                    command_buffer.handle(),
                    cmd.index_count,
                    1,
                    cmd.index_offset,
                    cmd.vertex_offset,
                    0,
                );
            }

            self.device
                .handle()
                .cmd_end_rendering(command_buffer.handle());
        }

        Ok(())
    }

    fn update_textures(
        &mut self,
        id: egui::TextureId,
        delta: &egui::epaint::ImageDelta,
    ) -> Result<(), Error> {
        match &delta.image {
            egui::ImageData::Color(image) => {
                let data: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                if let Some(pos) = delta.pos {
                    let existing = self
                        .images
                        .get_mut(&id)
                        .expect("partial texture update for unknown TextureId");
                    existing.upload_region(
                        &data,
                        [pos[0] as u32, pos[1] as u32],
                        [image.width() as u32, image.height() as u32],
                    )?;
                } else {
                    let image = self.device.create_image_and_upload(
                        image.width() as u32,
                        image.height() as u32,
                        &data,
                        vk::Format::R8G8B8A8_SRGB,
                        vk::ImageUsageFlags::SAMPLED,
                        MemoryLocation::GpuOnly,
                    )?;

                    self.images.insert(id, image);
                    self.ensure_descriptor(id)?;
                }
            }
        }

        Ok(())
    }

    fn ensure_descriptor(&mut self, id: TextureId) -> Result<vk::DescriptorSet, Error> {
        if let Some(&set) = self.texture_descriptors.get(&id) {
            return Ok(set);
        }

        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.descriptor_pool)
            .set_layouts(std::slice::from_ref(&self.descriptor_set_layout));
        let descriptor_set = unsafe {
            self.device
                .handle()
                .allocate_descriptor_sets(&allocate_info)?[0]
        };

        self.write_descriptor(descriptor_set, id);
        self.texture_descriptors.insert(id, descriptor_set);
        Ok(descriptor_set)
    }

    fn write_descriptor(&self, descriptor_set: vk::DescriptorSet, id: TextureId) {
        let image = &self.images[&id];
        let image_info = vk::DescriptorImageInfo::default()
            .image_view(image.view())
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .sampler(self.sampler);
        let descriptor_write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));

        unsafe {
            self.device
                .handle()
                .update_descriptor_sets(&[descriptor_write], &[]);
        }
    }
}

fn clip_rect_to_scissor(
    clip_rect: egui::Rect,
    pixels_per_point: f32,
    framebuffer_extent: vk::Extent3D,
) -> vk::Rect2D {
    let min_x = (clip_rect.min.x * pixels_per_point).round() as i32;
    let min_y = (clip_rect.min.y * pixels_per_point).round() as i32;
    let max_x = (clip_rect.max.x * pixels_per_point).round() as i32;
    let max_y = (clip_rect.max.y * pixels_per_point).round() as i32;

    let fb_width = framebuffer_extent.width as i32;
    let fb_height = framebuffer_extent.height as i32;

    let min_x = min_x.clamp(0, fb_width);
    let min_y = min_y.clamp(0, fb_height);
    let max_x = max_x.clamp(min_x, fb_width);
    let max_y = max_y.clamp(min_y, fb_height);

    vk::Rect2D {
        offset: vk::Offset2D { x: min_x, y: min_y },
        extent: vk::Extent2D {
            width: (max_x - min_x) as u32,
            height: (max_y - min_y) as u32,
        },
    }
}
