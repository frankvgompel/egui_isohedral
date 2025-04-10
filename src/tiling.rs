
use crate::data::{tiling_type_data, TilingTypeData};
use crate::iterators::{FillAlgorithm, TilingShapeIterator, TilingShapePartIterator};
use crate::utils::{fill_affine, fill_vector, r_match, Affine2, Vec2, vec2};


/// One of the 93 isohedral tiling types. Can be used to initialise or reset an [`IsohedralTiling`]
/// instance.
#[derive(Debug, Default, Clone, Copy)]
pub struct TilingType(pub(crate) usize);

impl std::fmt::Display for TilingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IH{:02}", self.0)
    }
}

/// Represents the "shape" of an edge, i.e. the set of constraints that this edge must follow.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EdgeShape {
    /// Edges that can be of any shape
    J,
    /// Edges that must look the same after reflecting across their length (like the letter `U`)
    U,
    /// Edges that must look the same after a 180° rotation (like the letter `S`)
    S,
    /// Edges that must look the same after both a 180° and a reflection (like the letter `I`)
    I,
}

/// Represents a particular isohedral tiling type.
#[derive(Debug, Default)]
pub struct IsohedralTiling {
    pub(crate) tiling_type: TilingType,
    pub(crate) num_params: usize,
    pub(crate) parameters: [f32; 6],
    pub(crate) vertices: [Vec2; 6],
    pub(crate) edges: [Affine2; 6],
    pub(crate) reversals: [bool; 6],
    pub(crate) aspects: [Affine2; 12],
    pub(crate) t1: Vec2,
    pub(crate) t2: Vec2,
    pub(crate) ttd: &'static TilingTypeData,
}

impl IsohedralTiling {
    /// Create a new [`IsohedralTiling`] instance for the given tiling type. You can get a valid
    /// tiling type by using the [`get_tiling_type`] function.
    pub fn new(ihtype: TilingType) -> Self {
        let mut tiling = Self::default();
        tiling.reset(ihtype);

        tiling
    }

    /// Resets the current instance to describe the new given tiling type, and recomputes the
    /// internal state as necessary.
    pub fn reset(&mut self, ihtype: TilingType) {
        self.tiling_type = ihtype;
        let ttd = &tiling_type_data[ihtype.0];

        self.num_params = ttd.num_params;
        self.ttd = ttd;

        self.parameters[..ttd.num_params].copy_from_slice(ttd.default_params);
        self.recompute();
    }

    /// # Accessors

    /// The tiling type described by this instance.
    pub fn tiling_type(&self) -> TilingType {
        self.tiling_type
    }

    /// The number of parameters that can affect the shape of the prototile. Some tiling types have
    /// no parameters (i.e. the prototile is a fixed shape), others have up to 6 parameters.
    pub fn num_params(&self) -> usize {
        self.num_params
    }

    /// The number of different edge shapes for the current tiling type's prototile.
    pub fn num_edge_shapes(&self) -> usize {
        self.ttd.num_edge_shapes
    }

    /// The number of vertices that the current prototile has.
    pub fn num_vertices(&self) -> usize {
        self.ttd.num_vertices
    }

    /// Returns the shape of the given edge.
    ///
    /// See [`num_edge_shapes`] for the valid range of values for `idx`.
    ///
    /// [`num_edge_shapes`]: IsohedralTiling::num_edge_shapes
    pub fn edge_shape(&self, idx: usize) -> EdgeShape {
        self.ttd.edge_shapes[idx]
    }

    /// Returns the vertex specified by `idx`.
    ///
    /// See [`num_vertices`] for the valid range of values for `idx`.
    ///
    /// [`num_vertices`]: IsohedralTiling::num_vertices
    pub fn vertex(&self, idx: usize) -> &Vec2 {
        &self.vertices[idx]
    }

    /// The number of aspects that the current tiling type has.
    pub fn num_aspects(&self) -> usize {
        self.ttd.num_aspects
    }

    /// Returns the aspect transformation matrix for the given aspect index.
    ///
    /// See [`num_aspects`] for the valid range of values for `idx`.
    ///
    /// [`num_aspects`]: IsohedralTiling::num_aspects
    pub fn aspect_transform(&self, idx: usize) -> &Affine2 {
        &self.aspects[idx]
    }

    /// Computes a colour index used for tiling a region.
    ///
    /// The return value can be 0, 1, 2, representing one of 3 possible colours. The parameters `t1`,
    /// `t2`, and `aspect` can be obtained while iterating over the tiles of a region. See
    /// [`iterators::FillRegionIterator`].
    pub fn colour(&self, t1: isize, t2: isize, aspect: usize) -> usize{
        let nc = self.ttd.colouring[18] as isize;

        let mut mt1 = t1 % nc;
        if mt1 < 0 {
            mt1 += nc;
        }
        let mut mt2 = t2 % nc;
        if mt2 < 0 {
            mt2 += nc;
        }
        let mut col = self.ttd.colouring[aspect];

        for _ in 0..mt1 {
            col = self.ttd.colouring[12 + col as usize];
        }

        for _ in 0..mt2 {
            col = self.ttd.colouring[15 + col as usize];
        }

        col as usize
    }

    /// The first translation vector.
    pub fn t1(&self) -> &Vec2 {
        &self.t1
    }

    /// The second translation vector.
    pub fn t2(&self) -> &Vec2 {
        &self.t2
    }

    /// # Iterators

    /// Iterate over all the edge shapes of the prototile.
    pub fn shapes(&self) -> TilingShapeIterator {
        TilingShapeIterator {
            idx: 0,
            tiling: self,
        }
    }

    /// Iterate over all the shape parts of the prototile.
    pub fn parts(&self) -> TilingShapePartIterator {
        TilingShapePartIterator {
            idx: 0,
            tiling: self,
            second: false,
        }
    }

    /// Helper to fill a region of the plan with tiles.
    ///
    /// The  returned object can be turned into an iteretor where each element gives you the necessary
    /// transform to apply to the prototile.
    pub fn fill_region(&self, xmin: f32, ymin: f32, xmax: f32, ymax: f32) -> FillAlgorithm<'_> {
        FillAlgorithm::new(
            self,
            vec2(xmin, ymin),
            vec2(xmax, ymin),
            vec2(xmax, ymax),
            vec2(xmin, ymax),
        )
    }

    /// Return all the vertex parameters.
    ///
    /// Note: not all tiling types have the same number of parameters. Only the first `n` values of the
    /// returned array are valid, where `n` is the value returned by [`#num_params].
    pub fn parameters(&self, params: &mut [f32; 6]) {
        params.copy_from_slice(&self.parameters);
    }

    /// Set the vertex parameters.
    ///
    /// See also: [`#parameters`]
    pub fn set_parameters(&mut self, params: &[f32; 6]) {
        self.parameters.copy_from_slice(params);
        self.recompute();
    }

    /// Return the vertices for this prototile.
    ///
    /// See also: [`#parameters`]
    pub fn vertices(&self) -> &[Vec2] {
        &self.vertices[0..self.num_vertices()]
    }

    fn recompute(&mut self) {
        let ntv = self.ttd.num_vertices;

        // Recompute tiling vertex locations
        let mut data = self.ttd.tiling_vertex_coeffs;
        for idx in 0..ntv {
            fill_vector(
                data,
                &self.parameters,
                self.num_params,
                &mut self.vertices[idx],
            );
            data = &data[(2 * (self.num_params + 1))..];
        }

        // Recompute edge transforms and reversals from orientation information
        for idx in 0..ntv {
            let fl = self.ttd.edge_orientations[2 * idx];
            let ro = self.ttd.edge_orientations[2 * idx + 1];
            self.reversals[idx] = fl != ro;
            self.edges[idx] = r_match(&self.vertices[idx], &self.vertices[(idx + 1) % ntv])
                * crate::utils::M_ORIENTS[2 * (fl as usize) + (ro as usize)];
        }

        // Recompute aspect xforms
        data = self.ttd.aspect_xform_coeffs;
        let sz = self.ttd.num_aspects;
        for idx in 0..sz {
            fill_affine(
                data,
                &self.parameters,
                self.num_params,
                &mut self.aspects[idx],
            );
            data = &data[(6 * (self.num_params + 1))..];
        }

        // Recompute translation vectors
        data = self.ttd.translation_vertex_coeffs;
        fill_vector(data, &self.parameters, self.num_params, &mut self.t1);
        fill_vector(
            &data[(2 * (self.num_params + 1))..],
            &self.parameters,
            self.num_params,
            &mut self.t2,
        );
    }
}
