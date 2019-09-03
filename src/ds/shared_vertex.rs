//! Everything related to the `SharedVertexMesh`.

use std::fmt;

use crate as lox;
use crate::{
    prelude::*,
    handle::hsize,
    map::VecMap,
    mesh::SplitEdgeWithFacesResult,
    traits::marker::TriFaces,
    traits::adj::HandleIterFamily,
    util::TriArrayIntoIter,
};



#[derive(Clone, Empty)]
pub struct SharedVertexMesh {
    vertices: VecMap<VertexHandle, ()>,
    faces: VecMap<FaceHandle, [VertexHandle; 3]>,
}

impl Mesh for SharedVertexMesh {
    type FaceKind = TriFaces;

    fn num_vertices(&self) -> hsize {
        self.vertices.num_elements()
    }

    #[inline(always)]
    fn next_vertex_handle_from(&self, start: VertexHandle) -> Option<VertexHandle> {
        // TODO: optimize
        (start.idx()..self.vertices.next_push_handle().idx())
            .map(VertexHandle::new)
            .find(|&vh| self.vertices.contains_handle(vh))
    }

    #[inline(always)]
    fn next_face_handle_from(&self, start: FaceHandle) -> Option<FaceHandle> {
        // TODO: optimize
        (start.idx()..self.faces.next_push_handle().idx())
            .map(FaceHandle::new)
            .find(|&fh| self.faces.contains_handle(fh))
    }

    fn last_vertex_handle(&self) -> Option<VertexHandle> {
        self.vertices.last_handle()
    }
    fn last_face_handle(&self) -> Option<FaceHandle> {
        self.faces.last_handle()
    }

    fn contains_vertex(&self, vertex: VertexHandle) -> bool {
        self.vertices.contains_handle(vertex)
    }

    fn num_faces(&self) -> hsize {
        self.faces.num_elements()
    }

    fn contains_face(&self, face: FaceHandle) -> bool {
        self.faces.contains_handle(face)
    }

    fn num_edges(&self) -> hsize
    where
        Self: EdgeMesh
    {
        unreachable!()
    }

    fn next_edge_handle_from(&self, _: EdgeHandle) -> Option<EdgeHandle>
    where
        Self: EdgeMesh
    {
        unreachable!()
    }

    fn last_edge_handle(&self) -> Option<EdgeHandle>
    where
        Self: EdgeMesh
    {
        unreachable!()
    }

    fn check_integrity(&self) {
        for (f, &[va, vb, vc]) in self.faces.iter() {
            assert!(self.vertices.contains_handle(va), "va = {:?} of faces {:?}", va, f);
            assert!(self.vertices.contains_handle(vb), "vb = {:?} of faces {:?}", vb, f);
            assert!(self.vertices.contains_handle(vc), "vc = {:?} of faces {:?}", vc, f);

            if va == vb || va == vc || vb == vc {
                panic!("bug: vertices of face {:?} are not unique: {:?}", f, [va, vb, vc]);
            }
        }
    }
}

impl MeshMut for SharedVertexMesh {
    fn add_vertex(&mut self) -> VertexHandle {
        self.vertices.push(())
    }

    fn add_triangle(&mut self, [va, vb, vc]: [VertexHandle; 3]) -> FaceHandle {
        assert!(self.vertices.contains_handle(va));
        assert!(self.vertices.contains_handle(vb));
        assert!(self.vertices.contains_handle(vc));
        assert_ne!(va, vb, "vertices of new face are not unique");
        assert_ne!(va, vc, "vertices of new face are not unique");

        self.faces.push([va, vb, vc])
    }

    fn remove_face(&mut self, face: FaceHandle) {
        self.faces.remove(face);
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

    fn split_face(&mut self, f: FaceHandle) -> VertexHandle {
        let [va, vb, vc] = self.faces[f];
        let center = self.add_vertex();
        self.faces[f] = [va, vb, center];
        self.faces.push([vb, vc, center]);
        self.faces.push([vc, va, center]);

        center
    }

    fn add_face(&mut self, _: &[VertexHandle]) -> FaceHandle
    where
        Self: PolyMesh
    {
        unreachable!()
    }

    fn flip_edge(&mut self, _: EdgeHandle)
    where
        Self: EdgeMesh + TriMesh
    {
        unreachable!()
    }

    fn split_edge_with_faces(&mut self, _: EdgeHandle) -> SplitEdgeWithFacesResult
    where
        Self: EdgeMesh + TriMesh
    {
        unreachable!()
    }
}


impl BasicAdj for SharedVertexMesh {
    fn vertices_around_triangle(&self, face: FaceHandle) -> [VertexHandle; 3] {
        self.faces[face]
    }

    type VerticesAroundFaceIterFamily = FaceToVertexIterFam;

    fn vertices_around_face(&self, face: FaceHandle)
        -> <Self::VerticesAroundFaceIterFamily as HandleIterFamily<'_, VertexHandle>>::Iter
    {
        self.vertices_around_triangle(face).owned_iter()
    }
}

#[allow(missing_debug_implementations)]
pub struct FaceToVertexIterFam(!);
impl<'a> HandleIterFamily<'a, VertexHandle> for FaceToVertexIterFam {
    type Iter = TriArrayIntoIter<VertexHandle>;
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

    gen_mesh_tests!(SharedVertexMesh: [TriMesh, BasicAdj, SupportsMultiBlade]);
}
