use bevy::{
    color::palettes,
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use vleue_navigator2d::{NavMesh, VleueNavigatorPlugin};

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
        ))
        .add_event::<NewPathStepEvent>()
        .insert_resource(PathToDisplay::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                on_mesh_change,
                mesh_change,
                on_click,
                compute_paths,
                update_path_display,
            ),
        )
        .run();
}

#[derive(Resource)]
struct Meshes {
    simple: Handle<NavMesh>,
    arena: Handle<NavMesh>,
    aurora: Handle<NavMesh>,
}

enum CurrentMesh {
    Simple,
    Arena,
    Aurora,
}

#[derive(Resource)]
struct MeshDetails {
    mesh: CurrentMesh,
    size: Vec2,
}

const SIMPLE: MeshDetails = MeshDetails {
    mesh: CurrentMesh::Simple,
    size: Vec2::new(13.0, 8.0),
};

const ARENA: MeshDetails = MeshDetails {
    mesh: CurrentMesh::Arena,
    size: Vec2::new(49.0, 49.0),
};

const AURORA: MeshDetails = MeshDetails {
    mesh: CurrentMesh::Aurora,
    size: Vec2::new(1024.0, 768.0),
};

fn setup(
    mut commands: Commands,
    mut navmeshes: ResMut<Assets<NavMesh>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);
    commands.insert_resource(Meshes {
        simple: navmeshes.add(NavMesh::from_polyanya_mesh(
            polyanya::Mesh::new(
                vec![
                    polyanya::Vertex::new(Vec2::new(0., 6.), vec![0, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(2., 5.), vec![0, u32::MAX, 2]),
                    polyanya::Vertex::new(Vec2::new(5., 7.), vec![0, 2, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(5., 8.), vec![0, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(0., 8.), vec![0, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(1., 4.), vec![1, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(2., 1.), vec![1, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(4., 1.), vec![1, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(4., 2.), vec![1, u32::MAX, 2]),
                    polyanya::Vertex::new(Vec2::new(2., 4.), vec![1, 2, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(7., 4.), vec![2, u32::MAX, 4]),
                    polyanya::Vertex::new(Vec2::new(10., 7.), vec![2, 4, 6, u32::MAX, 3]),
                    polyanya::Vertex::new(Vec2::new(7., 7.), vec![2, 3, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(11., 8.), vec![3, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(7., 8.), vec![3, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(7., 0.), vec![5, 4, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(11., 3.), vec![4, 5, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(11., 5.), vec![4, u32::MAX, 6]),
                    polyanya::Vertex::new(Vec2::new(12., 0.), vec![5, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(12., 3.), vec![5, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(13., 5.), vec![6, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(13., 7.), vec![6, u32::MAX]),
                    polyanya::Vertex::new(Vec2::new(1., 3.), vec![1, u32::MAX]),
                ],
                vec![
                    polyanya::Polygon::new(vec![0, 1, 2, 3, 4], true),
                    polyanya::Polygon::new(vec![5, 22, 6, 7, 8, 9], true),
                    polyanya::Polygon::new(vec![1, 9, 8, 10, 11, 12, 2], false),
                    polyanya::Polygon::new(vec![12, 11, 13, 14], true),
                    polyanya::Polygon::new(vec![10, 15, 16, 17, 11], false),
                    polyanya::Polygon::new(vec![15, 18, 19, 16], true),
                    polyanya::Polygon::new(vec![11, 17, 20, 21], true),
                ],
            )
            .unwrap(),
        )),
        arena: asset_server.load("arena-merged.polyanya.mesh"),
        aurora: asset_server.load("aurora-merged.polyanya.mesh"),
    });
    commands.insert_resource(SIMPLE);
}

#[derive(Default, Resource)]
struct PathToDisplay {
    steps: Vec<Vec2>,
}

fn on_mesh_change(
    mut path_to_display: ResMut<PathToDisplay>,
    mesh: Res<MeshDetails>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    navmeshes: Res<Assets<NavMesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    known_meshes: Res<Meshes>,
    mut current_mesh_entity: Local<Option<Entity>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    window_resized: EventReader<WindowResized>,
    text: Query<Entity, With<Text>>,
) {
    if !mesh.is_changed() && window_resized.is_empty() {
        return;
    }
    path_to_display.steps.clear();
    let handle = match mesh.mesh {
        CurrentMesh::Simple => &known_meshes.simple,
        CurrentMesh::Arena => &known_meshes.arena,
        CurrentMesh::Aurora => &known_meshes.aurora,
    };
    let navmesh = navmeshes.get(handle).unwrap();
    if let Some(entity) = *current_mesh_entity {
        commands.entity(entity).despawn();
    }
    let Ok(window) = primary_window.single() else {
        return;
    };
    let factor = (window.width() / mesh.size.x).min(window.height() / mesh.size.y);

    *current_mesh_entity = Some(
        commands
            .spawn((
                Mesh2d(meshes.add(navmesh.to_mesh())),
                Transform::from_translation(Vec3::new(
                    -mesh.size.x / 2.0 * factor,
                    -mesh.size.y / 2.0 * factor,
                    0.0,
                ))
                .with_scale(Vec3::splat(factor)),
                MeshMaterial2d(
                    materials.add(ColorMaterial::from(Color::Srgba(palettes::css::BLUE))),
                ),
            ))
            .with_children(|main_mesh| {
                main_mesh.spawn((
                    Mesh2d(meshes.add(navmesh.to_wireframe_mesh())),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                    MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.5, 0.5, 1.0)))),
                ));
            })
            .id(),
    );
    if let Ok(entity) = text.single() {
        commands.entity(entity).despawn();
    }
    commands
        .spawn((
            Text::default(),
            Node {
                position_type: PositionType::Absolute,
                margin: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn((
                TextSpan::new(
                    match mesh.mesh {
                        CurrentMesh::Simple => "Simple\n",
                        CurrentMesh::Arena => "Arena\n",
                        CurrentMesh::Aurora => "Aurora\n",
                    }
                    .to_string(),
                ),
                TextFont {
                    font_size: 25.0,
                    ..default()
                },
            ));
            p.spawn((
                TextSpan::new("Press spacebar to switch mesh\n".to_string()),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
            ));
            p.spawn((
                TextSpan::new("Click to find a path".to_string()),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
            ));
        });
}

fn mesh_change(mut mesh: ResMut<MeshDetails>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        match mesh.mesh {
            CurrentMesh::Simple => *mesh = ARENA,
            CurrentMesh::Arena => *mesh = AURORA,
            CurrentMesh::Aurora => *mesh = SIMPLE,
        }
    }
}

#[derive(Event)]
struct NewPathStepEvent(Vec2);

fn on_click(
    mut path_step_event: EventWriter<NewPathStepEvent>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    primary_window: Single<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mesh: Res<MeshDetails>,
    meshes: Res<Meshes>,
    navmeshes: Res<Assets<NavMesh>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let Ok((camera, camera_transform)) = camera_q.single() else {
            return;
        };
        let window = *primary_window;
        if let Some(position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
            .map(|ray| ray.origin.truncate())
        {
            let screen = Vec2::new(window.width(), window.height());
            let factor = (screen.x / mesh.size.x).min(screen.y / mesh.size.y);

            let in_mesh = position / factor + mesh.size / 2.0;
            if navmeshes
                .get(match mesh.mesh {
                    CurrentMesh::Simple => &meshes.simple,
                    CurrentMesh::Arena => &meshes.arena,
                    CurrentMesh::Aurora => &meshes.aurora,
                })
                .map(|mesh| mesh.is_in_mesh(in_mesh))
                .unwrap_or_default()
            {
                info!("going to {}", in_mesh);
                path_step_event.write(NewPathStepEvent(in_mesh));
            } else {
                info!("clicked outside of mesh");
            }
        }
    }
}

fn compute_paths(
    mut event_new_step_path: EventReader<NewPathStepEvent>,
    mut path_to_display: ResMut<PathToDisplay>,
    mesh: Res<MeshDetails>,
    meshes: Res<Meshes>,
    navmeshes: Res<Assets<NavMesh>>,
) {
    for ev in event_new_step_path.read() {
        if path_to_display.steps.is_empty() {
            path_to_display.steps.push(ev.0);
            return;
        }

        let navmesh = navmeshes
            .get(match mesh.mesh {
                CurrentMesh::Simple => &meshes.simple,
                CurrentMesh::Arena => &meshes.arena,
                CurrentMesh::Aurora => &meshes.aurora,
            })
            .unwrap();
        if let Some(path) = navmesh.path(*path_to_display.steps.last().unwrap(), ev.0) {
            for p in path.path {
                path_to_display.steps.push(p);
            }
        } else {
            info!("no path found");
        }
    }
}

fn update_path_display(
    path_to_display: Res<PathToDisplay>,
    mut gizmos: Gizmos,
    mesh: Res<MeshDetails>,
    primary_window: Single<&Window, With<PrimaryWindow>>,
) {
    let window = *primary_window;
    let factor = (window.width() / mesh.size.x).min(window.height() / mesh.size.y);

    let path = path_to_display
        .steps
        .iter()
        .map(|p| (*p - mesh.size / 2.0) * factor);

    if path.len() >= 1 {
        gizmos.linestrip_2d(path, palettes::css::YELLOW);
    }
}
