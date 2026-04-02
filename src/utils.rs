use bevy::prelude::*;

pub(crate) fn iter_descendants(
    root: Entity,
    children: &Query<&Children>,
) -> impl Iterator<Item = Entity> {
    children.iter_descendants(root)
}
