use std::cmp::Reverse;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy::picking::PickSet;
use bevy::prelude::*;
use ordered_float::OrderedFloat;

//-------------------------------------------------------------------------------------------------------------------

/// Used to sort the entities within a sprite layer.
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ZIndexSortKey(Reverse<OrderedFloat<f32>>);

impl ZIndexSortKey
{
    // This is reversed because bevy uses +y pointing upwards, which is the
    // opposite of what you generally want.
    fn new(transform: &GlobalTransform) -> Self
    {
        Self(Reverse(OrderedFloat(transform.translation().y)))
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn clear_z_coordinates<Layer: LayerIndex>(mut query: Query<&mut Transform, With<Layer>>)
{
    for mut transform in query.iter_mut() {
        transform.bypass_change_detection().translation.z = 0.0;
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn propagate_layers_recursive<Layer: LayerIndex>(
    need_sortkey: bool,
    entity: Entity,
    propagated_layer: Layer,
    maybe_children: Option<&Children>,
    transforms: &Query<&mut GlobalTransform>,
    query: &Query<Option<&Children>>,
    layers: &mut Vec<(ZIndexSortKey, f32, Entity)>,
)
{
    let sort_key = match need_sortkey {
        true => transforms
            .get(entity)
            .map(ZIndexSortKey::new)
            .unwrap_or_default(),
        false => ZIndexSortKey::default(),
    };
    layers.push((sort_key, propagated_layer.as_z_coordinate(), entity));

    let Some(children) = maybe_children else { return };
    for child in children {
        let Ok(maybe_children) = query.get(*child) else { continue };
        propagate_layers_recursive(
            need_sortkey,
            *child,
            propagated_layer,
            maybe_children,
            transforms,
            query,
            layers,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets the given entity's global transform z. Does nothing if it doesn't have one.
fn set_transform_z(query: &mut Query<&mut GlobalTransform>, entity: Entity, z: f32)
{
    let Some(mut transform) = query.get_mut(entity).ok() else { return };
    let transform = transform.bypass_change_detection();
    let mut affine = transform.affine();
    affine.translation.z = z;
    *transform = GlobalTransform::from(affine);
}

//-------------------------------------------------------------------------------------------------------------------

/// Propagates the `Layer` of each entity to its descendants.
fn update_z_coordinates<Layer: LayerIndex>(
    mut layers: Local<Vec<(ZIndexSortKey, f32, Entity)>>,
    options: Res<SpriteLayerOptions>,
    root_layers: Query<(Entity, &Layer, Option<&Children>)>,
    children: Query<Option<&Children>>,
    mut transforms: Query<&mut GlobalTransform>,
)
{
    layers.clear();

    // Propagate layers to children.
    for (entity, layer, maybe_children) in &root_layers {
        propagate_layers_recursive(
            options.y_sort,
            entity,
            *layer,
            maybe_children,
            &transforms,
            &children,
            &mut layers,
        );
    }

    // Compute the z-coordinate that each entity should have. This is equal to its layer's equivalent
    // z-coordinate, plus an offset in the range [0, 1) corresponding to its y-sorted position
    // (if y-sorting is enabled).
    if layers.is_empty() {
        return;
    }

    if options.y_sort {
        // We y-sort everything because this avoids the overhead of grouping
        // entities by their layer.
        layers.sort_unstable_by(|(a, _, _), (b, _, _)| a.cmp(b));

        let scale_factor = 1.0 / layers.len() as f32;
        for (i, (_, layer_z, entity)) in layers.iter().enumerate() {
            let z = *layer_z + (i as f32) * scale_factor;
            set_transform_z(&mut transforms, *entity, z);
        }
    } else {
        for (_, layer_z, entity) in layers.iter() {
            set_transform_z(&mut transforms, *entity, *layer_z);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for the type you use to indicate your sprites' layers. Add this as a
/// component to any entity you want to treat as a sprite. Note that this will
/// propagate to children. Entities with a `LayerIndex` component are treated
/// as root entities for propagation even if they are not hierarchically root entities.
// TODO: use LayerOverride(Layer) wrapper component to allow descendant subtrees to override their parents' layer.
pub trait LayerIndex: Eq + Hash + Component + Copy + Clone + Debug
{
    /// The actual numeric z-value that the layer index corresponds to.  Note
    /// that the z-value for an entity can be any value in the range
    /// `layer.as_z_coordinate() <= z < layer.as_z_coordinate() + 1.0`, and the
    /// exact values are an implementation detail!
    ///
    /// With the default Bevy camera settings, your return values from this
    /// function should be between 0 and 999.0, since the camera is at z =
    /// 1000.0. Prefer smaller z-values since that gives more precision.
    fn as_z_coordinate(&self) -> f32;
}

//-------------------------------------------------------------------------------------------------------------------

/// Resource that configures how the sprite layer is handled.
#[derive(Resource, Debug, Reflect)]
pub struct SpriteLayerOptions
{
    pub y_sort: bool,
}

impl Default for SpriteLayerOptions
{
    fn default() -> Self
    {
        Self { y_sort: true }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Set for all systems related to [`SpriteLayerPlugin`].
#[derive(SystemSet, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum SpriteLayerSet
{
    /// Runs in `PreUpdate` after `PickSet::Backend`.
    ClearZCoordinates,
    /// Runs in `Last`.
    SetZCoordinates,
}

//-------------------------------------------------------------------------------------------------------------------

/// This plugin adjusts your entities' transforms so that their z-coordinates are sorted in the
/// proper order, where the order is specified by the `Layer` component. Layers propagate to
/// children (including through entities with no )
///
/// Layers propagate to children, including 'through' entities with no [`GlobalTransform`].
///
/// If you need to know the z-coordinate, you can read it out of the [`GlobalTransform`] after the
/// [`SpriteLayer::SetZCoordinates`] set has run.
///
/// In general you should only instantiate this plugin with a single type you use throughout your
/// program.
///
/// By default your sprites will also be y-sorted. If you don't need this, replace the
/// [`SpriteLayerOptions`] like so:
///
/// ```
/// # use bevy::prelude::*;
/// # use extol_sprite_layer::SpriteLayerOptions;
/// # let mut app = App::new();
/// app.insert_resource(SpriteLayerOptions { y_sort: false });
/// ```
pub struct SpriteLayerPlugin<Layer>
{
    phantom: PhantomData<Layer>,
}

impl<Layer> Default for SpriteLayerPlugin<Layer>
{
    fn default() -> Self
    {
        Self { phantom: Default::default() }
    }
}

impl<Layer: LayerIndex> Plugin for SpriteLayerPlugin<Layer>
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<SpriteLayerOptions>()
            .configure_sets(PreUpdate, SpriteLayerSet::ClearZCoordinates.after(PickSet::Backend))
            .add_systems(
                PreUpdate,
                clear_z_coordinates::<Layer>.in_set(SpriteLayerSet::ClearZCoordinates),
            )
            .add_systems(
                Last,
                // We need to run this *after* the transform's systems because they need the
                // proper y-coordinate to be set for y-sorting.
                update_z_coordinates::<Layer>.in_set(SpriteLayerSet::SetZCoordinates),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests
{
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Component)]
    enum Layer
    {
        Top,
        Middle,
        Bottom,
    }

    impl LayerIndex for Layer
    {
        fn as_z_coordinate(&self) -> f32
        {
            use Layer::*;
            match self {
                Bottom => 0.0,
                Middle => 1.0,
                Top => 2.0,
            }
        }
    }

    fn test_app() -> App
    {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(TransformPlugin)
            .add_plugins(SpriteLayerPlugin::<Layer>::default());

        app
    }

    /// Just verify that adding the plugin doesn't somehow blow everything up.
    #[test]
    fn plugin_add_smoke_check()
    {
        let _ = test_app();
    }

    fn transform_at(x: f32, y: f32) -> Transform
    {
        Transform::from_xyz(x, y, 0.0)
    }

    fn get_z(world: &World, entity: Entity) -> f32
    {
        world
            .get::<GlobalTransform>(entity)
            .unwrap()
            .translation()
            .z
    }

    #[test]
    fn simple()
    {
        let mut app = test_app();
        let top = app
            .world_mut()
            .spawn((transform_at(1.0, 1.0), Layer::Top))
            .id();
        let middle = app
            .world_mut()
            .spawn((transform_at(1.0, 1.0), Layer::Middle))
            .id();
        let bottom = app
            .world_mut()
            .spawn((transform_at(1.0, 1.0), Layer::Bottom))
            .id();
        app.update();

        assert!(get_z(app.world(), bottom) < get_z(app.world(), middle));
        assert!(get_z(app.world(), middle) < get_z(app.world(), top));
    }

    fn layer_bundle(layer: Layer) -> impl Bundle
    {
        (transform_at(0.0, 0.0), layer)
    }

    #[test]
    fn inherited()
    {
        let mut app = test_app();
        let top = app.world_mut().spawn(layer_bundle(Layer::Top)).id();
        let child_with_layer = app
            .world_mut()
            .spawn((layer_bundle(Layer::Middle), ChildOf(top)))
            .id();
        let child_without_layer = app
            .world_mut()
            .spawn((transform_at(0.0, 0.0), ChildOf(top)))
            .id();
        app.update();

        // we use .floor() here since y-sorting can add a fractional amount to the coordinates
        assert_eq!(
            get_z(app.world(), child_with_layer).floor(),
            Layer::Middle.as_z_coordinate()
        );
        assert_eq!(
            get_z(app.world(), child_without_layer).floor(),
            get_z(app.world(), top).floor()
        );
    }

    #[test]
    fn y_sorting()
    {
        let mut app = test_app();
        for i in 0..10 {
            app.world_mut()
                .spawn((transform_at(0.0, i as f32), Layer::Top));
        }
        app.update();
        let positions = app
            .world_mut()
            .run_system_once(|query: Query<&GlobalTransform>| -> Vec<Vec3> {
                query
                    .into_iter()
                    .map(|transform| transform.translation())
                    .collect()
            })
            .unwrap();
        let mut sorted_by_z = positions.clone();
        sorted_by_z.sort_by_key(|vec| OrderedFloat(vec.z));
        let mut sorted_by_y = positions;
        sorted_by_y.sort_by_key(|vec| Reverse(OrderedFloat(vec.y)));
        assert_eq!(sorted_by_z, sorted_by_y);
    }

    #[test]
    fn child_with_no_transform()
    {
        let mut app = test_app();
        let entity = app.world_mut().spawn(layer_bundle(Layer::Top)).id();
        let child = app.world_mut().spawn(ChildOf(entity)).id();
        let grandchild = app
            .world_mut()
            .spawn((transform_at(0.0, 0.0), ChildOf(child)))
            .id();
        app.update();
        assert_eq!(
            get_z(app.world(), grandchild).floor(),
            Layer::Top.as_z_coordinate()
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
