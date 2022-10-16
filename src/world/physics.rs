use std::collections::{HashMap, HashSet};
use cgmath::{Vector2, InnerSpace, Zero};
use uuid::Uuid;
use crate::bounding_box::BoundingBox;

use super::{IDObject, projectile::ProjectileType};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PhysObjType {
    Player,
    Wall,
    Projectile(ProjectileType),
    Enemy,
}

impl PhysObjType {
    pub fn all() -> HashSet<PhysObjType> {
        use PhysObjType::*;
        let mut all = vec![Player, Wall, Enemy];
        all.extend(ProjectileType::all().into_iter().map(|p| Projectile(p)));
        all.into_iter().collect()
    }
}

pub trait Physics: IDObject {
    fn get_physics(&self) -> Vec<(PhysicsID, PhysicsObject)>;
    fn pre_physics(&mut self);
    fn resolve(&mut self, id: PhysicsID, delta: Vector2<f32>, resolve: Vector2<f32>, typ: Vec<(PhysObjType, Uuid)>) -> Vector2<f32>;
    fn typ(&self) -> PhysObjType;
}

impl IDObject for (Uuid, PhysicsObject) {
    fn get_uuid(&self) -> Uuid {
        self.0
    }
}

impl Physics for (Uuid, PhysicsObject) {
    fn get_physics(&self) -> Vec<(PhysicsID, PhysicsObject)> {
        vec![self.clone()]
    }

    fn pre_physics(&mut self) {
    }

    fn resolve(&mut self, id: PhysicsID, delta: Vector2<f32>, resolve: Vector2<f32>, typ: Vec<(PhysObjType, Uuid)>) -> Vector2<f32> {
        Vector2::zero()
    }

    fn typ(&self) -> PhysObjType {
        self.1.typ
    }
}

#[derive(Clone, Debug)]
pub struct PhysicsObject {
    pub bounding_box: BoundingBox,
    pub velocity: Vector2<f32>,
    pub can_move: bool,
    pub typ: PhysObjType,
    pub collides_with: HashSet<PhysObjType>,
    pub move_by: HashSet<PhysObjType>,
}

type PhysicsID = Uuid;

pub fn simulate<F: FnMut(PhysicsID, Vector2<f32>, Vector2<f32>, &mut PhysicsObject, Vec<(PhysObjType, Uuid)>)>
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
        let obj_typ = obj.typ;
        let obj_collides_with = obj.collides_with.clone();
        let obj_move_by = obj.move_by.clone();
        if obj.can_move {
            let delta = obj.velocity * delta_time;
            // finds the number of overlaps of one bounding box against the self

            let find_overlaps = |objects: &HashMap<Uuid, PhysicsObject>, box_a: &BoundingBox, box_a_id: Uuid| {
                objects.iter().fold((0, vec![]), |(count, mut types), (other_id, other)| {
                    if *other_id != box_a_id {
                        let box_b = other.bounding_box.clone();
                        if box_a.does_intersect(&box_b) {
                            if other.collides_with.contains(&obj_typ) &&
                                obj_collides_with.contains(&other.typ) {
                                types.push(other_id.clone());
                            }
                            if obj_move_by.contains(&other.typ) {
                                return (count + 1, types)
                            }
                        }
                    }
                    (count, types)
                })
            };

            for delta in [
                Vector2::new(delta.x, 0.0),
                Vector2::new(0.0, delta.y),
            ] {
                let mut overlappers = vec![];
                let obj = objects.get(&id).unwrap();
                let mut box_a = obj.bounding_box.clone();

                // move in delta direction
                box_a.add(delta);

                // get starting overlaps
                let (starting_overlaps, ov) = find_overlaps(&objects, &box_a, id);
                overlappers.extend(ov.into_iter());

                // find best way to resolve collisions
                let mut best_resolve: Vector2<f32> = Vector2::new(0.0, 0.0);
                let mut best_resolve_len_sq = delta.magnitude2();
                let mut best_resolve_overlaps = starting_overlaps;
                for (other_id, other) in &objects {
                    if *other_id != id &&
                            obj_move_by.contains(&other.typ) {
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
                            let (new_overlaps, _) = find_overlaps(&objects, &box_a_resolved, id);
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

                // if let PhysObjType::Projectile(_) = obj_typ {
                //     println!("Phys proj1");
                // }

                let overlap_types = overlappers.iter()
                    .flat_map(|id| objects.get(id).map(|obj| (id, obj)))
                    .map(|(id, obj)| (obj.typ, *id)).collect();

                let obj = objects.get_mut(&id).unwrap();
                resolve(id, delta, best_resolve, obj, overlap_types);
                overlappers.iter().for_each(|id| {
                    objects.get_mut(id).map(|obj| {
                        resolve(*id, Vector2::zero(), Vector2::zero(), obj, vec![(obj_typ, *id)]);
                    });
                });
            }
        }
    }
}
