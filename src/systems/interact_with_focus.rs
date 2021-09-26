use bevy::{ecs::system::SystemParam, prelude::*};

use crate::entities::{tree, Boulder, ConveyorBelt, Fireplace, Mage, Pile};

use super::Focus;

pub struct ModPlugin;
impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractWithFocusEvent>().add_system(update);
    }
}
pub struct InteractWithFocusEvent;

fn update(
    mut interact_with_focus: InteractWithFocus,
    mut events: EventReader<InteractWithFocusEvent>,
) {
    for _ in events.iter() {
        interact_with_focus.interact();
    }
}

#[derive(SystemParam)]
pub struct InteractWithFocus<'w, 's> {
    focus: Query<'w, 's, &'static Focus>,
    mage: Query<'w, 's, &'static mut Mage>,
    trees: Query<'w, 's, &'static mut tree::Tree>,
    boulders: Query<'w, 's, &'static mut Boulder>,
    belts: Query<'w, 's, &'static mut ConveyorBelt>,
    piles: Query<'w, 's, &'static mut Pile>,
    fireplaces: Query<'w, 's, &'static mut Fireplace>,
}

impl<'w, 's> InteractWithFocus<'w, 's> {
    pub fn interact(&mut self) {
        let mut mage = self.mage.single_mut();
        let focus = self.focus.single();
        let entity = if let Some(entity) = focus.entity {
            entity
        } else {
            return;
        };

        if let Ok(mut boulder) = self.boulders.get_mut(entity) {
            boulder.marked_for_digging = !boulder.marked_for_digging;
        }

        if let Ok(mut pile) = self.piles.get_mut(entity) {
            if pile.amount >= 1.0 {
                pile.amount -= 1.0;
                mage.put_into_inventory(pile.load, 1.0);
            }
        }

        if let Ok(mut belt) = self.belts.get_mut(entity) {
            if let Some(thing) = mage.take_first(1.0) {
                belt.store(thing, 1.0);
            } else {
                belt.marked_for_thing = None;
            }
        }

        if let Ok(mut tree) = self.trees.get_mut(entity) {
            tree.mark_cut_tree = !tree.mark_cut_tree;
        }

        if let Ok(mut fireplace) = self.fireplaces.get_mut(entity) {
            fireplace.lit = !fireplace.lit;
        }
    }
}
