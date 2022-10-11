use crate::{texture::Texture, camera::Camera, world::World};
use self::textured::{TextureRenderer, Instance};

pub mod textured;
pub mod text;

pub struct RenderPrereq<'a> {
    pub device: &'a mut wgpu::Device,
    pub queue: &'a mut wgpu::Queue,
    pub surface: &'a mut wgpu::Surface,
    pub camera: &'a Camera,
}

pub struct RenderEngine {
    pub instance_list: textured::InstanceList,
    pub texture_renderer: TextureRenderer,
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
        let mut texture_renderer = TextureRenderer::init(device, queue, config);
        texture_renderer.prep_textures(device, &[&diffuse_texture, &solid_texture]);
        let instance_list = textured::InstanceList::new(device, queue);
        Self {
            instance_list,
            texture_renderer,
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

            let instances = world.objects.iter().map(|obj| {
                Instance {
                    position: obj.position,
                    scale: obj.scale,
                    color: obj.color,
                }
            }).collect::<Vec<_>>();
            self.instance_list.reset();
            let range = self.instance_list.fill_next_buffers(
                render.queue,
                vec![
                    instances[instances.len()/2..].to_vec(),
                    instances[0..instances.len()/2].to_vec(),
                ]
            );
            let instance_buffers = self.instance_list.get_buffers(range);

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
            let instance_buffers = vec![(instance_buffers[0].clone(), &self.diffuse_texture), (instance_buffers[1].clone(), &self.solid_texture)];
            self.texture_renderer.render(&mut render_pass, instance_buffers[0].clone())?;
            // when we add font rendering, time to fight with the borrow checker, probably
        }

        // submit will accept anything that implements IntoIter
        render.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
