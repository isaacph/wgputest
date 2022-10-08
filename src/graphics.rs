use crate::{texture::Texture, camera::Camera, world::World};
use std::collections::hash_map::HashMap;
use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
        // self.view_proj = cgmath::Matrix4::identity().into();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0], },
    Vertex { position: [-0.5,  0.5, 0.0], tex_coords: [0.0, 1.0], },
    Vertex { position: [ 0.5,  0.5, 0.0], tex_coords: [1.0, 1.0], },
    Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 0.0], },
    Vertex { position: [ 0.5,  0.5, 0.0], tex_coords: [1.0, 1.0], },
    Vertex { position: [ 0.5, -0.5, 0.0], tex_coords: [1.0, 0.0], },
];

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

#[derive(Clone)]
pub struct Instance {
    position: cgmath::Vector2<f32>,
    scale: cgmath::Vector2<f32>,
    // rotation: f32,
    color: cgmath::Vector4<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        let translation = cgmath::Matrix4::from_translation(
            cgmath::Vector3::new(self.position.x, self.position.y, 0.0)
        );
        // let rotation = //cgmath::Matrix4::from_angle_z(cgmath::Rad(self.rotation));
        //             cgmath::Matrix4::identity();
        let scaling = cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0);
        let transf = translation * scaling;
        InstanceRaw {
            model: transf.into(),
        //    model: cgmath::Matrix4::identity().into(),
            color: self.color.into(),
        }
    }
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // for color
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct TextureBindGroups {
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: HashMap<String, wgpu::BindGroup>,
}

impl TextureBindGroups {
    pub fn new(texture_bind_group_layout: wgpu::BindGroupLayout) -> Self {
        Self {
            texture_bind_group_layout,
            texture_bind_groups: HashMap::new()
        }
    }

    pub fn make_texture_bind_group(&mut self, device: &wgpu::Device, texture: &Texture) {
        if !self.texture_bind_groups.contains_key(&texture.id) {
            // create bind group for texture
            let diffuse_bind_group = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &&self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                    ],
                    label: Some("diffuse_bind_group"),
                }
            );
            self.texture_bind_groups.insert(texture.id.clone(), diffuse_bind_group);
        }
    }

    pub fn get_texture_bind_group<'a>(&'a self, texture: &Texture) -> Option<&'a wgpu::BindGroup> {
        self.texture_bind_groups.get(&texture.id)
    }
}

pub struct TextureRenderer {
    render_pipeline: wgpu::RenderPipeline,

    square_vertex_buffer: wgpu::Buffer,
    square_num_vertices: u32,

    texture_bind_groups: TextureBindGroups,

    camera_bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,

    instance_buffer: wgpu::Buffer,
    num_instances: u32,

    // color_bind_group: wgpu::BindGroup,
    // color_buffer: wgpu::Buffer,
}

impl TextureRenderer {
    pub fn init(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        // create texture bind group layout, specifying how valid textures for this renderer are formatted
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        // create default square vertex buffer
        let square_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX
            }
        );
        let square_num_vertices = VERTICES.len() as u32;

        // create a camera: buffer + bind group
        // filler data before we specify a real camera later
        let camera_uniform_dummy = CameraUniform {
            view_proj: [[0.0; 4]; 4],
        };
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera_buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform_dummy]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &&camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        // create an instance buffer + bind group
        // this is a temp move, later we'll use real instancing
        let instance_uniform_dummy = InstanceRaw {
            model: [[0.0; 4]; 4],
            color: [0.0; 4]
        };
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("instance_buffer"),
                contents: bytemuck::cast_slice(&[instance_uniform_dummy]),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        // create color buffer + bind group
        // let color_uniform_dummy = ColorRaw {
        //     color: [0.0; 4],
        // };
        // let color_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("color_buffer"),
        //         contents: bytemuck::cast_slice(&[color_uniform_dummy]),
        //         usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        //     }
        // );
        // let color_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //     entries: &[
        //         wgpu::BindGroupLayoutEntry {
        //             binding: 3,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Buffer {
        //                 ty : wgpu::BufferBindingType::Uniform,
        //                 has_dynamic_offset: false,
        //                 min_binding_size: None,
        //             },
        //             count: None,
        //         }
        //     ],
        //     label: Some("instance_bind_group_layout"),
        // });
        // let instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &&instance_bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: instance_buffer.as_entire_binding(),
        //         }
        //     ],
        //     label: Some("instance_bind_group"),
        // });

        // create the render pipeline
        let render_pipeline = {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("texture_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("texture_shader.wgsl").into()),
            });

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                        &camera_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        Vertex::desc(),
                        InstanceRaw::desc()
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });
            render_pipeline
        };
        Self {
            render_pipeline,
            camera_buffer,
            camera_bind_group,
            texture_bind_groups: TextureBindGroups::new(texture_bind_group_layout),
            square_vertex_buffer,
            square_num_vertices,
            instance_buffer,
            num_instances: 0,
        }
    }

    pub fn render<'a>(&'a mut self, device: &mut wgpu::Device, queue: &mut wgpu::Queue, render_pass: &mut wgpu::RenderPass<'a>, camera: &Camera, instances: &Vec<Instance>, texture: &Texture) -> Result<(), wgpu::SurfaceError> {
        // update the camera
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

        // add all needed textures to cache
        self.texture_bind_groups.make_texture_bind_group(device, texture);
        // retrieve bind group for the given texture
        let diffuse_bind_group = self.texture_bind_groups.get_texture_bind_group(texture)
            .expect("Could not find texture bind group (should be impossible");

        // make and set instance buffer, resetting if the number of instances changes
        let buffer = instances.iter().map(|instance| instance.to_raw()).collect::<Vec<_>>();
        let buffer_bytes = bytemuck::cast_slice(&buffer);
        if self.num_instances != instances.len() as u32 {
            self.instance_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("instance_buffer_rewrite"),
                    contents: buffer_bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                }
            );
        } else {
            queue.write_buffer(&self.instance_buffer, 0, buffer_bytes);
        }

        // set up for drawing, shared state across instances
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.square_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.draw(0..self.square_num_vertices, 0..(instances.len() as u32));
        Ok(())
    }
}

pub struct RenderPrereq<'a> {
    pub device: &'a mut wgpu::Device,
    pub queue: &'a mut wgpu::Queue,
    pub surface: &'a mut wgpu::Surface,
    pub camera: &'a Camera,
}

pub struct RenderEngine {
    pub texture_renderer: TextureRenderer,
    pub texture_renderer2: TextureRenderer,
    pub diffuse_texture: Texture,
    pub solid_texture: Texture,
}

impl RenderEngine {
    pub fn init(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> RenderEngine {
        // load a texture - happy tree
        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_texture =
            Texture::from_image(&device, &queue, &diffuse_image, "happy-tree").unwrap();
        let solid_texture =
            Texture::blank_texture(&device, &queue, "blank").unwrap();
        Self {
            texture_renderer: TextureRenderer::init(device, queue, config),
            texture_renderer2: TextureRenderer::init(device, queue, config),
            diffuse_texture,
            solid_texture,
        }
    }

    pub fn render(&mut self, render: RenderPrereq, world: &World) -> Result<(), wgpu::SurfaceError> {
        let output = render.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = render.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: true,
                        },
                    })
                ],
                depth_stencil_attachment: None,
            });

            // render instructions go here
            let instances = world.objects.iter().map(|obj| {
                Instance {
                    position: obj.position,
                    scale: obj.scale,
                    color: obj.color,
                }
            }).collect::<Vec<_>>();
            self.texture_renderer.render(render.device, render.queue, &mut render_pass, render.camera, &instances, &self.solid_texture)?;
            // when we add font rendering, time to fight with the borrow checker, probably
        }

        // submit will accept anything that implements IntoIter
        render.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
