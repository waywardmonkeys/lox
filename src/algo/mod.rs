use cgmath::{
    prelude::*,
    Point3,
};

use crate::{
    prelude::*,
    map::{VecMap, VertexPropMap},
    math::Pos3Like,
};


pub fn cog_smoothing<MeshT, MapT>(
    mesh: &MeshT,
    vertex_positions: &MapT,
) -> VecMap<VertexHandle, MapT::Target>
where
    MeshT: Mesh + VerticesAroundVertex,
    MapT: VertexPropMap,
    MapT::Target: Pos3Like,
{
    // TODO: things to improve
    // - calculate centroid directly from iterator instead of pushing to Vec
    //   first
    // - use cool function of `v`
    let mut out = VecMap::with_capacity(mesh.num_vertices());
    let mut positions = Vec::new();

    for v in mesh.vertices() {
        positions.clear();
        let ps = mesh.vertices_around_vertex(v.handle())
            .map(|vh| vertex_positions.get(vh).expect("missing vertex position").to_point3());
        positions.extend(ps);
        let new_pos = Point3::centroid(&positions);
        out.insert(v.handle(), new_pos.convert());
    }

    out
}
