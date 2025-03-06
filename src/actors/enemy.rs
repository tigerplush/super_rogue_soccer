use bevy::prelude::*;

use crate::{states::{gameplay::Log, GameplayStates}, ui::LogEvent};

use super::CurrentPlayer;

pub fn enemy_ai(mut events: EventWriter<LogEvent>, mut next: ResMut<NextState<GameplayStates>>, query: Single<&Name, With<CurrentPlayer>>) {
    info!("thinking...");
    events.send(LogEvent(format!("{} is passing their turn", query.into_inner())));
    next.set(GameplayStates::PlayerTurn);
}
