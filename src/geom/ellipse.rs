use geom::{Rect, Tri};
use math::{self, BaseNum, Point2};
use math::num_traits::{Float, NumCast};
use std;
use std::ops::Neg;

/// A simple ellipse type with helper methods around the `ellipse` module's functions.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Ellipse<S = f64> {
    /// The width and height off the `Ellipse`.
    pub rect: Rect<S>,
    /// The resolution (number of sides) of the `Ellipse`.
    pub resolution: usize,
}

/// A subsection of an `Ellipse`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Section<S = f64> {
    /// The ellipse from which this section is produced.
    pub ellipse: Ellipse<S>,
    /// The angle in radians of the start of the section.
    pub offset_radians: S,
    /// The section of the circumference in radians.
    pub section_radians: S,
}

/// An iterator yielding the edges of an ellipse (or some section of an `ellipse`) as a series of
/// points.
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct Circumference<S = f64> {
    index: usize,
    num_points: usize,
    middle: Point2<S>,
    rad_step: S,
    rad_offset: S,
    half_w: S,
    half_h: S,
}

/// An iterator yielding triangles that describe an oval or some section of an oval.
#[derive(Clone, Debug)]
pub struct Triangles<S> {
    // The last circumference point yielded by the `CircumferenceOffset` iterator.
    last: Point2<S>,
    // The circumference points used to yield yielded by the `CircumferenceOffset` iterator.
    points: Circumference<S>,
}

impl<S> Ellipse<S>
where
    S: BaseNum + Neg<Output=S>,
{
    /// Construct a new ellipse from its bounding rect and resolution (number of sides).
    pub fn new(rect: Rect<S>, resolution: usize) -> Self {
        Ellipse { rect, resolution }
    }

    /// A section of the `Ellipse`.
    ///
    /// `offset_radians` describes the angle at which the offset begins.
    ///
    /// `section_radians` describes how large the section is as an angle.
    pub fn section(self, offset_radians: S, section_radians: S) -> Section<S> {
        Section {
            ellipse: self,
            offset_radians,
            section_radians,
        }
    }

    /// Produces an iterator yielding the points of the ellipse circumference.
    pub fn circumference(self) -> Circumference<S> {
        let Ellipse { rect, resolution } = self;
        Circumference::new(rect, resolution)
    }

    /// Produces an iterator yielding the triangles that describe the ellipse.
    ///
    /// TODO: Describe the order.
    pub fn triangles(self) -> Triangles<S>
    where
        S: Float,
    {
        self.circumference().triangles()
    }
}

impl<S> Section<S>
where
    S: BaseNum + Neg<Output=S>,
{
    /// Produces an iterator yielding the points of the ellipse circumference.
    pub fn circumference(self) -> Circumference<S> {
        let Section { ellipse, offset_radians, section_radians } = self;
        let circ = Circumference::new_section(ellipse.rect, ellipse.resolution, section_radians);
        circ.offset_radians(offset_radians)
    }

    /// Produces an iterator yielding the triangles that describe the ellipse section.
    ///
    /// TODO: Describe the order.
    pub fn trangles(self) -> Triangles<S>
    where
        S: Float,
    {
        self.circumference().triangles()
    }
}

impl<S> Circumference<S>
where
    S: BaseNum + Neg<Output=S>,
{
    fn new_inner(rect: Rect<S>, num_points: usize, rad_step: S) -> Self {
        let (x, y, w, h) = rect.x_y_w_h();
        let two = math::two();
        Circumference {
            index: 0,
            num_points: num_points,
            middle: [x, y].into(),
            half_w: w / two,
            half_h: h / two,
            rad_step: rad_step,
            rad_offset: S::zero(),
        }
    }

    /// An iterator yielding the ellipse's edges as a circumference represented as a series of
    /// points.
    ///
    /// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
    /// produces `NaN` values.
    pub fn new(rect: Rect<S>, mut resolution: usize) -> Self {
        resolution = std::cmp::max(resolution, 1);
        use std::f64::consts::PI;
        let radians = S::from(2.0 * PI).unwrap();
        Self::new_section(rect, resolution, radians)
    }

    /// Produces a new iterator that yields only a section of the ellipse's circumference, where
    /// the section is described via its angle in radians.
    ///
    /// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
    /// produces `NaN` values.
    pub fn new_section(rect: Rect<S>, resolution: usize, radians: S) -> Self
    where
        S: BaseNum,
    {
        let res = S::from(resolution).unwrap();
        Self::new_inner(rect, resolution + 1, radians / res)
    }

    /// Produces a new iterator that yields only a section of the ellipse's circumference, where
    /// the section is described via its angle in radians.
    pub fn section(mut self, radians: S) -> Self {
        let resolution = self.num_points - 1;
        let res = S::from(resolution).unwrap();
        self.rad_step = radians / res;
        self
    }

    /// Rotates the position at which the iterator starts yielding points by the given radians.
    ///
    /// This is particularly useful for yielding a different section of the circumference when
    /// using `circumference_section`
    pub fn offset_radians(mut self, radians: S) -> Self {
        self.rad_offset = radians;
        self
    }

    /// Produces an `Iterator` yielding `Triangle`s.
    ///
    /// Triangles are created by joining each edge yielded by the inner `Circumference` to the
    /// middle of the ellipse.
    pub fn triangles(mut self) -> Triangles<S>
    where
        S: Float,
    {
        let last = self.next().unwrap_or(self.middle);
        Triangles { last, points: self }
    }
}

impl<S> Iterator for Circumference<S>
where
    S: BaseNum + Float,
{
    type Item = Point2<S>;
    fn next(&mut self) -> Option<Self::Item> {
        let Circumference {
            ref mut index,
            num_points,
            middle,
            rad_step,
            rad_offset,
            half_w,
            half_h,
        } = *self;
        if *index >= num_points {
            return None;
        }
        let index_s: S = NumCast::from(*index).unwrap();
        let x = middle.x + half_w * (rad_offset + rad_step * index_s).cos();
        let y = middle.y + half_h * (rad_offset + rad_step * index_s).sin();
        *index += 1;
        Some([x, y].into())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

// TODO:
// impl<S> DoubleEndedIterator for Circumference<S>
// where
//     S: BaseNum + Float,
// {
// }

impl<S> ExactSizeIterator for Circumference<S>
where
    S: BaseNum + Float,
{
    fn len(&self) -> usize {
        self.num_points - self.index
    }
}

impl<S> Iterator for Triangles<S>
where
    S: BaseNum + Float,
{
    type Item = Tri<Point2<S>>;
    fn next(&mut self) -> Option<Self::Item> {
        let Triangles { ref mut points, ref mut last } = *self;
        points.next().map(|next| {
            let triangle = Tri([points.middle, *last, next]);
            *last = next;
            triangle
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> ExactSizeIterator for Triangles<S>
where
    S: BaseNum + Float,
{
    fn len(&self) -> usize {
        self.points.len()
    }
}
