use std::f32::consts::TAU;

use eframe::egui::*;

/// Trait shared by everything that can be plotted.
pub trait Drawable {
    /// Function to turn the drawable item into Shapes.
    fn paint(self, painter: &mut Painter, transform: &dyn Fn(&Pos2) -> Pos2);
}

/// Text positioned on the plot.
/// Does not work correctly right now due to how the text is transformed.
pub struct Text {
    position: Pos2,
    _rotation: f32,
    text: String,
    color: Color32,
    anchor: Align2,
}

impl Text {
    pub fn new(position: Pos2, text: impl Into<String>) -> Self {
        Self {
            position,
            _rotation: 0.,
            text: text.into(),
            color: Color32::WHITE,
            anchor: Align2::CENTER_CENTER,
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

impl Drawable for Text {
    fn paint(self, painter: &mut Painter, transform: &dyn Fn(&Pos2) -> Pos2) {
        let Text {
            position,
            _rotation,
            text,
            color,
            anchor,
        } = self;

        painter.text(
            transform(&position),
            anchor,
            text,
            TextStyle::Monospace,
            color,
        );
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
    fn paint(self, painter: &mut Painter, transform: &dyn Fn(&Pos2) -> Pos2) {
        let Self {
            points,
            fill,
            stroke,
        } = self;

        painter.add(Shape::polygon(
            points.iter().map(|p| transform(p)).collect(),
            fill,
            stroke,
        ));
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MarkerShape {
    Circle,
    Triangle,
    Square,
    Plus,
    X,
    Star,
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
    fn paint(self, painter: &mut Painter, transform: &dyn Fn(&Pos2) -> Pos2) {
        let Self {
            points,
            fill,
            stroke,
            size,
            shape,
            stems,
            stems_stroke,
        } = self;

        points.iter().for_each(|p| {
            let p0 = transform(&Pos2::new(p.x, 0.));
            let p1 = transform(p);
            if stems {
                painter.line_segment([p0, p1], stems_stroke);
            }

            match shape {
                MarkerShape::Circle => painter.circle(p1, size, fill, stroke),
                MarkerShape::Square => painter.rect(
                    Rect::from_center_size(p1, Vec2::new(2. * size, 2. * size)),
                    0.,
                    fill,
                    stroke,
                ),
                MarkerShape::Triangle => {
                    let outer_radius = 1.0 * size;
                    let inner_radius = 0.5 * size;
                    let bottom = Vec2::new(0., -outer_radius);
                    let left = Vec2::new(-(3f32.sqrt()) / 2. * outer_radius, inner_radius);
                    let right = Vec2::new(3f32.sqrt() / 2. * outer_radius, inner_radius);
                    let points = vec![p1 + bottom, p1 + right, p1 + left];
                    painter.add(Shape::polygon(points, fill, stroke));
                }
                MarkerShape::Plus => {
                    let dx = Vec2::new(size, 0.);
                    painter.line_segment([p1 - dx, p1 + dx], stroke);
                    let dy = Vec2::new(0., size);
                    painter.line_segment([p1 - dy, p1 + dy], stroke);
                }
                MarkerShape::X => {
                    let diag = Vec2::new(size, size) / std::f32::consts::SQRT_2;
                    painter.line_segment([p1 - diag, p1 + diag], stroke);
                    let diag = diag.rot90();
                    painter.line_segment([p1 - diag, p1 + diag], stroke);
                }
                MarkerShape::Star => {
                    let spikes = 8; // Has to be be even.
                    (0..spikes / 2).for_each(|i| {
                        let angle = i as f32 / spikes as f32 * TAU;
                        let diag = Vec2::angled(angle) * size;
                        painter.line_segment([p1 - diag, p1 + diag], stroke);
                    });
                }
            };
        });
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
    fn paint(self, painter: &mut Painter, transform: &dyn Fn(&Pos2) -> Pos2) {
        let Self {
            points,
            color,
            weight,
        } = self;

        painter.add(Shape::line(
            points.iter().map(|p| transform(p)).collect(),
            Stroke::new(weight, color),
        ));
    }
}

pub struct Quiver {
    points: Vec<Pos2>,
    directions: Vec<Vec2>,
    color: Color32,
    weight: f32,
}

impl Quiver {
    pub fn new(points: Vec<Pos2>, directions: Vec<Vec2>) -> Self {
        Self {
            points,
            directions,
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

impl Drawable for Quiver {
    fn paint(self, painter: &mut Painter, transform: &dyn Fn(&Pos2) -> Pos2) {
        let Self {
            points,
            directions,
            color,
            weight,
        } = self;

        points
            .iter()
            .zip(directions.iter())
            .for_each(|(point, direction)| {
                let p0 = transform(point);
                let p1 = transform(&(*point + *direction));

                painter.arrow(p0, p1 - p0, Stroke::new(weight, color));
            });
    }
}
