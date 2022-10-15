// kevin
// mostly copying from existing impl at 
// https://github.com/isaacph/GameJamNov2020/blob/master/src/main/java/Box.java

use cgmath::{Vector2, Vector4};

#[derive(Clone)]
pub struct BoundingBox {
    pub center: Vector2<f32>,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    const INTERSECT_STEP_SIZE: f32 = 0.01;

    const RESOLVE_OFFSET: f32 = 0.00005;

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

    pub fn get_x_max(&self) -> f32 {
        self.center.x + self.width / 2.0
    }

    pub fn get_x_min(&self) -> f32 {
        self.center.x - self.width / 2.0
    }

    pub fn get_y_max(&self) -> f32 {
        self.center.y + self.height / 2.0
    }

    pub fn get_y_min(&self) -> f32 {
        self.center.y - self.height / 2.0
    }

    pub fn add(&mut self, offset : Vector2<f32>) {
        self.center += offset;
    }

    pub fn get_scale(&self) -> Vector2<f32> {
        return Vector2::new(self.width, self.height);
    }

    pub fn points(&self) -> Vec<Vector2<f32> > {
        let mut point_set: Vec<Vector2<f32> > = Vec::new();
        
        let mut x_current = self.center.x - self.width / 2.0;
        while x_current < self.center.x + self.width / 2.0 {
            let mut y_current = self.center.y - self.height / 2.0;
            while y_current < self.center.y + self.height / 2.0 {
                point_set.push(Vector2::new(x_current, y_current));
                y_current = f32::min(y_current + BoundingBox::INTERSECT_STEP_SIZE, self.center.y + self.height / 2.0);
            }
            x_current = f32::min(x_current + BoundingBox::INTERSECT_STEP_SIZE, self.center.x + self.width / 2.0);
        }

        return point_set;
    }
 
    pub fn calculate_resolve_options(&self, b : &BoundingBox) -> Vec<Option<Vector2<f32> > > {
        let get_x_resolves = |a : &BoundingBox, b : &BoundingBox| -> Vector2<Option<Vector2<f32> > > {
            let mut left_resolve: Option<Vector2<f32> > = None;
            let mut right_resolve: Option<Vector2<f32> > = None; 
            // the resolution goes to the right: intersection is on the left, and the added position is rightward facing to account for that
            if a.center.x - a.width / 2.0 < b.center.x + b.width / 2.0 
                && b.center.x - b.width / 2.0 < a.center.x - a.width / 2.0 {
                left_resolve = Some(Vector2::new(
                    a.center.x + (b.center.x + b.width / 2.0 - (a.center.x - a.width / 2.0)), 
                    a.center.y)
                );
            }
            else if b.center.x - b.width / 2.0 < a.center.x + a.width / 2.0 
                && a.center.x - a.width / 2.0 < b.center.x - b.width / 2.0 {
                right_resolve = Some(Vector2::new(
                    a.center.x + (a.center.x + a.width / 2.0 - (b.center.x - b.width / 2.0)), 
                    a.center.y)
                );
            }

            Vector2::new(left_resolve, right_resolve)
        };

        let get_y_resolves = |a : &BoundingBox, b : &BoundingBox| -> Vector2<Option<Vector2<f32> > > {
            let mut down_resolve: Option<Vector2<f32> > = None;
            let mut up_resolve: Option<Vector2<f32> > = None;  
            if a.center.y - a.height / 2.0 < b.center.y + b.height / 2.0 
                && b.center.y - b.height / 2.0 < a.center.y - a.height / 2.0 {
                down_resolve = Some(Vector2::new(
                    a.center.y,
                    a.center.y + (b.center.y + b.height / 2.0 - (a.center.y - a.height / 2.0)), 
                ));
            }
            else if b.center.y - b.height / 2.0 < a.center.y + a.height / 2.0 
                && a.center.y - a.height / 2.0 < b.center.y - b.height / 2.0 {
                up_resolve = Some(Vector2::new(
                    a.center.y + (a.center.y + a.height / 2.0 - (b.center.y - b.height / 2.0)), 
                    a.center.y)
                );
            }

            Vector2::new(down_resolve, up_resolve)
        };

        let x_resolves = get_x_resolves(self, b);
        let y_resolves = get_y_resolves(self, b);

        return vec![x_resolves[0], x_resolves[1], y_resolves[0], y_resolves[1]];
    }

    // returns empty vector if no intersection detected.
    pub fn resolve_options(&self, mover: &BoundingBox) -> Vec<Vector2<f32> > {
        let mut options = Vec::new();
        
        if self.does_intersect(mover) {
            let intersection = BoundingBox::calculate_resolve_options(self, mover);
            options = intersection.into_iter().flatten().collect();
            // for direction in intersection {
            //     if direction != None {

            //     }
            // }


            // options = vec![
            //     Vector2::new(intersection[1] - intersection[0] + BoundingBox::RESOLVE_OFFSET, 0.0),
            //     Vector2::new(-(intersection[1] - intersection[0] + BoundingBox::RESOLVE_OFFSET), 0.0),
            //     Vector2::new(0.0, intersection[3] - intersection[2] + BoundingBox::RESOLVE_OFFSET),
            //     Vector2::new(0.0, -(intersection[3] - intersection[2] + BoundingBox::RESOLVE_OFFSET)),
            // ]

        }
        return options;
    }

    pub fn does_intersect(&self, other: &BoundingBox) -> bool {
        // BoundingBox::get_intersection(self, other) != BoundingBox::NO_INTERSECTION
        let does_intersect_in_x = |a : &BoundingBox, b : &BoundingBox| -> bool {
            (a.center.x - a.width / 2.0 < b.center.x + b.width / 2.0 && b.center.x - b.width / 2.0 < a.center.x - a.width / 2.0 )
            || (b.center.x - b.width / 2.0 < a.center.x + a.width / 2.0 && a.center.x - a.width / 2.0 < b.center.x - b.width / 2.0)
            || (a.center.x - a.width / 2.0 < b.center.x - b.width / 2.0 && b.center.x + b.width / 2.0 < a.center.x + a.width / 2.0 )
            || (b.center.x - b.width / 2.0 < a.center.x - a.width / 2.0 && a.center.x + a.width / 2.0 < b.center.x + b.width / 2.0)
        };

        let does_intersect_in_y = |a : &BoundingBox, b : &BoundingBox| -> bool {
            (a.center.y - a.height / 2.0 < b.center.y + b.height / 2.0 && b.center.y - b.height / 2.0 < a.center.y - a.height / 2.0)
            || (b.center.y - b.height / 2.0 < a.center.y + a.height / 2.0 && a.center.y - a.height / 2.0 < b.center.y - b.height / 2.0)
            || (a.center.y - a.height / 2.0 < b.center.y - b.height / 2.0 && b.center.y + b.height / 2.0 < a.center.y + a.height / 2.0)
            || (b.center.y - b.height / 2.0 < a.center.y - a.height / 2.0 && a.center.y + a.height / 2.0 < b.center.y + b.height / 2.0)
        };

        return does_intersect_in_x(self, other) && does_intersect_in_y(self, other);
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

