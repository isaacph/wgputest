use crate::{graphics::texture::Texture, camera::Camera, world::World};
use self::{textured::{TextureRenderer, Instance}, text::{Font, FontRenderer, make_font_infos, default_characters}};

pub mod textured;
pub mod text;
pub mod texture;

pub struct RenderPrereq<'a> {
    pub device: &'a mut wgpu::Device,
    pub queue: &'a mut wgpu::Queue,
    pub surface: &'a mut wgpu::Surface,
    pub camera: &'a Camera,
}

pub struct RenderEngine {
    pub texture_renderer: TextureRenderer,
    pub diffuse_texture: Texture,
    pub solid_texture: Texture,
    font: Font,
    font_renderer: FontRenderer,
}

impl RenderEngine {
    pub fn init(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> RenderEngine {
        // load a texture - happy tree
        let diffuse_bytes = include_bytes!("happy-tree.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_texture =
            Texture::from_image(&device, &queue, &diffuse_image, "happy-tree", wgpu::FilterMode::Linear).unwrap();
        let solid_texture =
            Texture::blank_texture(&device, &queue, "blank").unwrap();
        let mut texture_renderer = TextureRenderer::init(device, queue, config);
        texture_renderer.add_texture(device, [&diffuse_texture, &solid_texture].into_iter());
        let font_info = make_font_infos(
            include_bytes!("arial.ttf"), &[48.0], default_characters().iter(), None, "arial".to_string()).unwrap();
        let font = Font::make_from_info(device, queue, &font_info[0], wgpu::FilterMode::Linear).unwrap();
        let mut font_renderer = FontRenderer::new(device, queue, config).unwrap();
        font_renderer.register_font(device, &font);
        Self {
            texture_renderer,
            diffuse_texture,
            solid_texture,
            font,
            font_renderer,
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
            self.texture_renderer.reset();
            self.font_renderer.reset();

            // render instructions go here
            let mut instances = vec![
                Instance {
                    position: world.player.physics.bounding_box.center,
                    scale: world.player.physics.bounding_box.get_scale(),
                    color: cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                },
            ];
            instances.extend(world.stage.iter().map(|stage|
                Instance {
                    position: stage.physics.bounding_box.center,
                    scale: stage.physics.bounding_box.get_scale(),
                    color: cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                })
            );
            self.texture_renderer.render(render.queue, &mut render_pass, render.camera,
                vec![
                   (
                       instances[0..instances.len()].to_vec(),
                       &self.solid_texture
                   ),
                   (
                       world.debug_objects.clone(),
                       &self.solid_texture
                   ),
                ])?;

            let text = format!("Text");
            self.font_renderer.render(&self.font, render.queue, &mut render_pass, &render.camera,
                  &vec![
                (text.clone(),
                cgmath::Vector2::new(0.0, 32.0),
                cgmath::Vector4::new(1.0, 0.5, 1.0, 1.0)),
            ])?;
        }

        // submit will accept anything that implements IntoIter
        render.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
