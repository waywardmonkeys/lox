//! Types and functions related to handles (e.g. face handle, vertex handle).
//!
//! A handle is some kind of identifier which allows you to retrieve the data
//! associated with that handle. It's a bit like the key in a hashmap. There
//! are different kinds of handles to refer to different data. For example, a
//! `FaceHandle` can be used to refer to a face.
//!
//! Note that different kinds of handles usually only exist to use strong
//! typing and are identical on machine level. This allows you to get a
//! compiler error when you pass a `VertexHandle` where a `FaceHandle` is
//! expected. Without strong handle types, it's easy to get very strange and
//! hard to debug runtime errors.
//!
//! # `hsize` and Arena Allocators
//!
//! A handle type is just a wrapper around a simple integer, or more precisely:
//! around the [`hsize`][handle::hsize] integer type. This integer is usually
//! used as an index to an array-like thing (like a `Vec`) -- that way, we can
//! refer to data.
//!
//! Note that handles share some properties with pointers (they refer to a
//! thing and are actually just an integer), but have some important
//! differences:
//!
//! - `hsize` (and thus handles) are always 32 bit wide, unlike pointers which
//!   might vary with target platform.
//! - While all pointers exist in "universe" (they all refer to the global
//!   memory of your PC), handles often refer to many different universes. For
//!   example, it's perfectly fine to have two `FaceHandle`s with the value 0
//!   that refer to different faces: one handle belongs to the "universe" of
//!   one mesh and the other to a different mesh. The user has to remember
//!   which universe (mesh) a handle belongs to.
//! - Pointers are usually handled as references in Rust. With that, the borrow
//!   checker makes sure we don't do bad things with those. With handles, all
//!   those bad things can still happen, e.g. referring to an already deleted
//!   thing.
//!
//! When looking closer at it, this system can look a lot like an "arena
//! allocator": it's a type that reserves a big chunk of memory at its creation
//! and serves as a memory allocator with the advantage that all its memory can
//! be freed at once. In many cases, simple integers are used to refer to data
//! within such an arena. There are some more "high level" allocators for this,
//! e.g. `slab`. The easiest allocator that works like that would be `Vec`: you
//! can add elements and can refer to them via `usize`.
//!
//! It's a common criticism that this way of allocating things is used to
//! sidestep the borrow checker and all the nice guarantees Rust gives us. And
//! as such, that this pattern should be discouraged. However, there are some
//! situations where it cannot be avoided or where this has very large
//! advantages.
//!
//! For example, in mesh processing, meshes with more than 2<sup>32</sup>
//! elements are extremely rare and most of the memory in a mesh data structure
//! is taken by references to other elements. So we can drastically reduce the
//! overall memory consumption (and due to caching: the execution time). It
//! would also be very hard to use this library if every element reference
//! would be a real reference. So it does make sense for this library to use
//! such a system.
//!
//! Of course, we would like to avoid annoying bugs due to errors like "use
//! after free". The crate `slotmap` has really great ideas regarding this.
//! This crate will try out some ideas to avoid some common mistakes in the
//! future.

use std::fmt;


/// The integer used in all handle types. See [the module documentation][self]
/// for more information on the general system behind handles.
///
/// This is called `hsize` because it is very similar to `usize` in many
/// regards. The *h* is for *handle*.
///
/// Since we can't be generic over the integer type right now (due to the lack
/// of GATs and a huge increase in API complexity), we have to choose a good
/// default. `u32` is fitting for most use cases.
///
/// Since the ID is always used to refer to some data, exhausting `u32` means
/// that we have more than 2<sup>32</sup> instances of that data. If one
/// instance is only 1 byte big, this results in 4GB memory usage. However, in
/// practice 1 byte is not enough to store anything useful for meshes. Usually,
/// you at least store some kind of position (e.g. `[f32; 3]` = 12 bytes) per
/// vertex plus the connectivity information, which is at something like 3
/// handles per face. So the useful minimum of stored information is:
///
/// - 12 bytes per vertex
/// - 12 bytes per face
///
/// From [here][1] we can see that in a typical triangular mesh, there are
/// around twice as many faces as vertices. The effective size per face is thus
/// around 18 bytes. To have more than 2<sup>32</sup> faces, the mesh would
/// occupy around 2<sup>32</sup> · 18 bytes = 72 GB of memory. In other data
/// structures which store more connectivity information, this would be even
/// more. There do exist rare situations (mostly in research) where one has to
/// deal with huge meshes of that size. But again, it's rather rare.
///
/// On the other side are use cases where a smaller ID type, like `u16` would
/// be sufficient. Here, one could save memory by using a smaller ID type.
/// Making `u16` the default ID type is not OK though: 2<sup>16</sup> = 65536
/// is not a huge number and there are many situations in which meshes have way
/// more than 65536 elements.
///
///
/// [1]: https://math.stackexchange.com/q/425968/340615
#[allow(non_camel_case_types)]
pub type hsize = u32;

/// Extension trait to add a few useful methods to `hsize`.
pub trait HSizeExt {
    /// Returns a new index.
    ///
    /// When the index space has been exhausted and there is no new index, this
    /// function either panics or returns an old index. In debug mode, this
    /// function is guaranteed to panic in this case.
    fn next(self) -> Self;
}

impl HSizeExt for hsize {
    #[inline(always)]
    fn next(self) -> Self {
        self + 1
    }
}


/// Types that can be used to refer to some data. See [the module
/// documentation][self] for more information on handles.
pub trait Handle: 'static + Copy + fmt::Debug + Eq + Ord {
    /// Create a handle from the given index. The index must not be
    /// `hsize::max_value()` as this value is reserved!
    fn new(idx: hsize) -> Self;

    /// Return the index of the current handle.
    fn idx(&self) -> hsize;

    /// Helper method to create a handle directly from an `usize`.
    ///
    /// If `raw` cannot be represented by `hsize`, this function either panics
    /// or returns a nonsensical ID. In debug mode, this function is guaranteed
    /// to panic in this case.
    #[inline(always)]
    fn from_usize(raw: usize) -> Self {
        // If `usize` is bigger than `u32`, we assert that the value is fine.
        #[cfg(target_pointer_width = "64")]
        debug_assert!(raw <= hsize::max_value() as usize);

        Self::new(raw as hsize)
    }

    /// Helper method to get the ID as a usize directly from an handle.
    ///
    /// If the index cannot be represented by `usize`, this function either
    /// panics or returns a nonsensical value. In debug mode, this function is
    /// guaranteed to panic in this case. Note however, that this usually won't
    /// happen, because `hsize` is in almost all cases smaller than or equal to
    /// `usize`.
    #[inline(always)]
    fn to_usize(&self) -> usize {
        // If `usize` is smaller than `u32`, we assert that the value is fine.
        #[cfg(any(target_pointer_width = "16", target_pointer_width = "8"))]
        debug_assert!(self.idx() <= usize::max_value() as hsize);

        self.idx() as usize
    }
}


macro_rules! make_handle_type {
    ($(#[$attr:meta])* $name:ident = $short:expr;) => {
        $(#[$attr])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(hsize);

        impl Handle for $name {
            #[inline(always)]
            fn new(id: hsize) -> Self {
                $name(id)
            }

            #[inline(always)]
            fn idx(&self) -> hsize {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", $short)?;
                self.idx().fmt(f)
            }
        }
    }
}

make_handle_type!{
    /// A handle that is associated with a face.
    FaceHandle = "F";
}
make_handle_type!{
    /// A handle that is associated with an edge.
    EdgeHandle = "E";
}
make_handle_type!{
    /// A handle that is associated with a vertex.
    VertexHandle = "V";
}

/// An optional handle, semantically equivalent to `Option<H>`.
///
/// Sadly, it's not too easy to make `Option<H>` the same size as `H`. So we
/// need our own optional-type to store space efficient optional handles. We
/// use `hsize::max_value` as `None` value.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Opt<H: Handle>(H);

impl<H: Handle> Opt<H> {
    /// Returns a `None` instance of this optional handle.
    #[inline(always)]
    pub fn none() -> Self {
        Opt(H::new(hsize::max_value()))
    }

    /// Creates a `Some` instance with the given handle.
    #[inline(always)]
    pub fn some(handle: H) -> Self {
        Opt(handle)
    }

    /// Converts `self` to `Option<H>`.
    #[inline(always)]
    pub fn to_option(self) -> Option<H> {
        if self.is_none() {
            None
        } else {
            Some(self.0)
        }
    }

    /// Returns `true` if there is no handle inside.
    #[inline(always)]
    pub fn is_none(self) -> bool {
        self.0.idx() == hsize::max_value()
    }

    /// Returns `true` if there is a handle inside.
    #[inline(always)]
    pub fn is_some(self) -> bool {
        !self.is_none()
    }

    /// Returns the stored handle or panics if `self.is_none()`.
    #[inline(always)]
    pub fn unwrap(self) -> H {
        if self.is_none() {
            panic!("called `unwrap()` on a `Opt::none` value");
        }

        self.0
    }
}

impl<H: Handle> From<H> for Opt<H> {
    fn from(src: H) -> Self {
        Self::some(src)
    }
}

impl<H: Handle> fmt::Debug for Opt<H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // We don't just forward it because we want to stay in one line, even
        // with `#` pretty printing activated.
        match self.to_option() {
            Some(h) => write!(f, "Some({:?})", h),
            None => write!(f, "None"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// This could/should be a compile time check, but there is no easy,
    /// built-in `static_assert` yet, so this has to be sufficient.
    #[test]
    fn opt_small_size() {
        use std::mem::size_of;

        assert_eq!(size_of::<FaceHandle>(), size_of::<Opt<FaceHandle>>());
        assert_eq!(size_of::<VertexHandle>(), size_of::<Opt<VertexHandle>>());
        assert_eq!(size_of::<EdgeHandle>(), size_of::<Opt<EdgeHandle>>());
    }
}
