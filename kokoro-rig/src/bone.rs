//! Bone primitives and the small 2D vector type the rig uses.
//!
//! There is no `glam` or `nalgebra` here on purpose — the rig only ever needs
//! +/- /scale/rotate/length on 2D points, and pulling in a heavy linalg crate
//! for a 64×64 sprite world buys nothing but compile time. The local `Vec2`
//! is `Copy`, has the small pile of operations the rig needs, and is easy to
//! audit.

/// Stable identifier for a bone within a `Skeleton`. Wraps a `u16` index into
/// the skeleton's `bones` vector — never a pointer, so the id stays valid as
/// long as the skeleton structure isn't re-ordered.
///
/// `u16` (max 65 535) is overkill for any creature in this game; it leaves
/// room for compound rigs without forcing `usize` everywhere.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoneId(pub u16);

impl BoneId {
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Whether a bone is driven by the rig's pose system or by an external
/// soft-body simulation. The `softbody::reconcile` pass uses this to decide
/// the direction information flows on a frame-by-frame basis.
///
/// - `Rigid` (default): skeleton wins. After FK, the bone's tip is pushed
///   into the soft-body point cloud (anchored points). Use for skull,
///   torso, mantle — anything that should hold its shape.
/// - `Soft`: soft body wins. The simulation moves a point freely; the
///   bone reads the point and rotates to match. Use for tail tips,
///   tentacles, ear tips — anything that should drift with physics.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Stiffness {
    Rigid,
    Soft,
}

/// Plain 2D point/vector in pixel space. Y grows downward to match the
/// `image` crate's pixel convention used everywhere else in Kokoro.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Rotate this vector by `angle` radians around the origin. Standard
    /// 2D rotation matrix; positive angle = clockwise in image-space (since
    /// y grows down).
    #[inline]
    pub fn rotated(self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            x: self.x * c - self.y * s,
            y: self.x * s + self.y * c,
        }
    }

    #[inline]
    pub fn add(self, rhs: Vec2) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }

    #[inline]
    pub fn sub(self, rhs: Vec2) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }

    #[inline]
    pub fn scale(self, s: f32) -> Self {
        Self { x: self.x * s, y: self.y * s }
    }

    #[inline]
    pub fn distance(self, rhs: Vec2) -> f32 {
        self.sub(rhs).length()
    }

    /// `atan2(y, x)` of this vector — angle from the +x axis. Useful when
    /// turning a "where is the target relative to me" delta into a rotation.
    #[inline]
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }
}

/// Material composition of a bone — currently mass only. Held as a struct
/// so future fields (density, fracture state, marrow_state) can be added
/// without breaking the `Bone` API.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BoneTissue {
    /// Mass of the bone in kilograms. The default 1.0 is "one body unit"
    /// — gameplay parameters can stay dimensionless until a real physical
    /// reference is chosen.
    pub mass: f32,
}

impl BoneTissue {
    pub const fn new(mass: f32) -> Self {
        Self { mass }
    }
}

impl Default for BoneTissue {
    fn default() -> Self {
        Self { mass: 1.0 }
    }
}

/// A single bone in a skeleton.
///
/// Position model: a bone has a **base** (origin) and a **tip** (where its
/// length ends). The base is computed during forward kinematics from the
/// parent's tip plus this bone's `rest_offset` (rotated into the parent's
/// frame). The tip is base + length × direction(world_angle). Children
/// attach to the tip via their own `rest_offset`.
///
/// `rest_offset` is in the *parent's local frame* — measured from the
/// parent's tip, before any pose rotation is applied. This means the
/// skeleton's rest pose can be authored as flat numbers without pre-rotating.
#[derive(Clone, Debug)]
pub struct Bone {
    pub name: &'static str,
    pub parent: Option<BoneId>,

    /// Offset from the parent's tip, in the parent's local (rest) frame.
    /// For the root bone this is ignored — the root sits at the skeleton's
    /// `root_position`.
    pub rest_offset: Vec2,

    /// Rest rotation in radians, relative to the parent's world angle.
    /// Pose layers add their own delta on top of this without overwriting it.
    pub rest_angle: f32,

    /// Length of this bone in pixels. Visual brushes use this to know how
    /// far to paint along the bone's direction.
    pub length: f32,

    /// Visual thickness (used by brushes; the rig itself doesn't care).
    pub width: f32,

    /// Render order relative to siblings/peers. Lower z paints first.
    /// Range -2..=+2 is enough for our pixel sprites; using `i8` makes the
    /// intent obvious and saves bytes.
    pub z_layer: i8,

    /// Per-creature length multiplier sourced from genome. 1.0 = no
    /// modulation; gameplay code typically sets values in 0.85..=1.15.
    pub genome_length_scale: f32,

    /// Per-creature width multiplier. Same convention as above.
    pub genome_width_scale: f32,

    /// How this bone interacts with an external soft-body simulation.
    /// See `Stiffness` docs. Default is `Rigid`.
    pub stiffness: Stiffness,

    /// Material composition of the bone. Defaults to mass = 1.0 (one body
    /// unit). The physics integrator uses this to derive moment of
    /// inertia for the joint that hangs this bone.
    pub tissue: BoneTissue,
}

impl Bone {
    /// Construct the root bone of a skeleton. Convenience constructor —
    /// forces `parent = None` and `rest_offset = ZERO` so the root can't be
    /// mis-initialised with stale offsets.
    pub const fn root(name: &'static str, length: f32, width: f32) -> Self {
        Self {
            name,
            parent: None,
            rest_offset: Vec2::ZERO,
            rest_angle: 0.0,
            length,
            width,
            z_layer: 0,
            genome_length_scale: 1.0,
            genome_width_scale: 1.0,
            stiffness: Stiffness::Rigid,
            tissue: BoneTissue { mass: 1.0 },
        }
    }

    /// Construct a child bone hung off the given parent. `rest_offset` is in
    /// the parent's local frame; `rest_angle` is the bone's rotation
    /// relative to the parent's world angle.
    pub const fn child(
        name: &'static str,
        parent: BoneId,
        rest_offset: Vec2,
        rest_angle: f32,
        length: f32,
        width: f32,
    ) -> Self {
        Self {
            name,
            parent: Some(parent),
            rest_offset,
            rest_angle,
            length,
            width,
            z_layer: 0,
            genome_length_scale: 1.0,
            genome_width_scale: 1.0,
            stiffness: Stiffness::Rigid,
            tissue: BoneTissue { mass: 1.0 },
        }
    }

    pub const fn with_tissue(mut self, tissue: BoneTissue) -> Self {
        self.tissue = tissue;
        self
    }

    pub const fn with_z(mut self, z: i8) -> Self {
        self.z_layer = z;
        self
    }

    pub const fn with_genome_scales(mut self, length_scale: f32, width_scale: f32) -> Self {
        self.genome_length_scale = length_scale;
        self.genome_width_scale = width_scale;
        self
    }

    pub const fn with_stiffness(mut self, s: Stiffness) -> Self {
        self.stiffness = s;
        self
    }

    /// Length after genome modulation — what FK and brushes should actually
    /// use. The raw `length` field is the "design" value.
    #[inline]
    pub fn effective_length(&self) -> f32 {
        self.length * self.genome_length_scale
    }

    #[inline]
    pub fn effective_width(&self) -> f32 {
        self.width * self.genome_width_scale
    }
}
