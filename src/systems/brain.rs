use bevy::prelude::*;
use big_brain::*;

pub struct BrainPlugin;

impl Plugin for BrainPlugin {
    fn build(&self, app: &mut App) {
        use CoreStage::*;
        app.add_system_set_to_stage(
            First,
            SystemSet::new()
                .with_system(scorers::fixed_score_system.system())
                .with_system(scorers::all_or_nothing_system.system())
                .with_system(scorers::sum_of_scorers_system.system())
                .with_system(scorers::winning_scorer_system.system())
                .with_system(scorers::evaluating_scorer_system.system())
                .label("scorers"),
        );
        app.add_system_to_stage(First, thinker::thinker_system.system().after("scorers"));

        app.add_system_set_to_stage(
            PreUpdate,
            SystemSet::new()
                .with_system(actions::steps_system.system())
                .with_system(actions::concurrent_system.system())
                .label("aggregate-actions"),
        );

        // run your actions in PreUpdate after aggregate-actions or in a later stage

        app.add_system_to_stage(Last, thinker::thinker_component_attach_system.system());
        app.add_system_to_stage(Last, thinker::thinker_component_detach_system.system());
        app.add_system_to_stage(Last, thinker::actor_gone_cleanup.system());
    }
}
