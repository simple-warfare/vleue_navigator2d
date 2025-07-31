use bevy::{
    color::palettes,
    math::vec2,
    platform::collections::HashMap,
    prelude::*,
    window::WindowResized,
};
use bevy_ecs_tiled::prelude::*;
use parry2d::shape::TypedShape;
use parry2d::{
    math::{Isometry, Real},
    shape::SharedShape,
};
use polyanya::Triangulation;
use std::ops::Deref;
use tiled::{ObjectLayerData, ObjectShape};
use vleue_navigator2d::prelude::*;
use vleue_navigator2d::prelude::{CachableObstacle, CachedObstacle, SharedShapeStorage};

use crate::ui::ShowingNavMesh;
#[path = "helpers/agent2d.rs"]
mod agent;
#[path = "helpers/ui.rs"]
mod ui;

const MESH_WIDTH: u32 = 150;
const MESH_HEIGHT: u32 = 100;

fn main() {
    App::new()
        .insert_resource(ClearColor(palettes::css::BLACK.into()))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Navmesh with Polyanya".to_string(),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            VleueNavigatorPlugin,
            // Auto update the navmesh.
            // Obstacles will be entities with the `Obstacle` marker component,
            // and use the `SharedShape` component as the obstacle data source.
            NavmeshUpdaterPlugin::<CachedObstacle<SharedShapeStorage>>::default(),
        ))
        .add_plugins(TiledMapPlugin::default())
        .add_plugins(TiledPhysicsPlugin::<CustomPhysicsBackend>::default())
        .add_systems(
            Startup,
            (
                ui::setup_stats::<true>,
                ui::setup_settings::<false>,
                agent::setup_agent::<10, 10, 1>,
                setup,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                display_obstacle,
                display_mesh,
                ui::update_stats::<CachedObstacle<SharedShapeStorage>>,
                ui::display_settings,
                ui::update_settings::<10>,
                agent::give_target_to_navigator::<10, MESH_WIDTH, MESH_HEIGHT>,
                agent::move_navigator,
                agent::display_navigator_path,
                agent::refresh_path::<10, MESH_WIDTH, MESH_HEIGHT>,
            ),
        )
        .run();
}


fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TiledMapHandle(asset_server.load("maps/[p2]Small_Island (2p).tmx")),
        TilemapAnchor::Center,
    ));

    commands.spawn(Camera2d);

    commands.add_observer(
        |trigger: Trigger<TiledMapCreated>,
         mut commands: Commands,
         mut showing_navmesh: ResMut<ShowingNavMesh>,
         map_asset: Res<Assets<TiledMap>>| {
            let map = &map_asset.get(trigger.asset_id).unwrap().map;
            let tiled_width = map.tile_width as f32;
            let tiled_height = map.tile_height as f32;
            let tilemap_size = map_asset.get(trigger.asset_id).unwrap().tilemap_size;
            // Spawn a new navmesh that will be automatically updated.
            let navmesh = commands
                .spawn((
                    NavMeshSettings {
                        // Define the outer borders of the navmesh.
                        // This will be in navmesh coordinates
                        fixed: Triangulation::from_outer_edges(&[
                            vec2(0.0, 0.0),
                            vec2(tilemap_size.x as f32 * tiled_width, 0.0),
                            vec2(
                                tilemap_size.x as f32 * tiled_width,
                                tilemap_size.y as f32 * tiled_height,
                            ),
                            vec2(0.0, tilemap_size.y as f32 * tiled_height),
                        ]),
                        // Starting with a small mesh simplification factor to avoid very small geometry.
                        // Small geometry can make navmesh generation fail due to rounding errors.
                        // This example has round obstacles which can create small details.
                        simplify: 0.2,
                        merge_steps: 2,
                        ..default()
                    },
                    // Mark it for update as soon as obstacles are changed.
                    // Other modes can be debounced or manually triggered.
                    NavMeshUpdateMode::Direct,
                    // This transform places the (0, 0) point of the navmesh, and is used to transform coordinates from the world to the navmesh.
                    Transform::from_translation(Vec3::new(
                        -(tilemap_size.x as f32 * tiled_width) / 2.0,
                        -(tilemap_size.y as f32 * tiled_height) / 2.0,
                        0.0,
                    )),
                ))
                .id();

            showing_navmesh.0 = Some(navmesh);
        },
    );
}

fn display_obstacle(mut gizmos: Gizmos, query: Query<(&SharedShapeStorage, &Transform)>) {
    for (shape, transform) in &query {
        match shape.shape_scaled().as_typed_shape() {
            TypedShape::Ball(ball) => {
                gizmos.circle_2d(
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    ball.radius,
                    Color::WHITE,
                );
            }
            TypedShape::Cuboid(cuboid) => {
                gizmos.rect_2d(
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    (cuboid.half_extents.xy() * 2.0).into(),
                    Color::WHITE,
                );
            }
            TypedShape::Capsule(capsule) => {
                gizmos.primitive_2d(
                    &Capsule2d::new(capsule.radius, capsule.height()),
                    Isometry2d::new(
                        transform.translation.xy(),
                        Rot2::radians(transform.rotation.to_axis_angle().1),
                    ),
                    Color::WHITE,
                );
            }
            _ => {}
        }
    }
}

fn display_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    navmeshes: Res<Assets<NavMesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut current_mesh_entity: Local<Option<Entity>>,
    window_resized: EventReader<WindowResized>,
    navmesh: Single<(&ManagedNavMesh, Ref<NavMeshStatus>)>,
) {
    let (navmesh_handle, status) = navmesh.deref();
    if (!status.is_changed() || **status != NavMeshStatus::Built) && window_resized.is_empty() {
        return;
    }

    let Some(navmesh) = navmeshes.get(*navmesh_handle) else {
        return;
    };
    if let Some(entity) = *current_mesh_entity {
        commands.entity(entity).despawn();
    }

    *current_mesh_entity = Some(
        commands
            .spawn((
                Mesh2d(meshes.add(navmesh.to_mesh())),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::Srgba(
                    palettes::tailwind::BLUE_800,
                )))),
            ))
            .with_children(|main_mesh| {
                main_mesh.spawn((
                    Mesh2d(meshes.add(navmesh.to_wireframe_mesh())),
                    MeshMaterial2d(materials.add(ColorMaterial::from(Color::Srgba(
                        palettes::tailwind::TEAL_300,
                    )))),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                ));
            })
            .id(),
    );
}

#[derive(Default, Debug, Clone, Reflect)]
#[reflect(Default, Debug)]
pub struct CustomPhysicsBackend;

impl TiledPhysicsBackend for CustomPhysicsBackend {
    fn spawn_colliders(
        &self,
        commands: &mut Commands,
        tiled_map: &TiledMap,
        filter: &TiledNameFilter,
        collider: &TiledCollider,
        anchor: &TilemapAnchor,
    ) -> Vec<TiledColliderSpawnInfos> {
        info!("Spawning colliders for: {:?}", collider);
        match collider {
            TiledCollider::Object {
                layer_id: _,
                object_id: _,
            } => {
                let Some(object) = collider.get_object(tiled_map) else {
                    return vec![];
                };

                match object.get_tile() {
                    Some(object_tile) => object_tile.get_tile().and_then(|tile| {
                        let Some(object_layer_data) = &tile.collision else {
                            return None;
                        };
                        let mut composables = HashMap::new();
                        let mut spawn_infos = vec![];
                        compose_tiles(
                            commands,
                            filter,
                            object_layer_data,
                            Vec2::ZERO,
                            get_grid_size(&tiled_map.map),
                            &mut composables,
                            &mut spawn_infos,
                        );
                        if !composables.is_empty() {
                            composables.iter().for_each(|(user_type, composables)| {
                                let shared_shape = SharedShape::compound(composables.to_vec());

                                spawn_infos.push(TiledColliderSpawnInfos {
                                    name: format!("{}[ComposedTile]", user_type),
                                    entity: commands
                                        .spawn((
                                            CachedObstacle::<SharedShapeStorage>::new(
                                                SharedShapeStorage::from(shared_shape),
                                            ),
                                            CachableObstacle,
                                        ))
                                        .id(),
                                    transform: Transform::default(),
                                });
                            });
                        };
                        Some(spawn_infos)
                    }),
                    None => get_position_and_shape(&object.shape).map(|(pos, shared_shape, _)| {
                        let iso = Isometry3d::from_rotation(Quat::from_rotation_z(
                            f32::to_radians(-object.rotation),
                        )) * Isometry3d::from_xyz(pos.x, pos.y, 0.);

                        vec![TiledColliderSpawnInfos {
                            name: format!("Custom[Object={}]", object.name),
                            entity: commands
                                .spawn((
                                    CachedObstacle::<SharedShapeStorage>::new(
                                        SharedShapeStorage::from(shared_shape),
                                    ),
                                    CachableObstacle,
                                ))
                                .id(),
                            transform: Transform::from_isometry(iso),
                        }]
                    }),
                }
                .unwrap_or_default()
            }

            TiledCollider::TilesLayer { layer_id: _ } => {
                let mut composables = HashMap::new();
                let mut spawn_infos = vec![];
                for (tile_position, tile) in collider.get_tiles(tiled_map, anchor) {
                    if let Some(collision) = &tile.collision {
                        compose_tiles(
                            commands,
                            filter,
                            collision,
                            tile_position,
                            get_grid_size(&tiled_map.map),
                            &mut composables,
                            &mut spawn_infos,
                        );
                    }
                }
                if !composables.is_empty() {
                    composables.iter().for_each(|(user_type, composables)| {
                        let shared_shape = SharedShape::compound(composables.to_vec());

                        spawn_infos.push(TiledColliderSpawnInfos {
                            name: format!("{}[ComposedTile]", user_type),
                            entity: commands
                                .spawn((
                                    CachedObstacle::<SharedShapeStorage>::new(
                                        SharedShapeStorage::from(shared_shape),
                                    ),
                                    CachableObstacle,
                                ))
                                .id(),
                            transform: Transform::default(),
                        });
                    });
                }
                spawn_infos
            }
        }
    }
}

fn compose_tiles(
    commands: &mut Commands,
    filter: &TiledNameFilter,
    object_layer_data: &ObjectLayerData,
    tile_offset: Vec2,
    grid_size: TilemapGridSize,
    composables: &mut HashMap<String, Vec<(Isometry<Real>, SharedShape)>>,
    spawn_infos: &mut Vec<TiledColliderSpawnInfos>,
) {
    for object in object_layer_data.object_data() {
        if !filter.contains(&object.name) {
            continue;
        }
        let position = tile_offset
            // Object position
            + Vec2 {
                x: object.x - grid_size.x / 2.,
                y: (grid_size.y - object.y) - grid_size.y / 2.,
            };
        if let Some((shape_offset, shared_shape, is_composable)) =
            get_position_and_shape(&object.shape)
        {
            if is_composable {
                let iso_and_shape = (
                    Isometry::<Real>::new(position.into(), f32::to_radians(-object.rotation))
                        * Isometry::<Real>::new(shape_offset.into(), 0.),
                    shared_shape,
                );
                composables
                    .entry_ref(&object.user_type)
                    .or_insert(vec![])
                    .push(iso_and_shape);
            } else {
                let iso = Isometry3d::from_xyz(position.x, position.y, 0.)
                    * Isometry3d::from_rotation(Quat::from_rotation_z(f32::to_radians(
                        -object.rotation,
                    )));

                spawn_infos.push(TiledColliderSpawnInfos {
                    name: "Custom[ComplexTile]".to_string(),
                    entity: commands
                        .spawn((
                            CachedObstacle::<SharedShapeStorage>::new(SharedShapeStorage::from(
                                shared_shape,
                            )),
                            CachableObstacle,
                        ))
                        .id(),
                    transform: Transform::from_isometry(iso),
                });
            }
        }
    }
}

fn get_position_and_shape(shape: &ObjectShape) -> Option<(Vec2, SharedShape, bool)> {
    match shape {
        ObjectShape::Rect { width, height } => {
            let shape = SharedShape::cuboid(width / 2., height / 2.);
            let pos = Vec2::new(width / 2., -height / 2.);
            Some((pos, shape, true))
        }
        ObjectShape::Ellipse { width, height } => {
            let shape = if width > height {
                SharedShape::capsule(
                    Vec2::new((-width + height) / 2., 0.).into(),
                    Vec2::new((width - height) / 2., 0.).into(),
                    height / 2.,
                )
            } else {
                SharedShape::capsule(
                    Vec2::new(0., (-height + width) / 2.).into(),
                    Vec2::new(0., (height - width) / 2.).into(),
                    width / 2.,
                )
            };
            let pos = Vec2::new(width / 2., -height / 2.);
            Some((pos, shape, true))
        }
        ObjectShape::Polyline { points } => {
            let vertices = points
                .iter()
                .map(|(x, y)| Vec2::new(*x, -*y))
                .map(|v| v.into())
                .collect();
            let shape = SharedShape::polyline(vertices, None);
            Some((Vec2::ZERO, shape, false))
        }
        ObjectShape::Polygon { points } => {
            if points.len() < 3 {
                return None;
            }

            let vertices = points
                .iter()
                .map(|(x, y)| Vec2::new(*x, -*y))
                .map(|v| v.into())
                .collect();
            let indices = (0..points.len() as u32 - 1)
                .map(|i| [i, i + 1])
                .chain([[points.len() as u32 - 1, 0]])
                .collect();
            let shape = SharedShape::polyline(vertices, Some(indices));
            Some((Vec2::ZERO, shape, false))
        }
        _ => None,
    }
}
