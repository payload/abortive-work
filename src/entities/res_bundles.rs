use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};

use super::*;

#[derive(SystemParam)]
pub struct EntityAssets<'w, 's> {
    pub boulder: Res<'w, BoulderAssets>,
    pub imp: Res<'w, ImpAssets>,
    pub smithery: Res<'w, SmitheryAssets>,
    pub storage: Res<'w, StorageAssets>,

    #[system_param(ignore)]
    _secret: PhantomData<&'s ()>,
}

#[derive(SystemParam)]
pub struct EntitySpawns<'w, 's> {
    pub boulder: BoulderSpawn<'w, 's>,
    pub imp: ImpSpawn<'w, 's>,
    pub smithery: SmitherySpawn<'w, 's>,
    pub storage: StorageSpawn<'w, 's>,
}
