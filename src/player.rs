use cgmath::{Vector2, Vector4};

// temporary location for player -- should be combined with Player class in world I think
pub struct GameObjectData {
    pub bounding_box: BoundingBox,
    pub velocity: Vector2<f32>,
    
    // carryover info from preexisting GameObject class 
    pub scale: Vector2<f32>,
    pub color: Vector4<f32>, // in the future don't store any rendering info inside the world
}

struct Player_temp {
    id: Uuid,
    data: GameObjectData,
    // make direction the vector you add to bounding_box.center to get gun position
    // in game, this should respond to wasd input
    direction: Vector2<f32>

}

impl Player_temp {
    // 
    pub fn jump(&mut self) {
        const JUMP_VELOCITY: Vector2<f32> = Vector2::new(0.0, 2.0);
        self.data.velocity += JUMP_VELOCITY;
    }

    pub fn dash(&mut self) {
        const DASH_VELOCITY: Vector2<f32> = Vector2::new(2.0, 0.0);
        self.data.velocity += JUMP_VELOCITY;
    }

    // creates and returns a projectile to add to the game world
    pub fn shoot(&self) -> Projectile {
        // Wand is in front of the player
        const WAND_LOCATION: Vector2<f32> = self.data.bounding_box.center + self.direction;
        return Projectile::new(
            WAND_LOCATION,
            Vector2::new(0.25, 0.25),
            Vector4::new(0.0, 0.0, 0.0, 0.0),
            self.direction
        )
    }
}
