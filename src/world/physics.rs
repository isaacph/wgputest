use std::collections::HashMap;
use cgmath::{Vector2, InnerSpace};
use uuid::Uuid;
use crate::bounding_box::BoundingBox;

use super::IDObject;

pub trait Physics: IDObject {
    fn get_physics(&self) -> Option<(Uuid, PhysicsObject)>;
    fn pre_physics(&mut self);
    fn resolve(&mut self, delta: Vector2<f32>, resolve: Vector2<f32>) -> Vector2<f32>;
}

#[derive(Clone)]
pub struct PhysicsObject {
    pub bounding_box: BoundingBox,
    pub velocity: Vector2<f32>,
    pub can_move: bool,
}

type PhysicsID = Uuid;

pub fn simulate<F: FnMut(PhysicsID, Vector2<f32>, Vector2<f32>, &mut PhysicsObject)>
    (delta_time: f32, mut objects: HashMap<PhysicsID, PhysicsObject>, mut resolve: F) {
    //  for each moveable object
    //      move object in x direction
    //          check collisions in x
    //          respond to collisions in x
    //      move object in y direction
    //          check collisions in y
    //          respond to collisions in y

    let obj_ids: Vec<Uuid> = objects.keys().cloned().collect();
    for id in obj_ids {
        let obj = objects.get(&id).unwrap();
        if obj.can_move {
            let delta = obj.velocity * delta_time;
            // finds the number of overlaps of one bounding box against the self

            let find_overlaps = |objects: &HashMap<Uuid, PhysicsObject>, box_a: &BoundingBox, box_a_id: Uuid| {
                objects.iter().fold(0, |count, (other_id, other)| {
                    if *other_id != box_a_id {
                        let box_b = other.bounding_box.clone();
                        if box_a.does_intersect(&box_b) {
                            return count + 1
                        }
                    }
                    count
                })
            };

            for delta in [
                Vector2::new(delta.x, 0.0),
                Vector2::new(0.0, delta.y),
            ] {
                let obj = objects.get(&id).unwrap();
                let mut box_a = obj.bounding_box.clone();

                // move in delta direction
                box_a.add(delta);

                // get starting overlaps
                let starting_overlaps = find_overlaps(&objects, &box_a, id);

                // find best way to resolve collisions
                let mut best_resolve: Vector2<f32> = Vector2::new(0.0, 0.0);
                let mut best_resolve_len_sq = delta.magnitude2();
                let mut best_resolve_overlaps = starting_overlaps;
                for (other_id, other) in &objects {
                    if *other_id != id {
                        // got a non-me other
                        let box_b = other.bounding_box.clone();
                        let resolve_options = box_a.resolve_options(&box_b);
                        
                        for resolve_option in resolve_options {
                            // determine how good the resolve option is
                            let box_a_resolved = {
                                let mut box_a = box_a.clone();
                                box_a.add(resolve_option);
                                box_a
                            };
                            let new_overlaps = find_overlaps(&objects, &box_a_resolved, id);
                            let new_len_sq =
                                resolve_option.x * resolve_option.x +
                                resolve_option.y * resolve_option.y;
                            
                            // if it's better than best then use it
                            if new_overlaps < best_resolve_overlaps ||
                                    (new_overlaps == best_resolve_overlaps &&
                                    new_len_sq < best_resolve_len_sq) {
                                best_resolve = resolve_option;
                                best_resolve_len_sq = new_len_sq;
                                best_resolve_overlaps = new_overlaps;
                            }
                        }
                    }
                }

                let obj = objects.get_mut(&id).unwrap();
                resolve(id, delta, best_resolve, obj);
            }
        }
    }
}
