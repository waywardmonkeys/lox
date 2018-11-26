#![feature(proc_macro_hygiene, never_type)]
#![allow(unused_imports)]

use cgmath::{Point3, Vector3};
use failure::Error;

use lox::{
    MeshWithProps,
    ds::SharedVertexMesh,
    io::{stl, ply},
    map::{ConstMap, FnMap},
    mesh,
    prelude::*,
    shape,
};


fn main() -> Result<(), Error> {
    let reader = stl::Reader::open(std::env::args().nth(1).unwrap())?;
    let reader = reader.map_vertex(|info| info.position);
    let reader = reader.map_face(|info| info.normal.x);
    let res = MeshWithProps::<SharedVertexMesh, _, _>::build_from(reader).unwrap();

    println!("{:#?}", res);

    Ok(())
}
