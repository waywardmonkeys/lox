//! Everything related to the `SharedVertexMesh`.

use std::fmt;

use crate as lox;
use crate::{
    Empty,
    handle::{hsize, FaceHandle, VertexHandle},
    map::{VecMap, PropMap, PropStoreMut},
    traits::{TriVerticesOfFace, SupportsMultiBlade, Mesh, TriMesh, TriMeshMut, MeshMut},
};



#[derive(Clone, Empty)]
pub struct SharedVertexMesh {
    vertices: VecMap<VertexHandle, ()>,
    faces: VecMap<FaceHandle, [VertexHandle; 3]>,
}

impl SharedVertexMesh {
    pub fn new() -> Self {
        Self::empty()
    }
}

impl Mesh for SharedVertexMesh {
    fn num_vertices(&self) -> hsize {
        self.vertices.num_elements()
    }

    fn vertex_handles(&self) -> Box<Iterator<Item = VertexHandle> + '_> {
        Box::new(self.vertices.handles())
    }

    fn contains_vertex(&self, vertex: VertexHandle) -> bool {
        self.vertices.contains_handle(vertex)
    }

    fn num_faces(&self) -> hsize {
        self.faces.num_elements()
    }

    fn face_handles(&self) -> Box<Iterator<Item = FaceHandle> + '_> {
        Box::new(self.faces.handles())
    }

    fn contains_face(&self, face: FaceHandle) -> bool {
        self.faces.contains_handle(face)
    }
}

impl MeshMut for SharedVertexMesh {
    fn add_vertex(&mut self) -> VertexHandle {
        self.vertices.push(())
    }

    fn remove_all_vertices(&mut self) {
        assert!(
            self.num_faces() == 0,
            "call to `remove_all_vertices`, but there are faces in the mesh!",
        );

        self.vertices.clear();
    }

    fn remove_all_faces(&mut self) {
        self.faces.clear();
    }
}


impl TriMesh for SharedVertexMesh {}

impl TriMeshMut for SharedVertexMesh {
    fn add_face(&mut self, vertices: [VertexHandle; 3]) -> FaceHandle {
        assert_ne!(vertices[0], vertices[1], "vertices of new face are not unique");
        assert_ne!(vertices[0], vertices[2], "vertices of new face are not unique");

        self.faces.push(vertices)
    }
}

impl TriVerticesOfFace for SharedVertexMesh {
    fn vertices_of_face(&self, face: FaceHandle) -> [VertexHandle; 3] {
        self.faces[face]
    }
}

impl SupportsMultiBlade for SharedVertexMesh {}


impl fmt::Debug for SharedVertexMesh {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct VerticesDebug<'a>(&'a VecMap<VertexHandle, ()>);
        impl fmt::Debug for VerticesDebug<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_list().entries(self.0.handles()).finish()
            }
        }

        f.debug_struct("SharedVertexMesh")
            .field("vertices", &VerticesDebug(&self.vertices))
            .field("faces", &self.faces)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    gen_tri_mesh_tests!(SharedVertexMesh: [TriVerticesOfFace, SupportsMultiBlade]);
}
