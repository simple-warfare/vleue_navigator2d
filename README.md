# Navigation for Bevy with NavMesh

![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)
[![Release Doc](https://docs.rs/vleue_navigator2d/badge.svg)](https://docs.rs/vleue_navigator2d)
[![Crate](https://img.shields.io/crates/v/vleue_navigator2d.svg)](https://crates.io/crates/vleue_navigator2d)

Navigation mesh for [Bevy](http://github.com/bevyengine/bevy) using [Polyanya](https://github.com/vleue/polyanya).

![map with many points finding their paths](https://raw.githubusercontent.com/vleue/vleue_navigator2d/main/screenshots/many.png)

Check out the [WASM demos](https://vleue.github.io/vleue_navigator2d/)

## Usage

### From a prebuilt NavMesh

Loading a mesh from a gLTF file, then building a `NavMesh` from it and using it for getting paths. See [gltf.rs](https://github.com/vleue/vleue_navigator2d/blob/main/examples/gltf.rs) and [`NavMesh::from_bevy_mesh`](https://docs.rs/vleue_navigator2d/latest/vleue_navigator2d/struct.NavMesh.html#method.from_bevy_mesh).

### From obstacle components

Spawn entities marked as obstacles, create the NavMesh live from them. See [auto_navmesh_aabb](https://github.com/vleue/vleue_navigator2d/blob/main/examples/auto_navmesh_aabb.rs) and [`NavMeshUpdaterPlugin`](https://docs.rs/vleue_navigator2d/latest/vleue_navigator2d/prelude/struct.NavmeshUpdaterPlugin.html).

## Example videos

[Pathfinding many agents](https://www.youtube.com/watch?v=Zi9EMAdHp4M).

[Parameters for NavMesh generation](https://www.youtube.com/watch?v=wYRrvWaLjJ8)

## Reading list

Pathfinding:

- [Compromise-free Pathfinding on a Navigation Mesh](https://www.ijcai.org/proceedings/2017/0070.pdf): Fast and optimal path finding on a generalized navmesh

NavMesh building:

- [Line Generalisation by Repeated Elimination of Points](https://hull-repository.worktribe.com/preview/376364/000870493786962263.pdf): Geometry simplification while keeping the general shape
- [Constrained Delaunay Triangulation](https://en.wikipedia.org/wiki/Constrained_Delaunay_triangulation): Building a tri-mesh from edges
- [Polygon Offsetting by Computing Winding Numbers](https://mcmains.me.berkeley.edu/pubs/DAC05OffsetPolygon.pdf): Agent radius

### To Implement

- Steering Behaviors For Autonomous Characters https://www.red3d.com/cwr/steer/

## Bevy Supported Versions

| Bevy | vleue_navigator2d | avian |
| ---- | --------------- | ----- |
| 0.16 | 0.12            | 0.3   |
| 0.15 | 0.11.1          | 0.2   |
| 0.14 | 0.10            | 0.1   |
| 0.13 | 0.7             | x     |
