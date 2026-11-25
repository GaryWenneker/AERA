//! GPU material preview using eframe's existing wgpu device.
use eframe::egui;
use eframe::egui_wgpu::{self, wgpu, CallbackResources, CallbackTrait, ScreenDescriptor};

const WIDTH: u32 = 768;
const HEIGHT: u32 = 384;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

struct Resources {
    _textures: Vec<wgpu::Texture>,
    views: Vec<wgpu::TextureView>,
    groups: Vec<wgpu::BindGroup>,
    pipelines: Vec<wgpu::RenderPipeline>,
    uniform: wgpu::Buffer,
    scene_key: Option<[u32; 3]>,
}

fn texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("AERA Glass intermediate"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

pub fn install(state: &egui_wgpu::RenderState) {
    let device = &state.device;
    let uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Glass parameters"),
        size: 32,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    // Separate dummy input prevents attachment/sampling aliasing in the scene pass.
    let textures = vec![
        texture(device, 1, 1),
        texture(device, WIDTH, HEIGHT),
        texture(device, WIDTH / 2, HEIGHT / 2),
        texture(device, WIDTH, HEIGHT),
    ];
    let views: Vec<_> = textures
        .iter()
        .map(|t| t.create_view(&Default::default()))
        .collect();
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Glass inputs"),
        entries: &[
            texture_entry(0),
            texture_entry(1),
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let groups = [(0, 0), (1, 0), (2, 0), (1, 3)]
        .into_iter()
        .map(|(source, filtered)| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Glass pass inputs"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&views[source]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&views[filtered]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: uniform.as_entire_binding(),
                    },
                ],
            })
        })
        .collect();
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Glass pipeline layout"),
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("AERA Glass WGSL"),
        source: wgpu::ShaderSource::Wgsl(include_str!("glass.wgsl").into()),
    });
    let pipelines = ["scene", "down", "up", "glass"]
        .into_iter()
        .map(|entry| {
            let format = if entry == "glass" {
                state.target_format
            } else {
                FORMAT
            };
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        })
        .collect();
    state.renderer.write().callback_resources.insert(Resources {
        _textures: textures,
        views,
        groups,
        pipelines,
        uniform,
        scene_key: None,
    });
}

struct Material {
    values: [f32; 8],
}

impl CallbackTrait for Material {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen: &ScreenDescriptor,
        encoder: &mut wgpu::CommandEncoder,
        resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(r) = resources.get_mut::<Resources>() else {
            return Vec::new();
        };
        let bytes: Vec<u8> = self.values.iter().flat_map(|v| v.to_le_bytes()).collect();
        queue.write_buffer(&r.uniform, 0, &bytes);
        // Pointer and morph changes affect only the final optical pass.
        let scene_key = [
            self.values[3].to_bits(),
            self.values[4].to_bits(),
            self.values[5].to_bits(),
        ];
        if r.scene_key == Some(scene_key) {
            return Vec::new();
        }
        r.scene_key = Some(scene_key);
        for i in 0..3 {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Glass intermediate pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &r.views[i + 1],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&r.pipelines[i]);
            pass.set_bind_group(0, &r.groups[i], &[]);
            pass.draw(0..3, 0..1);
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        if let Some(r) = resources.get::<Resources>() {
            pass.set_pipeline(&r.pipelines[3]);
            pass.set_bind_group(0, &r.groups[3], &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

pub fn preview(ui: &mut egui::Ui, dark: bool, quality: f32, reduced_motion: bool) {
    let width = ui.available_width().clamp(100.0, 768.0);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, width * 0.5), egui::Sense::hover());
    let pointer = response.hover_pos().unwrap_or(rect.center_top());
    let uv = (pointer - rect.min) / rect.size();
    let radius = if reduced_motion {
        0.38
    } else {
        ui.ctx()
            .animate_bool_with_time(response.id, response.hovered(), 0.22)
            * 0.04
            + 0.36
    };
    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
        rect,
        Material {
            values: [
                uv.x,
                uv.y,
                radius,
                if dark { 1.0 } else { 0.0 },
                2.0,
                quality.clamp(0.0, 1.0),
                0.0,
                0.0,
            ],
        },
    ));
}
