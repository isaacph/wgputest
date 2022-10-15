// kevin
// mostly copying from existing impl at 
// https://github.com/isaacph/GameJamNov2020/blob/master/src/main/java/Box.java

use cgmath::{Vector2};

pub struct BoundingBox {
    pub center: Vector2<f32>,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    const INTERSECT_STEP_SIZE: f32 = 0.01;

    const RESOLVE_OFFSET: f32 = 0.005;

    const NO_INTERSECTION: Vec<f32> =  vec![
        f32::MIN,
        f32::MAX,
        f32::MIN,
        f32::MAX,
    ];

    pub fn new(center: Vector2<f32>, width: f32, height: f32) -> Self {
        Self {
            center,
            width,
            height,
        }
    }

    pub fn new_default() -> Self {
        Self {
            center: Vector2::new(0.0, 0.0),
            width: 1.0,
            height: 1.0,
        }
    }

    pub fn add(&mut self, offset : Vector2<f32>) {
        self.center += offset;
    }

    pub fn get_scale(&self) -> Vector2<f32> {
        return Vector2::new(self.width, self.height);
    }

    pub fn points(&self) -> Vec<Vector2<f32> > {
        let mut point_set: Vec<Vector2<f32> > = Vec::new();
        
        let x_current = self.center.x - self.width / 2.0;
        while x_current != self.center.x + self.width / 2.0 {
            let y_current = self.center.y - self.height / 2.0;
            while y_current != self.center.y + self.height / 2.0 {
                point_set.push(Vector2::new(x_current, y_current));
                y_current = f32::min(y_current + BoundingBox::INTERSECT_STEP_SIZE, self.height);
            }
            x_current = f32::min(x_current + BoundingBox::INTERSECT_STEP_SIZE, self.width);
        }

        return point_set;
    }
 
    pub fn get_intersection(a : &BoundingBox, b : &BoundingBox) -> Vec<f32> {
        // this is the GJK algorithm. Minkowski difference is the set of pairwise differences
        // between points in a and points in b.
        // a and b intersect iff origin in Minkowski difference, which we check on line 60

        // return the corners of the intersection rectangle
        let x_min = f32::MIN;
        let x_max = f32::MAX; 
        let y_min = f32::MIN; 
        let y_max = f32::MAX;
        for a_point in a.points() {
            for b_point in b.points() {
                if a_point == b_point {
                    x_min = f32::min(x_min, b_point.x);
                    x_max = f32::max(x_max, b_point.x);
                    y_min = f32::min(y_min, b_point.y);
                    y_max = f32::max(y_max, b_point.y);
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

    // returns empty vector if no intersection detected.
    pub fn resolve_options(&self, mover: &BoundingBox) -> Vec<Vector2<f32> > {
        let options = Vec::new();
        let intersection = BoundingBox::get_intersection(self, mover);
        
        if  intersection != BoundingBox::NO_INTERSECTION {
            options = vec![
                Vector2::new(intersection[1] - intersection[0] + BoundingBox::RESOLVE_OFFSET, 0.0),
                Vector2::new(-(intersection[1] - intersection[0] + BoundingBox::RESOLVE_OFFSET), 0.0),
                Vector2::new(0.0, intersection[3] - intersection[2] + BoundingBox::RESOLVE_OFFSET),
                Vector2::new(0.0, -(intersection[3] - intersection[2] + BoundingBox::RESOLVE_OFFSET)),
            ]
        }

        return options;
    }

    // really shitty implementation
    // TODO: can be way faster
    pub fn does_intersect(&self, other: &BoundingBox) -> bool {
        BoundingBox::get_intersection(self, other) != BoundingBox::NO_INTERSECTION
    }

    // dumb question but do we need these?
    // pub fn resolve(pusher: &BoundingBox, mover: &BoundingBox) -> Vector2<f32> {
    //     // TODO: make this work using GLK
    // }

    // pub fn resolve_x(pusher: &BoundingBox, mover: &BoundingBox) -> Vector2<f32> {
    //     // TODO: make this work using GLK
    // }

    // pub fn resolve_y(pusher: &BoundingBox, mover: &BoundingBox) -> Vector2<f32> {
    //     // TODO: make this work using GLK
    // }
}

