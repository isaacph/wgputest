// kevin
// mostly copying from existing impl at 
// https://github.com/isaacph/GameJamNov2020/blob/master/src/main/java/Box.java

use cgmath::{Vector2};

pub struct Box {
    pub center: cgmath::Vector2<f32>,
    pub width: f32,
    pub height: f32,
}

impl Box {
    const INTERSECT_STEP_SIZE: f32 = 0.01;

    const RESOLVE_OFFSET: f32 = 0.005;

    const NO_INTERSECTION: Vec<cgmath::Vector2<f32> > =  vec![
        cgmath::Vector2::new(f32::MIN, f32::MIN),
        cgmath::Vector2::new(f32::MAX, f32::MIN),
        cgmath::Vector2::new(f32::MIN, f32::MAX),
        cgmath::Vector2::new(f32::MAX, f32::MAX)
    ];


    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn new_default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        }
    }

    pub fn add(&self, offset : cgmath::Vector2<f32>) {
        center.add(offset);
    }

    pub fn get_scale() -> cgmath::Vector2<f32> {
        return cgmath::Vector2<f32>(width, height);
    }

    pub fn points() -> Vec<cgmath::Vector2<f32> > {
        let mut point_set = Vec::new();
        
        x_current = center.x - width / 2;
        while (x_current != center.x + width / 2) {
            y_current = center.y - height / 2;
            while (y_current != center.y + height / 2) {
                point_set.push(cgmath::Vector2<f32>(x_current, y_current));
                y_current = min(y_current + INTERSECT_STEP_SIZE, height);
            }
            x_current = min(x_current + INTERSECT_STEP_SIZE, width);
        }

        return point_set;
    }
 
    pub fn intersect(&a : Box, &b : Box) -> Vec<f32> {
        // this is the GJK algorithm. Minkowski difference is the set of pairwise differences
        // between points in a and points in b.
        // a and b intersect iff origin in Minkowski difference, which we check on line 60

        // return the corners of the intersection rectangle
        x_min = f32::MIN;
        x_max = f32::MAX; 
        y_min = f32::MIN; 
        y_max = f32::MAX;
        for a_point in a.points() {
            for b_point in b.points() {
                if a_point == b_point {
                    x_min = min(x_min, b_point.x);
                    x_max = max(x_max, b_point.x);
                    y_min = min(y_min, b_point.y);
                    y_max = max(y_max, b_point.y);
                }
            }
        }
        return vec![
            x_min,
            x_max,
            y_min,
            y_max,
        ];
    }


    pub fn resolve_options(&pusher: Box, &mover: Box) -> Vec<cgmath::Vector2<f32> > {
        options = Vec::new();
        intersection = intersect(pusher, mover);
        
        if intersection != NO_INTERSECTION {
            options = vec![
                cgmath::Vector2::new(x_max - x_min + RESOLVE_OFFSET, 0),
                cgmath::Vector2::new(-(x_max - x_min + RESOLVE_OFFSET), 0),
                cgmath::Vector2::new(y_max - y_min + RESOLVE_OFFSET, 0),
                cgmath::Vector2::new(-(y_max - y_min + RESOLVE_OFFSET), 0),
            ]
        }
    }

    pub fn resolve(&pusher: Box, &mover: Box) -> Vector2<f32> {
        // TODO: make this work using GLK
    }

    pub fn resolve_x(&pusher: Box, &mover: Box) -> Vector2<f32> {
        // TODO: make this work using GLK
    }

    pub fn resolve_y(&pusher: Box, &mover: Box) -> Vector2<f32> {
        // TODO: make this work using GLK
    }
}

