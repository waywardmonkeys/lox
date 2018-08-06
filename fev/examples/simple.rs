#![feature(rust_2018_preview)]
#![allow(unused_imports)] // TODO: remove later

extern crate fev;
extern crate failure;

use std::{
    env,
    fs,
    io,
};

use failure::Error;


use fev::{
    AdhocBuilder, TriMeshSource,
    handle::{VertexHandle, FaceHandle},
    impls::sv::SharedVertexMesh,
    prop::{HasNormal, HasPosition, LabeledPropList, PropLabel, FromProp, Pos3Like, Vec3Like},
    map::{PropMap, FaceVecMap, VertexVecMap, PropStoreMut, fn_map::FnMap},
    io::{
        MeshWriter,
        ser::{DataType, PropListSerialize, Serializer, Serialize, SingleProp},
        ply::{
            Format as PlyFormat,
            PlyWriter,
        },
        stl::{
            Format as StlFormat,
            StlWriter,
            StlReader,
        },
    },
};

#[derive(Copy, Clone, Debug)]
struct MyProp {
    pos: (f32, f32, f32),
}

impl HasPosition for MyProp {
    type Position = (f32, f32, f32);
    fn position(&self) -> &Self::Position {
        &self.pos
    }
}

impl PropListSerialize for MyProp {
    fn data_type_of(prop_index: usize) -> DataType {
        match prop_index {
            0 => <(f32, f32, f32) as Serialize>::DATA_TYPE,
            _ => unreachable!(),
        }
    }

    fn serialize_at<S: Serializer>(
        &self,
        prop_index: usize,
        serializer: S,
    ) -> Result<(), S::Error> {
        match prop_index {
            0 => self.pos.serialize(serializer),
            _ => unreachable!(),
        }
    }
}

impl LabeledPropList for MyProp {
    fn num_props() -> usize {
        1
    }

    fn label_of(prop_index: usize) -> PropLabel {
        match prop_index {
            0 => PropLabel::Position,
            _ => unreachable!(),
        }
    }
}

impl<T: HasPosition> FromProp<T> for MyProp {
    fn from_prop(src: T) -> Self {
        Self {
            pos: src.position().convert(),
        }
    }
}

#[derive(Debug)]
struct MyNormal {
    normal: [f32; 3],
}

impl HasNormal for MyNormal {
    type Normal = [f32; 3];
    fn normal(&self) -> &Self::Normal {
        &self.normal
    }
}

impl PropListSerialize for MyNormal {
    fn data_type_of(prop_index: usize) -> DataType {
        match prop_index {
            0 => <[f32; 3] as Serialize>::DATA_TYPE,
            _ => unreachable!(),
        }
    }

    fn serialize_at<S: Serializer>(
        &self,
        prop_index: usize,
        serializer: S,
    ) -> Result<(), S::Error> {
        match prop_index {
            0 => self.normal.serialize(serializer),
            _ => unreachable!(),
        }
    }
}

impl LabeledPropList for MyNormal {
    fn num_props() -> usize {
        1
    }

    fn label_of(prop_index: usize) -> PropLabel {
        match prop_index {
            0 => PropLabel::Normal,
            _ => unreachable!(),
        }
    }
}

impl<T: HasNormal> FromProp<T> for MyNormal {
    fn from_prop(src: T) -> Self {
        Self {
            normal: src.normal().convert(),
        }
    }
}

fn main() -> Result<(), Error> {
    let mut vm = VertexVecMap::new();
    let mut face_normals = FaceVecMap::new();

    let mut mesh = SharedVertexMesh::new();
    let a = mesh.add_vertex(MyProp { pos: (1.0, 2.0, 3.0) });
    vm.insert(a, MyNormal { normal: [1.0, 0.0, 0.0]});
    let b = mesh.add_vertex(MyProp { pos: (3.0, 2.0, 1.0) });
    vm.insert(b, MyNormal { normal: [0.0, 1.0, 0.0]});
    let c = mesh.add_vertex(MyProp { pos: (4.0, 4.0, 4.0) });
    vm.insert(c, MyNormal { normal: [0.0, 0.0, 1.0]});
    let f = mesh.add_face([a, b, c], ());
    face_normals.insert(f, MyNormal { normal: [1.0, 0.0, 0.0]});

    let filename = env::args().nth(1).unwrap();
    // StlReader::open(filename)?.append(&mut AdhocBuilder {
    //     add_vertex: |v| {
    //         println!("{:?}", v);
    //         VertexHandle(0)
    //     },
    //     add_face: |verts, f| println!("{:?} -> {:?}", verts, f),
    // }).unwrap();

    let mesh: SharedVertexMesh<MyProp, MyNormal> = StlReader::open(filename)?.build().unwrap();

    // println!("{:#?}", mesh);
    StlWriter::tmp_new(StlFormat::Binary, &mesh)?
        .write_to_file("mine.stl")?;

    Ok(())
}
