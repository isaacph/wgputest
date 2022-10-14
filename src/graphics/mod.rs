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
            let instances = world.objects.iter().map(|obj| {
                Instance {
                    position: obj.position,
                    scale: obj.scale,
                    color: obj.color,
                }
            }).collect::<Vec<_>>();
            self.texture_renderer.render(render.queue, &mut render_pass, render.camera,
                vec![
                   (
                       instances[instances.len()/2..].to_vec(),
                       &self.diffuse_texture
                   ),
                ])?;
            let text = format!("I should be behind stuff");
            self.font_renderer.render(&self.font, render.queue, &mut render_pass, &render.camera,
                  &vec![
                (text.clone(),
                cgmath::Vector2::new(200.0, 400.0),
                cgmath::Vector4::new(1.0, 0.5, 1.0, 1.0)),
                ("I am right of something".to_string(),
                cgmath::Vector2::new(200.0 + self.font.text_width(&text), 400.0),
                cgmath::Vector4::new(1.0, 0.0, 0.0, 1.0))
            ])?;
            self.texture_renderer.render(render.queue, &mut render_pass, render.camera,
                vec![
                   (
                       instances[0..instances.len()/2].to_vec(),
                       &self.solid_texture
                   ),
                ])?;
            let long_text = (0..30).map(|_| format!("I fit on the screen horizontally "))
                .collect::<Vec<_>>().join("");
            let long_text_split = self.font.split_lines(long_text.as_str(), Some(render.camera.window_size.x as f32));
            let long_text_join = long_text_split.join("\n");
            self.font_renderer.render(&self.font, render.queue, &mut render_pass, &render.camera,
                  &vec![
                (format!("Hello World!"),
                cgmath::Vector2::new(100.0, 100.0),
                cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0)),
                (format!("What's up? We got instancing,\nnew lines, colors, etc"),
                cgmath::Vector2::new(80.0, 200.0),
                cgmath::Vector4::new(1.0, 0.5, 0.0, 1.0)),
                (long_text_join,
                cgmath::Vector2::new(0.0, 48.0),
                cgmath::Vector4::new(0.0, 1.0, 0.0, 0.1))
            ])?;
            // when we add font rendering, time to fight with the borrow checker, probably
        }

        // submit will accept anything that implements IntoIter
        render.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}