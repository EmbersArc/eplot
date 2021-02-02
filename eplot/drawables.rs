use eframe::egui::*;
use paint::text::Fonts;

/// Trait shared by everything that can be plotted.
pub trait Drawable {
    /// Function to turn the drawable item into Shapes.
    fn to_shapes(self) -> Vec<Shape>;
}

/// Text positioned on the plot.
/// Does not work correctly right now due to how the text is transformed.
pub struct Text<'a> {
    position: Pos2,
    _rotation: f32,
    text: String,
    color: Color32,
    anchor: Align2,
    fonts: &'a Fonts,
}

impl<'a> Text<'a> {
    pub fn new(position: Pos2, text: impl Into<String>, fonts: &'a Fonts) -> Self {
        Self {
            position,
            _rotation: 0.,
            text: text.into(),
            color: Color32::WHITE,
            anchor: Align2::CENTER_CENTER,
            fonts,
        }
    }

    pub fn rotation(mut self, _rotation: f32) -> Self {
        self._rotation = _rotation;
        self
    }

    pub fn anchor(mut self, anchor: Align2) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
}

impl<'a> Drawable for Text<'a> {
    fn to_shapes(self) -> Vec<Shape> {
        let Text {
            position,
            text,
            color,
            anchor,
            fonts,
            ..
        } = self;

        vec![Shape::text(
            fonts,
            position,
            anchor,
            text,
            TextStyle::Monospace,
            color,
        )]
    }
}

/// A closed line. The first and last points are connected automatically.
/// Non-convex shapes may lead to unexpected results when `fill` is enabled.
pub struct Polygon {
    points: Vec<Pos2>,
    fill: Color32,
    stroke: Stroke,
}

impl Polygon {
    pub fn new(points: Vec<Pos2>) -> Self {
        Self {
            points,
            fill: Color32::WHITE,
            stroke: Stroke::none(),
        }
    }

    pub fn fill_color(mut self, color: Color32) -> Self {
        self.fill = color;
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }
}

impl Drawable for Polygon {
    fn to_shapes(self) -> Vec<Shape> {
        let Self {
            points,
            fill,
            stroke,
            ..
        } = self;

        vec![Shape::polygon(points, fill, stroke)]
    }
}

pub enum MarkerShape {
    Circle,
}

/// Plot a set of points.
pub struct Scatter {
    points: Vec<Pos2>,
    fill: Color32,
    stroke: Stroke,
    size: f32,
    shape: MarkerShape,
    stems: bool,
    stems_stroke: Stroke,
}

impl Scatter {
    pub fn new(points: Vec<Pos2>) -> Self {
        Self {
            points,
            fill: Color32::WHITE,
            stroke: Stroke::none(),
            size: 1.,
            shape: MarkerShape::Circle,
            stems: false,
            stems_stroke: Stroke::new(1., Color32::WHITE),
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn stems(mut self, on: bool) -> Self {
        self.stems = on;
        self
    }

    pub fn stems_stroke(mut self, stroke: Stroke) -> Self {
        self.stems_stroke = stroke;
        self
    }

    pub fn fill_color(mut self, color: Color32) -> Self {
        self.fill = color;
        self
    }

    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn shape(mut self, shape: MarkerShape) -> Self {
        self.shape = shape;
        self
    }
}

impl Drawable for Scatter {
    fn to_shapes(self) -> Vec<Shape> {
        let Self {
            points,
            fill,
            stroke,
            size,
            shape,
            stems,
            stems_stroke,
            ..
        } = self;

        let mut shapes = Vec::with_capacity(if stems { 2 } else { 1 } * points.len());

        if stems {
            shapes.extend(points.iter().map(|p| Shape::LineSegment {
                points: [*p, Pos2::new(p.x, 0.)],
                stroke: stems_stroke,
            }));
        }

        shapes.extend(points.iter().map(|p| match shape {
            MarkerShape::Circle => Shape::Circle {
                center: *p,
                radius: size,
                fill,
                stroke,
            },
        }));

        shapes
    }
}

/// Plot a sequence of connected points.
pub struct Line {
    points: Vec<Pos2>,
    color: Color32,
    weight: f32,
}

impl Line {
    pub fn new(points: Vec<Pos2>) -> Self {
        Self {
            points,
            color: Color32::WHITE,
            weight: 1.,
        }
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }
}

impl Drawable for Line {
    fn to_shapes(self) -> Vec<Shape> {
        let Self {
            points,
            color,
            weight,
            ..
        } = self;

        vec![Shape::line(points, Stroke::new(weight, color))]
    }
}
