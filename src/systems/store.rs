#![allow(unused)]

use crate::{entities::BoulderMaterial, systems::Thing};

#[derive(Default, Clone, Debug)]
pub struct Store {
    pub slots: Vec<StoreSlot>,
}

#[derive(Clone, Debug)]
pub struct StoreSlot {
    pub stack: Stack,
    pub thing_filter: ThingFilter,
    pub limit: f32,
    pub desired_amount: f32,
    pub high_priority: i32,
    pub low_priority: i32,
    pub direction: Direction,
}

#[derive(Clone, Debug)]
pub enum ThingFilter {
    None,
    Thing(Thing),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
}

#[derive(Default, Clone, Debug)]
pub struct Stack {
    pub amount: f32,
    pub thing: Option<Thing>,
}

impl Store {
    pub fn new(slots: &[StoreSlot]) -> Self {
        Self {
            slots: slots.to_vec(),
        }
    }

    pub fn priority_of_thing(&self, thing: Thing) -> i32 {
        self.slots
            .iter()
            .map(|slot| slot.priority_of_thing(thing))
            .max()
            .unwrap_or(0)
    }

    pub fn space_for_thing(&self, thing: Thing) -> f32 {
        self.slots
            .iter()
            .map(|slot| slot.space_for_thing(thing))
            .sum()
    }

    pub fn store_thing(&mut self, amount: f32, thing: Thing) -> f32 {
        let mut rest = amount;

        for slot in self.slots.iter_mut() {
            let stored = slot.store_thing(amount, thing);
            rest -= stored;

            if rest == 0.0 {
                break;
            }
        }

        let stored = amount - rest;
        stored
    }

    pub fn amount(&self, index: usize) -> f32 {
        self.slots.get(index).map_or(0.0, |s| s.stack.amount)
    }

    pub fn decrease(&mut self, index: usize, amount: f32) -> bool {
        if let Some(slot) = self.slots.get_mut(index) {
            assert!(amount > 0.0);

            let new_amount = slot.stack.amount - amount;
            if new_amount >= 0.0 {
                slot.stack.amount = new_amount;

                if new_amount == 0.0 {
                    slot.stack.thing = None;
                }

                return true;
            }
        }
        false
    }

    pub fn store(&mut self, index: usize, amount: f32, thing: Thing) -> bool {
        if let Some(slot) = self.slots.get_mut(index) {
            assert!(amount > 0.0);

            let new_amount = slot.stack.amount + amount;
            if new_amount <= slot.limit {
                slot.stack.amount = new_amount;
                slot.stack.thing = Some(thing);
                return true;
            }
        }
        false
    }

    pub fn first_output_stack(&self) -> Option<Stack> {
        self.slots
            .iter()
            .filter(|slot| slot.direction == Direction::Output)
            .next()
            .map(|slot| slot.stack.clone())
    }
}

impl Stack {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StoreSlot {
    pub fn new(limit: f32, filter: ThingFilter, direction: Direction) -> Self {
        Self {
            stack: Stack::new(),
            thing_filter: filter,
            limit,
            desired_amount: limit,
            high_priority: 1,
            low_priority: 0,
            direction,
        }
    }

    pub fn input(limit: f32, filter: ThingFilter) -> Self {
        Self::new(limit, filter, Direction::Input)
    }

    pub fn output(limit: f32, filter: ThingFilter) -> Self {
        Self::new(limit, filter, Direction::Output)
    }

    pub fn store_thing(&mut self, amount: f32, thing: Thing) -> f32 {
        let space = self.space_for_thing(thing);
        let stored = amount.min(space);
        if stored > 0.0 {
            self.stack.amount += stored;
            self.stack.thing = Some(thing);
        }
        stored
    }

    pub fn priority_of_thing(&self, thing: Thing) -> i32 {
        if self.space_for_thing(thing) == 0.0 {
            0
        } else if self.stack.amount >= self.desired_amount {
            self.low_priority
        } else {
            self.high_priority
        }
    }

    pub fn space_for_thing(&self, thing: Thing) -> f32 {
        if let Some(content) = self.stack.thing {
            assert!(self.stack.amount > 0.0);

            if content == thing {
                self.limit - self.stack.amount
            } else {
                0.0
            }
        } else {
            assert!(self.stack.amount == 0.0);

            if self.thing_filter.matches(thing) {
                self.limit
            } else {
                0.0
            }
        }
    }
}

impl ThingFilter {
    pub fn matches(&self, thing: Thing) -> bool {
        match self {
            ThingFilter::None => true,
            ThingFilter::Thing(filter) => *filter == thing,
        }
    }
}

impl From<Thing> for ThingFilter {
    fn from(thing: Thing) -> Self {
        Self::Thing(thing)
    }
}

impl From<BoulderMaterial> for Thing {
    fn from(material: BoulderMaterial) -> Self {
        match material {
            BoulderMaterial::Stone => Self::Stone,
            BoulderMaterial::Coal => Self::Coal,
            BoulderMaterial::Iron => Self::Iron,
            BoulderMaterial::Gold => Self::Gold,
        }
    }
}

#[test]
fn example() {
    let _store = Store::new(&[StoreSlot::input(1.0, Thing::Stone.into())]);
}

// TODO consider store slot direction
