use cgmath::Vector4;

pub struct GameObject {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>, // in the future don't store any rendering info inside the world
}

pub struct World {
    pub objects: Vec<GameObject>,
}

impl World {
    pub fn new() -> Self {
        use cgmath::Vector2;
        let objects = (0..(10)).into_iter().map(|i| {
            GameObject {
                position: Vector2::new((i as f32) * 0.8, (i as f32) * 0.8),
                scale: Vector2::new(1.5, 0.9),
                color: Vector4::new(
                    if i % 2 == 0 { 0.0 } else { 1.0 },
                    if i % 2 == 1 { 0.0 } else { 1.0 },
                    0.0,
                    1.0
                ),
            }
        }).collect::<Vec<_>>();
        Self {
            objects
        }
    }

    pub fn update(&mut self, delta_time: f32) {
    }
}
