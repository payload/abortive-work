use bevy::{
    ecs::query::{Fetch, FilterFetch, ReadOnlyFetch, WorldQuery},
    prelude::{Entity, Query},
};

pub trait QueryExt<'w, 's, Item> {
    fn get_some(&'s self, opt_entity: Option<Entity>) -> Option<Item>;
}

impl<'w, 's, Q: WorldQuery + 'static, F: WorldQuery + 'static>
    QueryExt<'w, 's, <<Q as WorldQuery>::Fetch as Fetch<'w, 's>>::Item> for Query<'w, 's, Q, F>
where
    F::Fetch: FilterFetch,
    Q::Fetch: ReadOnlyFetch,
{
    fn get_some(
        &'s self,
        opt_entity: Option<Entity>,
    ) -> Option<<<Q as WorldQuery>::Fetch as Fetch<'w, 's>>::Item> {
        opt_entity.and_then(|e| self.get(e).ok())
    }
}
