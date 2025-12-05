use bevy::prelude::*;

use crate::{assets::MHItemFinal};

/// Processed meshes ready for spawning
// TODO: pass handles, because accessing ResMut<Assets<Mesh>>
// to read and write in same system had issue
#[derive(EntityEvent)]
pub struct CharacterGenerate {
    pub entity: Entity,
    pub parts: Vec<MHItemFinal>,
        
    /// Character height (for collider sizing)
    pub height: f32,
    /// Min Y of mesh (for ground offset)
    pub min_y: f32,
    /// Floor offset (for shoes/bare feet)
    pub floor_offset: f32,
}

/// Event to trigger animation setup after character spawn
#[derive(EntityEvent)]
pub struct CharacterComplete {
    pub entity: Entity,
}
