use std::{collections::HashMap, cell::Cell};
use crate::camera::CameraObj;
use crate::{util::PartialOrdMinMax, camera::Camera};
use crate::graphics::texture::Texture;

use self::packing::{GlyphPacking, GlyphInfo};
use cgmath::{Matrix4, Vector2, SquareMatrix};
use wgpu::util::DeviceExt;

mod packing;

pub const MAX_GLYPHS_PER_FRAME: u32 = 8192;

pub fn default_characters() -> Vec<char> {
    let mut chars = vec![0];
    chars.append(&mut (32..127).collect::<Vec<u32>>());
    chars.iter().map(|i| char::from_u32(*i).unwrap()).collect()
}

#[derive(Clone)]
pub struct GlyphMetrics {
    pub glyph_pos: Vector2<f32>,
    pub glyph_size: Vector2<f32>,
    pub advance: f32,
    pub lsb: f32, // left side bearing
    pub tsb: f32, // top side bearing
}

struct GlyphBitmap {
    width: usize,
    height: usize,
    buffer: Vec<u8>,
    metrics: fontdue::Metrics,
    char_code: char,
}

pub struct FontInfo {
    pub image_buffer: Vec<u8>,
    pub image_size: Vector2<u32>,
    pub char_data: HashMap<char, GlyphMetrics>,
    pub font_size: f32,
    pub not_found_char: Option<char>,
    pub height: f32,
    pub name: String,
}

#[derive(Clone)]
pub struct FontMetricsInfo {
    pub char_data: HashMap<char, GlyphMetrics>,
    pub font_size: f32,
    pub height: f32,
}

pub fn make_font_infos<'a, T>(bytes: &[u8], font_sizes: &[f32], char_codes: T, not_found_char: Option<char>, name: String)
        -> Result<Vec<FontInfo>, String>
        where T: Iterator<Item = &'a char> {
    if not_found_char.is_some() {
        todo!("Not implemented not_found_char");
    }
    let char_codes: Vec<_> = char_codes.collect(); // allow multiple iterations of char_codes
    let font_settings = fontdue::FontSettings {
        collection_index: 0,
        scale: *font_sizes.iter().partial_max()
            .map_or_else(|| Err("Received NaN font size"), |x| Ok(x))?
    };
    let font = fontdue::Font::from_bytes(bytes, font_settings)?;
    let fonts: Vec<FontInfo> = font_sizes.iter().map(|font_size| {
        let glyphs: Vec<GlyphBitmap> = char_codes.iter().map(|char_code| {
            let (metrics, bitmap) = font.rasterize(**char_code, *font_size);
            GlyphBitmap {
                width: metrics.width,
                height: metrics.height,
                buffer: bitmap,
                metrics,
                char_code: **char_code,
            }
        }).collect();

        // make this padded thing pad each glyph by 1 px, and then send it to do_font_packing
        //     note that if we pad on the bottom and right edges for every box, then every box
        //     ends up with an exactly 1px boundary
        let padded: Vec<GlyphInfo<char>> = glyphs.iter().map(|glyph| {
            GlyphInfo {
                id: glyph.char_code,
                width: glyph.width as u32 + 1,
                height: glyph.height as u32 + 1
            }
        }).collect();

        // pack the glyphs
        let packing = match packing::do_font_packing(&padded) {
            Some(packing) => packing,
            None => panic!("Error loading font {} size {}: could not pack", "NO LONGER NAMING THESE", font_size)
        };

        // apparently it's in fractional pixels?
        let frac_pixels = 1.0;
        let font_size = *font_size;
 
        // create an image and isolate important metrics
        let font_info = FontInfo {
            image_buffer: apply_packing(&glyphs, &packing),
            image_size: Vector2::new(packing.width(), packing.height()),
            char_data: glyphs.iter().map(|glyph| (
                glyph.char_code,
                GlyphMetrics {
                    glyph_pos: {
                        let v = packing.get_glyph_pos(glyph.char_code).unwrap();
                        Vector2::new(v.x as f32, v.y as f32)
                    },
                    glyph_size: Vector2::new(glyph.width as f32,
                                             glyph.height as f32),
                    advance: glyph.metrics.advance_width * frac_pixels,
                    lsb: glyph.metrics.bounds.xmin as f32 * frac_pixels,
                    tsb: (glyph.metrics.bounds.ymin + glyph.metrics.bounds.height) as f32 * frac_pixels
                }
            )).collect(),
            font_size,
            not_found_char,
            height: font_size * frac_pixels,
            name: format!("{}-{}", name, font_size)
        };

        font_info
    }).collect();
    Ok(fonts)
}

// applies packing by copying glyphs to positions specified by the packing into a new vector
fn apply_packing(glyphs: &Vec<GlyphBitmap>, packing: &GlyphPacking<char>) -> Vec<u8> {
    let mut image: Vec<u8> = Vec::new();
    let width: usize = packing.width().try_into().unwrap();
    let height: usize = packing.height().try_into().unwrap();
    image.resize(width * height, 0);
    for glyph in glyphs {
        let uncv_l = packing.get_glyph_pos(glyph.char_code).unwrap();
        let location: Vector2<usize> = Vector2::new(
            uncv_l.x.try_into().unwrap(),
            uncv_l.y.try_into().unwrap());
        for y in 0..glyph.height as usize {
            for x in 0..glyph.width as usize {
                image[(location.y + y) * width + location.x + x] =
                    glyph.buffer[y * glyph.width as usize + x];
            }
        }
    }
    image
}

pub struct Font {
    sprite_texture: Texture,
    metrics: HashMap<char, GlyphMetrics>,
    font_size: f32,
    not_found_char: Option<char>,
    height: f32,
    image_size: cgmath::Vector2<f32>,
}

impl Font {
    pub fn make_from_info(device: &wgpu::Device, queue: &wgpu::Queue, font_info: &FontInfo, filter: wgpu::FilterMode) -> Result<Font, String> {
        // construct spritesheet
        let sprite_texture = Texture::from_bytes(
            device,
            queue,
            &font_info.image_buffer,
            (font_info.image_size.x, font_info.image_size.y),
            &font_info.name,
            wgpu::TextureFormat::R8Unorm,
            1,
            wgpu::TextureDimension::D2,
            filter
        ).map_err(|e| format!("Font {} error: {}", font_info.name, e.to_string()))?;

        // unfortunately just realized there's no point here if we're not going all the way
        // // construct glyph metric list and char id map
        // let (char_data, char_index): (Vec<_>, HashMap<_, _>) =
        //     font_info.char_data.iter()
        //         .zip(0..(font_info.char_data.len() as u16))
        //         .fold((vec![], HashMap::new()),
        //             |(mut char_data, mut char_index), ((&char, metrics), index)| {
        //         char_data.push(metrics.clone());
        //         char_index.insert(char, index);
        //         (char_data, char_index)
        //     });
        // // construct spritesheet location map buffer and put into a texture
        // let location_buffer: Vec<_> = char_data.iter()
        //     .flat_map(|metrics|
        //               [metrics.glyph_pos.x as u16, metrics.glyph_pos.y as u16].into_iter())
        //     .collect();
        // let location_texture = Texture::from_bytes(
        //     device,
        //     queue,
        //     bytemuck::cast_slice(location_buffer.as_slice()),
        //     (location_buffer.len() as u32, 1),
        //     font_info.name.as_str(),
        //     wgpu::TextureFormat::R16Uint,
        //     2,
        //     wgpu::TextureDimension::D1,
        //     wgpu::FilterMode::Nearest
        // ).map_err(|e| format!("Font {} location texture error: {}", font_info.name, e.to_string()))?;

        Ok(Font {
            sprite_texture,
            metrics: font_info.char_data.clone(),
            font_size: font_info.font_size,
            not_found_char: font_info.not_found_char,
            height: font_info.height,
            image_size: cgmath::Vector2::new(font_info.image_size.x as f32, font_info.image_size.y as f32),
        })
    }
}

pub trait BaseFontInfoContainer {
    fn line_height(&self) -> f32;
    fn font_size(&self) -> f32;
    fn get_metrics<'a>(&'a self, c: &char) -> Option<&'a GlyphMetrics>;
    fn get_metrics_info(&self) -> FontMetricsInfo;
}

impl BaseFontInfoContainer for FontInfo {
    fn line_height(&self) -> f32 {
        self.height // temp
    }

    fn font_size(&self) -> f32 {
        self.font_size
    }

    fn get_metrics<'a>(&'a self, c: &char) -> Option<&'a GlyphMetrics> {
        self.char_data.get(c)
    }

    fn get_metrics_info(&self) -> FontMetricsInfo {
        FontMetricsInfo {
            char_data: self.char_data.clone(),
            font_size: self.font_size,
            height: self.height
        }
    }
}

impl BaseFontInfoContainer for FontMetricsInfo {
    fn line_height(&self) -> f32 {
        self.height // temp
    }

    fn font_size(&self) -> f32 {
        self.font_size
    }

    fn get_metrics<'a>(&'a self, c: &char) -> Option<&'a GlyphMetrics> {
        self.char_data.get(c)
    }

    fn get_metrics_info(&self) -> FontMetricsInfo {
        self.clone()
    }
}

impl BaseFontInfoContainer for Font {
    fn line_height(&self) -> f32 {
        self.height
    }

    fn font_size(&self) -> f32 {
        self.font_size
    }

    fn get_metrics<'a>(&'a self, c: &char) -> Option<&'a GlyphMetrics> {
        self.metrics.get(c)
    }

    fn get_metrics_info(&self) -> FontMetricsInfo {
        FontMetricsInfo {
            char_data: self.metrics.clone(),
            font_size: self.font_size.clone(),
            height: self.height
        }
    }
}

pub trait FontInfoContainer {
    fn text_width(&self, text: &str) -> f32;
    // splits lines (word wrap) using maximum line length, new line characters, and white space
    fn split_lines(&self, text: &str, max_length: Option<f32>) -> Vec<String>;
}

impl<T> FontInfoContainer for T where T: BaseFontInfoContainer {
    fn text_width(&self, text: &str) -> f32{
        struct W {cur_adv: f32, longest: f32}
        text.chars().fold(W {cur_adv: 0.0, longest: 0.0}, |sum: W, c| match self.get_metrics(&c) {
            None => sum, // ignore non-characters
            Some(metrics) =>
                match c {
                    '\n' => W {cur_adv: 0.0, longest: sum.longest}, // new line
                    _ => W {
                        cur_adv: sum.cur_adv + metrics.advance,
                        longest: f32::max(sum.cur_adv + metrics.lsb + metrics.glyph_size.x, sum.longest)
                        // true size of line is the first argument of longest: last character's advance plus
                        // current character's lsb + width
                    }
                }
        }).longest
    }

    // splits lines (word wrap) using maximum line length, new line characters, and white space
    fn split_lines(&self, text: &str, max_length: Option<f32>) -> Vec<String> {
        let max_length = match max_length {
            Some(l) => l,
            None => f32::MAX,
        };
        struct W {cur_word: String, cur_word_adv: f32, cur_line: String, cur_line_adv: f32, lines: Vec<String>}
        let mut result = text.chars().fold(W {
            cur_word: String::new(),
            cur_word_adv: 0.0,
            cur_line: String::new(),
            cur_line_adv: 0.0,
            lines: Vec::new()
        }, |mut cur: W, c| match self.get_metrics(&c) {
            None => cur, // ignore unknown character
            Some(metrics) => match c {
                '\n' => W { // force a new line
                    cur_line: String::new(),
                    cur_line_adv: 0.0,
                    cur_word: String::new(),
                    cur_word_adv: 0.0,
                    lines: {
                        cur.cur_line += cur.cur_word.as_str();
                        cur.lines.push(cur.cur_line);
                        cur.lines
                    }
                },
                ' ' | '\t' => {
                    let next_length = cur.cur_line_adv + cur.cur_word_adv + metrics.lsb + metrics.glyph_size.x;
                    if next_length > max_length {
                        W {
                            cur_word: String::new(),
                            cur_word_adv: 0.0,
                            cur_line: String::from(c),
                            cur_line_adv: metrics.advance,
                            lines: {
                                cur.lines.push(cur.cur_line);
                                cur.lines
                            }
                        }
                    } else {
                        W {
                            cur_word: String::new(),
                            cur_word_adv: 0.0,
                            cur_line: {
                                cur.cur_line += cur.cur_word.as_str();
                                cur.cur_line.push(c);
                                cur.cur_line
                            },
                            cur_line_adv: cur.cur_line_adv + cur.cur_word_adv + metrics.advance,
                            lines: cur.lines
                        }
                    }
                },
                _ => {
                    let next_length = cur.cur_line_adv + cur.cur_word_adv + metrics.lsb + metrics.glyph_size.x;
                    if next_length > max_length { // determine if a new line is needed
                        if cur.cur_line.len() == 0 { // if the current line is empty i.e. it's all one word
                            W { // split the current word at the current position, the end of the line
                                cur_word: String::from(c),
                                cur_word_adv: metrics.advance,
                                cur_line: cur.cur_line, // was empty anyway
                                cur_line_adv: 0.0,
                                lines: {
                                    cur.lines.push(cur.cur_word);
                                    cur.lines
                                }
                            }
                        } else {
                            W {
                                cur_word: {
                                    cur.cur_word.push(c);
                                    cur.cur_word
                                },
                                cur_word_adv: cur.cur_word_adv + metrics.advance,
                                cur_line: String::new(),
                                cur_line_adv: 0.0,
                                lines: {
                                    cur.lines.push(cur.cur_line);
                                    cur.lines
                                }
                            }
                        }
                    } else { // the totally regular non-whitespace no new line case
                        W {
                            cur_word: {
                                cur.cur_word.push(c);
                                cur.cur_word
                            },
                            cur_word_adv: cur.cur_word_adv + metrics.advance,
                            cur_line: cur.cur_line,
                            cur_line_adv: cur.cur_line_adv,
                            lines: cur.lines
                        }
                    }
                }
            }
        });
        result.cur_line += result.cur_word.as_str();
        result.lines.push(result.cur_line);
        result.lines
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.0] },
    Vertex { position: [0.0, 1.0] },
    Vertex { position: [1.0, 1.0] },
    Vertex { position: [0.0, 0.0] },
    Vertex { position: [1.0, 1.0] },
    Vertex { position: [1.0, 0.0] },
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
                    format: wgpu::VertexFormat::Float32x2,
                },
            ]
        }
    }
}

// font renderer stuff starts here
#[derive(Clone)]
pub struct Instance {
    pub matrix: cgmath::Matrix4<f32>,
    pub texture_pos: cgmath::Vector2<f32>,
    pub texture_scale: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    pub model: [[f32; 4]; 4],
    pub texture_pos: [f32; 2],
    pub texture_scale: [f32; 2],
    pub color: [f32; 4],
}

impl InstanceRaw {
    const ZERO: Self = Self {
        model: [[0.0; 4]; 4],
        texture_pos: [0.0; 2],
        texture_scale: [0.0; 2],
        color: [0.0; 4],
    };
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: self.matrix.into(),
            texture_pos: self.texture_pos.into(),
            texture_scale: self.texture_scale.into(),
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
                // for texture_pos
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // for texture_scale
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 18]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // for color
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 11,
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

pub struct FontRenderer {
    render_pipeline: wgpu::RenderPipeline,

    square_vertex_buffer: wgpu::Buffer,
    square_num_vertices: u32,

    texture_bind_groups: TextureBindGroups,

    instance_buffer: wgpu::Buffer,
    current_buffer_pos: Cell<u32>,
}

impl FontRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) -> Result<FontRenderer, String> {

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
                label: Some("font_texture_bind_group_layout"),
            });

        // create default square vertex buffer
        let square_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Font Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX
            }
        );
        let square_num_vertices = VERTICES.len() as u32;

        // create an instance buffer list
        let instance_data = vec![InstanceRaw::ZERO; 8192];
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("font_instance_buffer"),
                contents: bytemuck::cast_slice(instance_data.as_slice()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        // create the render pipeline
        let render_pipeline = {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("font_texture_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("font_shader.wgsl").into()),
            });

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Font Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
            let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Font Render Pipeline"),
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
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
        
        Ok(Self {
            render_pipeline,
            current_buffer_pos: Cell::new(0),
            instance_buffer,
            square_num_vertices,
            square_vertex_buffer,
            texture_bind_groups: TextureBindGroups::new(texture_bind_group_layout),
        })
    }

    pub fn register_font(&mut self, device: &wgpu::Device, font: &Font) {
        // construct bind groups for font's textures
        self.texture_bind_groups.make_texture_bind_group(device, &font.sprite_texture);
    }

    pub fn reset(&self) {
        self.current_buffer_pos.set(0);
    }

    pub fn render<'a, C: CameraObj>(&'a self, font: &Font, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'a>, camera: &C, instances: &Vec<(String, cgmath::Vector2<f32>, cgmath::Vector4<f32>)>) -> Result<(), wgpu::SurfaceError> {
            
        // retrieve bind group for the given texture
        let diffuse_bind_group = self.texture_bind_groups.get_texture_bind_group(&font.sprite_texture)
        .expect("Could not find texture bind group, did you forget to register your font?");
        let proj = camera.proj_view();

        // split instances apart and reformat them
        let instances_calc = instances.iter().flat_map(|(text, pos, color)| {
            let base = proj * Matrix4::from_translation(cgmath::Vector3::new(pos.x, pos.y, 0.0));
            let mut line_width = 0.0;
            let line_height = font.line_height();
            let mut trans: Matrix4<f32> = Matrix4::identity();
            text.chars().flat_map(move |c| {
                // get metrics for character
                if c == '\n' {
                    trans = trans * cgmath::Matrix4::from_translation(
                        cgmath::Vector3::new(-line_width, line_height, 0.0)
                    );
                    line_width = 0.0;
                    return None;
                }
                let metrics = font.metrics.get(&c)?;
                trans = trans * Matrix4::from_translation(
                    cgmath::Vector3::new(metrics.lsb, -metrics.tsb, 0.0)
                );
                let matrix = base * trans * Matrix4::from_nonuniform_scale(
                    metrics.glyph_size.x, metrics.glyph_size.y, 1.0
                );
                trans = trans * Matrix4::from_translation(
                    cgmath::Vector3::new(-metrics.lsb + metrics.advance, metrics.tsb, 0.0)
                );
                line_width += metrics.advance;
                Some(Instance {
                    // matrix: camera.proj() *
                    //     Matrix4::from_translation(cgmath::Vector3 { x: 100.0, y: 100.0, z: 0.0 }) *
                    //     Matrix4::from_scale(100.0),
                    matrix,
                    color: *color,
                    texture_pos: cgmath::Vector2::new(
                        metrics.glyph_pos.x / font.image_size.x,
                        metrics.glyph_pos.y / font.image_size.y),
                    texture_scale: cgmath::Vector2::new(
                        metrics.glyph_size.x / font.image_size.x,
                        metrics.glyph_size.y / font.image_size.y)
                })
            })
            .map(|instance| instance.to_raw())
        }).collect::<Vec<_>>();
        let current_buffer_pos = self.current_buffer_pos.get();
        if MAX_GLYPHS_PER_FRAME < instances_calc.len() as u32 + current_buffer_pos {
            panic!("Gave too many font character instances! {} instances. Can only support {} instances maximum per frame",
            instances_calc.len(), MAX_GLYPHS_PER_FRAME);
        }
        queue.write_buffer(
            &self.instance_buffer,
            (current_buffer_pos as usize * std::mem::size_of::<InstanceRaw>()) as u64,
            bytemuck::cast_slice(&instances_calc));
        self.current_buffer_pos.set(current_buffer_pos + instances_calc.len() as u32);
        // let instances_calc = vec![
        //     Instance {
        //         matrix: cgmath::Matrix4::identity(),
        //         color: cgmath::Vector4::new(1.0, 1.0, 1.0, 1.0),
        //         texture_pos: cgmath::Vector2::new(0.0, 0.0),
        //         texture_scale: cgmath::Vector2::new(1.0, 1.0),
        //     }.to_raw(),
        // ];
        // queue.write_buffer(
        //     &self.instance_buffer,
        //     0, bytemuck::cast_slice(&instances_calc[0..1]));

        // bind everything
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.square_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.draw(0..self.square_num_vertices,
                         current_buffer_pos..(current_buffer_pos + instances_calc.len() as u32));
        // render_pass.draw(0..6, 0..1);

        Ok(())
    }
}

