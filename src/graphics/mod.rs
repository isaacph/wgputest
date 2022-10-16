use cgmath::{Vector2, Vector4};

use crate::{graphics::texture::Texture, camera::Camera, world::{World, physics::Physics}, chatbox::Chatbox};
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
    pub ui_texture_renderer: TextureRenderer,
    pub font: Font,
    font_renderer: FontRenderer,

    pub diffuse_texture: Texture,
    pub solid_texture: Texture,
    pub background_texture: Texture,
    pub basic_texture: Texture,
    pub player_texture: Texture,
    pub spearman_texture: Texture,
    pub tile_texture: Texture,
    pub tile_stained_glass: Texture,
    pub red_ball_texture: Texture,
}

#[derive(Clone)]
pub struct ResolveInstance {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>,
    pub overlaps: i32,
}

impl Into<Instance> for ResolveInstance {
    fn into(self) -> Instance {
        Instance {
            position: self.position,
            scale: self.scale,
            color: self.color
        }
    }
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
        let mut ui_texture_renderer = TextureRenderer::init(device, queue, config);
        ui_texture_renderer.add_texture(device, [&diffuse_texture, &solid_texture].into_iter());
        let font_info = make_font_infos(
            include_bytes!("arial.ttf"), &[48.0], default_characters().iter(), None, "arial".to_string()).unwrap();
        let font = Font::make_from_info(device, queue, &font_info[0], wgpu::FilterMode::Linear).unwrap();
        let mut font_renderer = FontRenderer::new(device, queue, config).unwrap();
        font_renderer.register_font(device, &font);

        let load_image = |bytes, s| {
            let diffuse_bytes = bytes;
            let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
            let diffuse_texture =
                Texture::from_image(&device, &queue, &diffuse_image, s, wgpu::FilterMode::Nearest).unwrap();
            diffuse_texture
        };

        let background_texture = load_image(include_bytes!("background.png"), "background.png");
        let basic_texture = load_image(include_bytes!("basic.png"), "basic.png");
        let player_texture = load_image(include_bytes!("player.png"), "player.png");
        let spearman_texture = load_image(include_bytes!("spearman.png"), "spearman.png");
        let tile_texture = load_image(include_bytes!("tile_glass_holy.png"), "tile_glass_holy.png");
        let tile_stained_glass = load_image(include_bytes!("tile_stained_glass.png"), "tile_stained_glass.png");
        let red_ball_texture = load_image(include_bytes!("red_ball.png"), "red_ball.png");
        texture_renderer.add_texture(
            device,
            vec![
                &background_texture,
                &basic_texture,
                &player_texture,
                &spearman_texture,
                &tile_texture,
                &tile_stained_glass,
                &red_ball_texture,
            ].into_iter());
        ui_texture_renderer.add_texture(
            device,
            vec![&background_texture].into_iter());

        Self {
            texture_renderer,
            diffuse_texture,
            solid_texture,
            font,
            font_renderer,
            ui_texture_renderer,
            background_texture,
            basic_texture,
            player_texture,
            spearman_texture,
            tile_texture,
            tile_stained_glass,
            red_ball_texture,
        }
    }

    pub fn render(&mut self, render: RenderPrereq, chatbox: &Chatbox, world: &World) -> Result<(), wgpu::SurfaceError> {
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
            self.ui_texture_renderer.reset();

            let ui_camera = render.camera.get_ui_camera();

            // render instructions go here
            // render background
            self.ui_texture_renderer.render(
                render.queue,
                &mut render_pass,
                &ui_camera,
                vec![
                    (vec![Instance {
                        position: Vector2::new(render.camera.window_size.x as f32 / 2.0,
                                               render.camera.window_size.y as f32 / 2.0),
                        scale: Vector2::new(render.camera.window_size.x as f32 / 1.0,
                                               render.camera.window_size.y as f32 / 1.0),
                        color: Vector4::new(1.0, 1.0, 1.0, 1.0),
                    }], &self.background_texture)
                ]
            )?;

            let mut instances = vec![(
                vec![Instance {
                    position: world.player.physics.bounding_box.center,
                    scale: Vector2::new(world.player.physics.bounding_box.get_scale().y, world.player.physics.bounding_box.get_scale().y),
                    color: cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0),
                }], &self.player_texture),
            ];
            instances.push((world.stage.iter().map(|(_, stage)| stage.get_physics()).flatten().map(|(_, phys)|
                Instance {
                    position: phys.bounding_box.center,
                    scale: phys.bounding_box.get_scale(),
                    color: cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0),
                }).collect(), &self.tile_texture)
            );
            instances.push((world.projectiles.iter().map(|projectile| projectile.get_physics()).flatten().map(|(_, phys)|
                Instance {
                    position: phys.bounding_box.center,
                    scale: phys.bounding_box.get_scale(),
                    color: cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0),
                }).collect(), &self.red_ball_texture
            ));
            instances.push((world.basic_enemies.iter().map(|enemy| enemy.get_physics()).flatten().map(|(_, phys)|
                Instance {
                    position: phys.bounding_box.center + Vector2::new(0.0, -0.15),
                    scale: Vector2::new(phys.bounding_box.height, phys.bounding_box.height) * 1.25,
                    color: cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0),
                }).collect(), &self.basic_texture)
            );
            self.texture_renderer.render(render.queue, &mut render_pass, render.camera,
                instances)?;

            let text = format!("{:?}\n{}", world.player.aerial_state, world.player.physics.velocity.y);
            let mut font_instances = vec![(text.clone(),
                    cgmath::Vector2::new(0.0, 38.0),
                    cgmath::Vector4::new(1.0, 0.5, 1.0, 1.0))];
            font_instances.extend(world.debug_objects.iter().map(|i| {
                (
                    format!("{}", i.overlaps),
                    render.camera.world_to_view_pos(cgmath::Point2::new(0.0, 0.0) + i.position),
                    cgmath::Vector4::new(i.color.x, i.color.y, i.color.z, 0.8)
                )
            }));
            self.font_renderer.render(&self.font, render.queue, &mut render_pass, &ui_camera,
                &font_instances)?;
            
            // copying code above for the purpose of horizontal info
            let text = format!("{:?}\n{}", world.player.horizontal_state, world.player.physics.velocity.x);
            let font_instances = vec![(text.clone(),
                    cgmath::Vector2::new(400.0, 38.0),
                    cgmath::Vector4::new(1.0, 0.5, 1.0, 1.0))];
            // font_instances.extend(world.debug_objects.iter().map(|i| {
            //     (
            //         format!("{}", i.overlaps),
            //         render.camera.world_to_view_pos(cgmath::Point2::new(0.0, 0.0) + i.position),
            //         cgmath::Vector4::new(i.color.x, i.color.y, i.color.z, 0.8)
            //     )
            // }));
            self.font_renderer.render(&self.font, render.queue, &mut render_pass, &ui_camera,
                &font_instances)?;

            // render ui
            // render chatbox
            let (background_instance, chatbox_text_instances) =
                chatbox.render();

            self.ui_texture_renderer.render(
                render.queue,
                &mut render_pass,
                &ui_camera,
                vec![
                    (vec![background_instance], &self.solid_texture)
                ]
            )?;

            self.font_renderer.render(
                &self.font,
                render.queue,
                &mut render_pass,
                &ui_camera,
                &chatbox_text_instances
            )?;
        }

        // submit will accept anything that implements IntoIter
        render.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
