use bevy::prelude::*;

#[derive(Clone, Component, Reflect)]
#[reflect(Component)]
pub enum Interactables {
    Wall,
    Goal(Teams),
    Ball,
    Person,
}

#[derive(Clone, Component, PartialEq, Reflect)]
#[reflect(Component)]
pub enum Teams {
    Enemy,
    Player,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Ball;

#[derive(Clone, Component, Reflect)]
#[reflect(Component)]
pub enum CharacterClasses {
    Goalkeeper,
    CentralDefender,
    Midfielder,
    Attacker,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Velocity(pub Vec2);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Stats {
    pub ap: usize,
    pub intial_ap: usize,
    pub kick_strength: f32,
    pub passing_skill: f32,
    pub wit: f32,
    pub defense: f32,
    pub initiative: u8,
}

impl Stats {
    pub fn with_initiative(mut self, initiative: usize) -> Self {
        self.initiative = initiative as u8;
        self
    }

    pub fn reset_ap(&mut self) {
        self.ap = self.intial_ap;
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ActionQueue(pub Vec<PlayerAction>);

impl Default for ActionQueue {
    fn default() -> Self {
        ActionQueue(Vec::new())
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub enum PlayerAction {
    MoveTo(Vec3),
    Kick(Entity),
    TakeControl(Entity),
    Foul(Entity),
    /// Which entity has to be passed where
    Pass(Entity, Vec3),
    DefendGoal,
    SkipTurn,
    EndTurn(Teams),
    Advance,
    PassDown,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PointerObject(pub Timer);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CurrentPlayer;
