//! This module contains macros to generate unit tests for mesh data
//! structures.

/// Creates a hashset from all elements passed to this macro.
macro_rules! set {
    ($($item:expr),* $(,)*) => {
        vec![$($item),*].into_iter().collect::<::std::collections::HashSet<_>>()
    }
}

/// Takes an iterator and a list of elements. Collects both into sets and
/// compares those sets for equality via `assert_eq`.
macro_rules! assert_eq_set {
    ($iter:expr, [$($item:expr),* $(,)*]) => {
        assert_eq!(
            $iter.collect::<::std::collections::HashSet<_>>(),
            set!($($item),*)
        );
    }
}

/// Takes something slice-like and a list of elements. Asserts that the
/// elements occur in the slice exactly in the specified order (but potentially
/// shifted).
///
/// So `assert_eq_order(v, [a, b, c])` would be fine if `v` is any of this:
/// - `[a, b, c]`
/// - `[b, c, a]`
/// - `[c, a, b]`
///
/// This check is a stricter version of `assert_eq_set`. For <= 2 elements,
/// it's more or less equivalent to `assert_eq_set`.
macro_rules! assert_eq_order {
    ($list:expr, []) => {{
        assert_eq!($list, []);
    }};
    ($list:expr, [$a:expr $(, $tail:expr)*]) => {{
        let actual = $list;
        let expected = [$a $(, $tail)*];

        if actual.len() != expected.len() {
            panic!(
                "assert_eq_order failed (length mismatch): \n  \
                    left: `{:?}` (`{}`),\n \
                    right: `{:?}`",
                actual,
                stringify!($list),
                expected,
            );
        }

        let pos = actual.iter().position(|&e| e == $a).expect(concat!(
            "assert_eq_order failed: ",
            stringify!($a),
            " not found in ",
            stringify!($list),
            " (expected `[",
            stringify!($a),
            $(", ", stringify!($tail),)*
            "]`)",
        ));

        let mut rotated = expected;
        rotated.rotate_right(pos);


        if actual != rotated {
            panic!(
                "assert_eq_order failed: \n  \
                    left: `{:?}` (`{}`),\n \
                    right: `{:?}` (original `{:?}`)",
                actual,
                stringify!($list),
                rotated,
                expected,
            );
        }
    }};
}

/// Generates unit tests for the mesh data structure `$name`.
///
/// In the brackets, you should specify additional traits that are implemented
/// for the mesh type. These will generate additional asserts in the tests. The
/// following traits are assumed to be implemented by every mesh type this
/// macro is invoked with:
/// - `TriMesh`
/// - `TriMeshMut`
///
/// These traits need to be specified in the brackets and will generate
/// additional asserts:
/// - `TriVerticesOfFace`
/// - `FacesAroundVertex`
/// - `VerticesAroundVertex`
macro_rules! gen_tri_mesh_tests {
    ($name:ident : [$($extra:ident),*]) => {
        $(
            gen_tri_mesh_tests!(@is_valid_extra_trait $extra);
        )*

        #[allow(unused_imports)]
        use crate::{
            prelude::*,
            handle::{Handle, HandleId},
        };

        #[test]
        fn empty() {
            let m = $name::empty();

            assert_eq!(m.num_faces(), 0);
            assert_eq!(m.num_vertices(), 0);

            assert!(m.faces().next().is_none());
            assert!(m.vertices().next().is_none());

            assert!(!m.contains_vertex(VertexHandle::from_id(0)));
            assert!(!m.contains_vertex(VertexHandle::from_id(27)));
            assert!(!m.contains_face(FaceHandle::from_id(0)));
            assert!(!m.contains_face(FaceHandle::from_id(27)));
        }

        #[test]
        fn single_vertex() {
            let mut m = $name::empty();
            let v = m.add_vertex();

            assert_eq!(m.num_faces(), 0);
            assert_eq!(m.num_vertices(), 1);

            assert!(m.faces().next().is_none());
            assert_eq_set!(m.vertices().map(|x| x.handle()), [v]);

            assert!(m.contains_vertex(v));
            assert!(!m.contains_vertex(VertexHandle::from_id(v.id().next())));

            gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.faces_around_vertex(v).into_vec(), []);
            });

            gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.vertices_around_vertex(v).into_vec(), []);
            });
        }

        #[test]
        fn single_triangle() {
            //
            //         (C)
            //        /   \
            //       /     \
            //      /       \
            //    (A) ----- (B)
            //
            let mut m = $name::empty();
            let va = m.add_vertex();
            let vb = m.add_vertex();
            let vc = m.add_vertex();
            let f = m.add_face([va, vb, vc]);

            assert_eq!(m.num_faces(), 1);
            assert_eq!(m.num_vertices(), 3);

            assert_eq_set!(m.faces().map(|x| x.handle()), [f]);
            assert_eq_set!(m.vertices().map(|x| x.handle()), [va, vb, vc]);

            assert!(m.contains_vertex(va));
            assert!(m.contains_vertex(vb));
            assert!(m.contains_vertex(vc));
            assert!(m.contains_face(f));
            assert!(!m.contains_face(FaceHandle::from_id(f.id().next())));

            gen_tri_mesh_tests!(@if TriVerticesOfFace in [$($extra),*] => {
                assert_eq_order!(m.vertices_of_face(f), [va, vb, vc]);
            });

            gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.faces_around_vertex(va).into_vec(), [f]);
                assert_eq_order!(m.faces_around_vertex(vb).into_vec(), [f]);
                assert_eq_order!(m.faces_around_vertex(vc).into_vec(), [f]);
            });

            gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.vertices_around_vertex(va).into_vec(), [vc, vb]);
                assert_eq_order!(m.vertices_around_vertex(vb).into_vec(), [va, vc]);
                assert_eq_order!(m.vertices_around_vertex(vc).into_vec(), [vb, va]);
            });
        }

        #[test]
        fn tetrahedron() {
            //
            //             (T)
            //            / | \
            //           /  |  \
            //          /   |   \
            //         /   (C)   \
            //        / ⋰     ⋱  \
            //       (A) ------- (B)
            //
            let mut m = $name::empty();
            let va = m.add_vertex();
            let vb = m.add_vertex();
            let vc = m.add_vertex();
            let v_top = m.add_vertex();

            let f_bottom = m.add_face([va, vc, vb]);
            let f_ab = m.add_face([va, vb, v_top]);
            let f_bc = m.add_face([vb, vc, v_top]);
            let f_ca = m.add_face([vc, va, v_top]);

            assert_eq!(m.num_faces(), 4);
            assert_eq!(m.num_vertices(), 4);

            assert_eq_set!(m.faces().map(|x| x.handle()), [f_bottom, f_ab, f_bc, f_ca]);
            assert_eq_set!(m.vertices().map(|x| x.handle()), [va, vb, vc, v_top]);

            gen_tri_mesh_tests!(@if TriVerticesOfFace in [$($extra),*] => {
                assert_eq_order!(m.vertices_of_face(f_bottom), [va, vc, vb]);
                assert_eq_order!(m.vertices_of_face(f_ab), [va, vb, v_top]);
                assert_eq_order!(m.vertices_of_face(f_bc), [vb, vc, v_top]);
                assert_eq_order!(m.vertices_of_face(f_ca), [vc, va, v_top]);
            });

            gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.faces_around_vertex(va).into_vec(), [f_bottom, f_ca, f_ab]);
                assert_eq_order!(m.faces_around_vertex(vb).into_vec(), [f_bottom, f_ab, f_bc]);
                assert_eq_order!(m.faces_around_vertex(vc).into_vec(), [f_bottom, f_bc, f_ca]);
                assert_eq_order!(m.faces_around_vertex(v_top).into_vec(), [f_ca, f_bc, f_ab]);
            });

            gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.vertices_around_vertex(va).into_vec(), [v_top, vb, vc]);
                assert_eq_order!(m.vertices_around_vertex(vb).into_vec(), [v_top, vc, va]);
                assert_eq_order!(m.vertices_around_vertex(vc).into_vec(), [v_top, va, vb]);
                assert_eq_order!(m.vertices_around_vertex(v_top).into_vec(), [va, vc, vb]);
            });
        }

        #[test]
        fn triangle_strip_build() {
            //
            //    (A)---(D)
            //     | \ Y | \
            //     |  \  |  \
            //     | X \ | Z \
            //     |    \|    \
            //    (B)---(C)---(E)
            //
            let mut m = $name::empty();
            let va = m.add_vertex();
            let vb = m.add_vertex();
            let vc = m.add_vertex();
            let fx = m.add_face([va, vb, vc]);
            // Everything is correct now, this is checked by `single_triangle`

            // ----- Add second face
            let vd = m.add_vertex();
            let fy = m.add_face([va, vc, vd]);

            assert_eq!(m.num_faces(), 2);
            assert_eq!(m.num_vertices(), 4);

            assert_eq_set!(m.faces().map(|x| x.handle()), [fx, fy]);
            assert_eq_set!(m.vertices().map(|x| x.handle()), [va, vb, vc, vd]);

            gen_tri_mesh_tests!(@if TriVerticesOfFace in [$($extra),*] => {
                assert_eq_order!(m.vertices_of_face(fx), [va, vb, vc]);
                assert_eq_order!(m.vertices_of_face(fy), [va, vc, vd]);
            });

            gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.faces_around_vertex(va).into_vec(), [fy, fx]);
                assert_eq_order!(m.faces_around_vertex(vb).into_vec(), [fx]);
                assert_eq_order!(m.faces_around_vertex(vc).into_vec(), [fx, fy]);
                assert_eq_order!(m.faces_around_vertex(vd).into_vec(), [fy]);
            });

            gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.vertices_around_vertex(va).into_vec(), [vd, vc, vb]);
                assert_eq_order!(m.vertices_around_vertex(vb).into_vec(), [va, vc]);
                assert_eq_order!(m.vertices_around_vertex(vc).into_vec(), [vb, va, vd]);
                assert_eq_order!(m.vertices_around_vertex(vd).into_vec(), [vc, va]);
            });

            // ----- Add third face
            let ve = m.add_vertex();
            let fz = m.add_face([vd, vc, ve]);

            assert_eq!(m.num_faces(), 3);
            assert_eq!(m.num_vertices(), 5);

            assert_eq_set!(m.faces().map(|x| x.handle()), [fx, fy, fz]);
            assert_eq_set!(m.vertices().map(|x| x.handle()), [va, vb, vc, vd, ve]);

            gen_tri_mesh_tests!(@if TriVerticesOfFace in [$($extra),*] => {
                assert_eq_order!(m.vertices_of_face(fx), [va, vb, vc]);
                assert_eq_order!(m.vertices_of_face(fy), [va, vc, vd]);
                assert_eq_order!(m.vertices_of_face(fz), [vd, vc, ve]);
            });

            gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.faces_around_vertex(va).into_vec(), [fy, fx]);
                assert_eq_order!(m.faces_around_vertex(vb).into_vec(), [fx]);
                assert_eq_order!(m.faces_around_vertex(vc).into_vec(), [fx, fy, fz]);
                assert_eq_order!(m.faces_around_vertex(vd).into_vec(), [fz, fy]);
                assert_eq_order!(m.faces_around_vertex(ve).into_vec(), [fz]);
            });

            gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.vertices_around_vertex(va).into_vec(), [vd, vc, vb]);
                assert_eq_order!(m.vertices_around_vertex(vb).into_vec(), [va, vc]);
                assert_eq_order!(m.vertices_around_vertex(vc).into_vec(), [vb, va, vd, ve]);
                assert_eq_order!(m.vertices_around_vertex(vd).into_vec(), [ve, vc, va]);
                assert_eq_order!(m.vertices_around_vertex(ve).into_vec(), [vc, vd]);
            });
        }

        #[test]
        fn simple_2d_hole() {
            // There are only six faces. The triangle in the middle is empty.
            //
            //                       (a)
            //                      / | \
            //     (b--c)          /  |  \        (b--e)
            //   u: [a, c, b]     /  (b)  \     w: [a, b, e]
            //   v: [b, c, d]    /  /   \  \    x: [b, f, e]
            //                  /  /     \  \
            //                 /  /       \  \
            //                /  (d)-----(f)  \
            //               / ⟋            ⟍ \
            //              (c)_______________(e)
            //
            //                    (c--f)
            //                  y: [c, f, d]
            //                  z: [c, e, f]
            //

            let mut m = $name::empty();
            let va = m.add_vertex();
            let vb = m.add_vertex();
            let vc = m.add_vertex();
            let vd = m.add_vertex();
            let ve = m.add_vertex();
            let vf = m.add_vertex();

            let fu = m.add_face([va, vc, vb]);
            let fv = m.add_face([vb, vc, vd]);

            let fw = m.add_face([va, vb, ve]);
            let fx = m.add_face([vb, vf, ve]);

            let fy = m.add_face([vc, vf, vd]);
            let fz = m.add_face([vc, ve, vf]);


            // ----- Check stuff
            assert_eq!(m.num_faces(), 6);
            assert_eq!(m.num_vertices(), 6);

            assert_eq_set!(m.faces().map(|x| x.handle()), [fu, fv, fw, fx, fy, fz]);
            assert_eq_set!(m.vertices().map(|x| x.handle()), [va, vb, vc, vd, ve, vf]);

            gen_tri_mesh_tests!(@if TriVerticesOfFace in [$($extra),*] => {
                assert_eq_order!(m.vertices_of_face(fu), [va, vc, vb]);
                assert_eq_order!(m.vertices_of_face(fv), [vb, vc, vd]);
                assert_eq_order!(m.vertices_of_face(fw), [va, vb, ve]);
                assert_eq_order!(m.vertices_of_face(fx), [vb, vf, ve]);
                assert_eq_order!(m.vertices_of_face(fy), [vc, vf, vd]);
                assert_eq_order!(m.vertices_of_face(fz), [vc, ve, vf]);
            });

            gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.faces_around_vertex(va).into_vec(), [fu, fw]);
                assert_eq_order!(m.faces_around_vertex(vb).into_vec(), [fw, fx, fv, fu]);
                assert_eq_order!(m.faces_around_vertex(vc).into_vec(), [fu, fv, fy, fz]);
                assert_eq_order!(m.faces_around_vertex(vd).into_vec(), [fv, fy]);
                assert_eq_order!(m.faces_around_vertex(ve).into_vec(), [fz, fx, fw]);
                assert_eq_order!(m.faces_around_vertex(vf).into_vec(), [fx, fz, fy]);
            });

            gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.vertices_around_vertex(va).into_vec(), [ve, vb, vc]);
                assert_eq_order!(m.vertices_around_vertex(vb).into_vec(), [va, ve, vf, vd, vc]);
                assert_eq_order!(m.vertices_around_vertex(vc).into_vec(), [va, vb, vd, vf, ve]);
                assert_eq_order!(m.vertices_around_vertex(vd).into_vec(), [vb, vf, vc]);
                assert_eq_order!(m.vertices_around_vertex(ve).into_vec(), [vc, vf, vb, va]);
                assert_eq_order!(m.vertices_around_vertex(vf).into_vec(), [vb, ve, vc, vd]);
            });
        }

        // TODO: maybe make this optional? Dunno if all mesh DS needs to
        // support this.
        #[test]
        fn vertex_with_two_blades() {
            //
            //      (b)-------(c)
            //        \       /
            //         \  X  /
            //          \   /
            //           \ /
            //           (a)
            //           / \
            //          /   \
            //         /  Y  \
            //        /       \
            //      (d)-------(e)
            //

            let mut m = $name::empty();
            let va = m.add_vertex();
            let vb = m.add_vertex();
            let vc = m.add_vertex();
            let vd = m.add_vertex();
            let ve = m.add_vertex();

            let fx = m.add_face([va, vc, vb]);
            let fy = m.add_face([va, vd, ve]);


            // ----- Check stuff
            assert_eq!(m.num_faces(), 2);
            assert_eq!(m.num_vertices(), 5);

            assert_eq_set!(m.faces().map(|x| x.handle()), [fx, fy]);
            assert_eq_set!(m.vertices().map(|x| x.handle()), [va, vb, vc, vd, ve]);

            gen_tri_mesh_tests!(@if TriVerticesOfFace in [$($extra),*] => {
                assert_eq_order!(m.vertices_of_face(fx), [va, vc, vb]);
                assert_eq_order!(m.vertices_of_face(fy), [va, vd, ve]);
            });

            gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.faces_around_vertex(va).into_vec(), [fx, fy]);
                assert_eq_order!(m.faces_around_vertex(vb).into_vec(), [fx]);
                assert_eq_order!(m.faces_around_vertex(vc).into_vec(), [fx]);
                assert_eq_order!(m.faces_around_vertex(vd).into_vec(), [fy]);
                assert_eq_order!(m.faces_around_vertex(ve).into_vec(), [fy]);
            });

            gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                assert_eq_order!(m.vertices_around_vertex(va).into_vec(), [vb, vc, ve, vd]);
                assert_eq_order!(m.vertices_around_vertex(vb).into_vec(), [vc, va]);
                assert_eq_order!(m.vertices_around_vertex(vc).into_vec(), [va, vb]);
                assert_eq_order!(m.vertices_around_vertex(vd).into_vec(), [va, ve]);
                assert_eq_order!(m.vertices_around_vertex(ve).into_vec(), [vd, va]);
            });
        }

        #[test]
        fn vertex_with_three_blades() {
            //
            //       (b)-------(c)
            //         \       /
            //          \  X  /
            //           \   /
            //            \ /
            //  (g)-------(a)-------(d)
            //    \       / \       /
            //     \  Z  /   \  Y  /
            //      \   /     \   /
            //       \ /       \ /
            //       (f)       (e)
            //

            let mut m = $name::empty();
            let va = m.add_vertex();
            let vb = m.add_vertex();
            let vc = m.add_vertex();
            let vd = m.add_vertex();
            let ve = m.add_vertex();
            let vf = m.add_vertex();
            let vg = m.add_vertex();

            let fx = m.add_face([va, vc, vb]);
            let fy = m.add_face([va, ve, vd]);
            let fz = m.add_face([va, vg, vf]);


            // ----- Check stuff
            assert_eq!(m.num_faces(), 3);
            assert_eq!(m.num_vertices(), 7);

            assert_eq_set!(m.faces().map(|x| x.handle()), [fx, fy, fz]);
            assert_eq_set!(m.vertices().map(|x| x.handle()), [va, vb, vc, vd, ve, vf, vg]);

            gen_tri_mesh_tests!(@if TriVerticesOfFace in [$($extra),*] => {
                assert_eq_order!(m.vertices_of_face(fx), [va, vc, vb]);
                assert_eq_order!(m.vertices_of_face(fy), [va, ve, vd]);
                assert_eq_order!(m.vertices_of_face(fz), [va, vg, vf]);
            });

            gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                // We can't assume any order for the faces around (a).
                assert_eq_set!(m.faces_around_vertex(va), [fx, fy, fz]);

                assert_eq_order!(m.faces_around_vertex(vb).into_vec(), [fx]);
                assert_eq_order!(m.faces_around_vertex(vc).into_vec(), [fx]);
                assert_eq_order!(m.faces_around_vertex(vd).into_vec(), [fy]);
                assert_eq_order!(m.faces_around_vertex(ve).into_vec(), [fy]);
                assert_eq_order!(m.faces_around_vertex(vf).into_vec(), [fz]);
                assert_eq_order!(m.faces_around_vertex(vg).into_vec(), [fz]);
            });

            gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                assert_eq_set!(m.vertices_around_vertex(va), [vb, vc, vd, ve, vf, vg]);

                assert_eq_order!(m.vertices_around_vertex(vb).into_vec(), [vc, va]);
                assert_eq_order!(m.vertices_around_vertex(vc).into_vec(), [va, vb]);
                assert_eq_order!(m.vertices_around_vertex(vd).into_vec(), [va, ve]);
                assert_eq_order!(m.vertices_around_vertex(ve).into_vec(), [vd, va]);
                assert_eq_order!(m.vertices_around_vertex(vf).into_vec(), [va, vg]);
                assert_eq_order!(m.vertices_around_vertex(vg).into_vec(), [vf, va]);
            });
        }

        #[test]
        fn connect_two_blades_around_vertex() {
            // We start with the same mesh as in `vertex_with_three_blades` and
            // will then add a face in two different ways.
            //
            // Since the order of the three blades is not defined, we should be
            // able to connect any two blades. All these faces should be valid
            // insertions:
            // - [d, c, a]
            // - [f, c, a]
            // - [b, e, a]
            // - [b, g, a]
            // - [a, d, g]
            // - [e, a, f]
            //
            // We only check everything for the first two, but still try to
            // insert the last four just to check that nothing panics.
            //
            //       (b)-------(c)
            //         \       /
            //          \  X  /
            //           \   /
            //            \ /
            //  (g)-------(a)-------(d)
            //    \       / \       /
            //     \  Z  /   \  Y  /
            //      \   /     \   /
            //       \ /       \ /
            //       (f)       (e)
            //

            let mut m = $name::empty();
            let va = m.add_vertex();
            let vb = m.add_vertex();
            let vc = m.add_vertex();
            let vd = m.add_vertex();
            let ve = m.add_vertex();
            let vf = m.add_vertex();
            let vg = m.add_vertex();

            let fx = m.add_face([va, vc, vb]);
            let fy = m.add_face([va, ve, vd]);
            let fz = m.add_face([va, vg, vf]);


            // Insert [d, c, a]
            {
                let mut m = m.clone();
                let f = m.add_face([vd, vc, va]);

                // ----- Check stuff
                assert_eq!(m.num_faces(), 4);
                assert_eq!(m.num_vertices(), 7);

                assert_eq_set!(m.faces().map(|x| x.handle()), [fx, fy, fz, f]);
                assert_eq_set!(m.vertices().map(|x| x.handle()), [va, vb, vc, vd, ve, vf, vg]);

                gen_tri_mesh_tests!(@if TriVerticesOfFace in [$($extra),*] => {
                    assert_eq_order!(m.vertices_of_face(fx), [va, vc, vb]);
                    assert_eq_order!(m.vertices_of_face(fy), [va, ve, vd]);
                    assert_eq_order!(m.vertices_of_face(fz), [va, vg, vf]);
                    assert_eq_order!(m.vertices_of_face(f), [vd, vc, va]);
                });

                gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                    // Since we have only two blades again, we can assume order
                    assert_eq_order!(m.faces_around_vertex(va).into_vec(), [fx, f, fy, fz]);

                    assert_eq_order!(m.faces_around_vertex(vb).into_vec(), [fx]);
                    assert_eq_order!(m.faces_around_vertex(vc).into_vec(), [fx, f]);
                    assert_eq_order!(m.faces_around_vertex(vd).into_vec(), [fy, f]);
                    assert_eq_order!(m.faces_around_vertex(ve).into_vec(), [fy]);
                    assert_eq_order!(m.faces_around_vertex(vf).into_vec(), [fz]);
                    assert_eq_order!(m.faces_around_vertex(vg).into_vec(), [fz]);
                });

                gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                    assert_eq_order!(
                        m.vertices_around_vertex(va).into_vec(),
                        [vb, vc, vd, ve, vf, vg]
                    );
                    assert_eq_order!(m.vertices_around_vertex(vb).into_vec(), [vc, va]);
                    assert_eq_order!(m.vertices_around_vertex(vc).into_vec(), [vd, va, vb]);
                    assert_eq_order!(m.vertices_around_vertex(vd).into_vec(), [ve, va, vc]);
                    assert_eq_order!(m.vertices_around_vertex(ve).into_vec(), [vd, va]);
                    assert_eq_order!(m.vertices_around_vertex(vf).into_vec(), [va, vg]);
                    assert_eq_order!(m.vertices_around_vertex(vg).into_vec(), [vf, va]);
                });
            }

            // Insert [f, c, a]
            {
                let mut m = m.clone();
                let f = m.add_face([vf, vc, va]);

                // ----- Check stuff
                assert_eq!(m.num_faces(), 4);
                assert_eq!(m.num_vertices(), 7);

                assert_eq_set!(m.faces().map(|x| x.handle()), [fx, fy, fz, f]);
                assert_eq_set!(m.vertices().map(|x| x.handle()), [va, vb, vc, vd, ve, vf, vg]);

                gen_tri_mesh_tests!(@if TriVerticesOfFace in [$($extra),*] => {
                    assert_eq_order!(m.vertices_of_face(fx), [va, vc, vb]);
                    assert_eq_order!(m.vertices_of_face(fy), [va, ve, vd]);
                    assert_eq_order!(m.vertices_of_face(fz), [va, vg, vf]);
                    assert_eq_order!(m.vertices_of_face(f), [vf, vc, va]);
                });

                gen_tri_mesh_tests!(@if FacesAroundVertex in [$($extra),*] => {
                    // Since we have only two blades again, we can assume order
                    assert_eq_order!(m.faces_around_vertex(va).into_vec(), [fx, f, fz, fy]);

                    assert_eq_order!(m.faces_around_vertex(vb).into_vec(), [fx]);
                    assert_eq_order!(m.faces_around_vertex(vc).into_vec(), [fx, f]);
                    assert_eq_order!(m.faces_around_vertex(vd).into_vec(), [fy]);
                    assert_eq_order!(m.faces_around_vertex(ve).into_vec(), [fy]);
                    assert_eq_order!(m.faces_around_vertex(vf).into_vec(), [fz, f]);
                    assert_eq_order!(m.faces_around_vertex(vg).into_vec(), [fz]);
                });

                gen_tri_mesh_tests!(@if VerticesAroundVertex in [$($extra),*] => {
                    assert_eq_order!(
                        m.vertices_around_vertex(va).into_vec(),
                        [vb, vc, vf, vg, vd, ve]
                    );
                    assert_eq_order!(m.vertices_around_vertex(vb).into_vec(), [vc, va]);
                    assert_eq_order!(m.vertices_around_vertex(vc).into_vec(), [vf, va, vb]);
                    assert_eq_order!(m.vertices_around_vertex(vd).into_vec(), [ve, va]);
                    assert_eq_order!(m.vertices_around_vertex(ve).into_vec(), [vd, va]);
                    assert_eq_order!(m.vertices_around_vertex(vf).into_vec(), [vg, va, vc]);
                    assert_eq_order!(m.vertices_around_vertex(vg).into_vec(), [vf, va]);
                });
            }

            // Try to insert the remaning faces (just make sure it doesn't
            // panic).
            m.clone().add_face([vb, ve, va]);
            m.clone().add_face([vb, vg, va]);
            m.clone().add_face([va, vd, vg]);
            m.clone().add_face([ve, va, vf]);
        }

        gen_tri_mesh_tests!(@if_item Manifold in [$($extra),*] => {
            #[test]
            #[should_panic]
            fn non_manifold_triple_edge() {
                // This creates a non-manifold mesh by connecting three faces to a
                // single edge. This is never allowed by mesh data structures. So
                // we expect them to panic.
                //
                //
                //            (a)⟍
                //           / | \ ⟍
                //          /  |  \  ⟍
                //         /   |   \   ⟍
                //       (c)   |   (d)  (e)
                //         \   |   /   ⟋
                //          \  |  /  ⟋
                //           \ | / ⟋
                //            (b)⟋
                //
                let mut m = $name::empty();
                let va = m.add_vertex();
                let vb = m.add_vertex();
                let vc = m.add_vertex();
                let vd = m.add_vertex();
                let ve = m.add_vertex();

                m.add_face([va, vc, vb]);
                m.add_face([va, vb, vd]);

                // This should panic
                m.add_face([va, vb, ve]);
            }

            #[test]
            #[should_panic]
            fn non_manifold_add_to_closed_fan() {
                // This creates a non-manifold mesh by first creating a vertex
                // (A) that has a closed fan around itself. Then we try to add
                // another face to that vertex (the one with E and F).
                //
                //             (B)          (E)
                //            / | \       ⟋  |
                //           /  |  \    ⟋    |
                //          /  (A)  \  A -- (F)
                //         / ⟋    ⟍ \
                //        (C) ----- (D)
                //
                let mut m = $name::empty();
                let va = m.add_vertex();
                let vb = m.add_vertex();
                let vc = m.add_vertex();
                let vd = m.add_vertex();
                let ve = m.add_vertex();
                let vf = m.add_vertex();

                m.add_face([va, vb, vc]);
                m.add_face([va, vc, vd]);
                m.add_face([va, vd, vb]);

                // This should panic
                m.add_face([va, vf, ve]);
            }
        });

        // TODO: Double Sided triangle
        // TODO: Möbius strip
    };

    // These two arms are used to conditionally expand to a given body.
    //
    // If the first ident ($needle) is in list following it, these arms expand
    // to `$body`, otherwise they expand to an empty expression.
    (@if $needle:ident in [] => $body:tt) => {{
        // The needle was not found in the extra traits. To make sure there
        // wasn't a typo bug in this macro definition, we check that `$needle`
        // is a valid extra trait to begin with. We know that all idents in the
        // list are valid, because we checked it above.
        gen_tri_mesh_tests!(@is_valid_extra_trait $needle);
    }};
    (@if $needle:ident in [$head:ident $(, $tail:ident)*] => $body:tt) => {{
        macro_rules! __inner_helper {
            ($needle $needle) => { $body };
            ($needle $head) => { gen_tri_mesh_tests!(@if $needle in [$($tail),*] => $body) }
        };

        __inner_helper!($needle $head)
    }};

    // This is the same as above but for bodies which expand to items (instead
    // of expressions).
    (@if_item $needle:ident in [] => { $($body:tt)* }) => {
        // The needle was not found in the extra traits. To make sure there
        // wasn't a typo bug in this macro definition, we check that `$needle`
        // is a valid extra trait to begin with. We know that all idents in the
        // list are valid, because we checked it above.
        gen_tri_mesh_tests!(@is_valid_extra_trait $needle);
    };
    (@if_item $needle:ident in [$head:ident $(, $tail:ident)*] => { $($body:tt)* }) => {
        macro_rules! __inner_helper {
            ($needle $needle) => { $($body)* };
            ($needle $head) => {
                gen_tri_mesh_tests!(@if_item $needle in [$($tail),*] => { $($body)* });
            }
        }

        __inner_helper!($needle $head);
    };

    // These arms are used to make sure all traits passed into the macro
    // (include the ones used in the definition of the macro) are valid.
    // Otherwise it's too easy to make a typo.
    (@is_valid_extra_trait TriVerticesOfFace) => {};
    (@is_valid_extra_trait FacesAroundVertex) => {};
    (@is_valid_extra_trait VerticesAroundVertex) => {};
    (@is_valid_extra_trait Manifold) => {}; // this is not a real trait yet...
    (@is_valid_extra_trait $other:ident) => {
        compile_error!(concat!(
            "`",
            stringify!($other),
            "` is not a valid trait to pass to `gen_tri_mesh_tests`",
        ));
    };
}
