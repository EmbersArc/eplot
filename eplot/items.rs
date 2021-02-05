use eframe::egui::*;

/// Trait shared by everything that can be plotted.
pub trait PlotItem {
    /// Function to turn the drawable item into Shapes.
    fn paint(self, painter: &mut Painter, transform: &dyn Fn(&Pos2) -> Pos2);
}

/// Text positioned on the plot.
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

impl PlotItem for Text {
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

impl PlotItem for Polygon {
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

pub enum YReference {
    Constant(f32),
    Series(Vec<f32>),
}

/// Plot a set of points.
pub struct Scatter {
    points: Vec<Pos2>,
    fill: Color32,
    stroke: Stroke,
    size: f32,
    shape: MarkerShape,
    stems: Option<(YReference, Stroke)>,
}

impl Scatter {
    pub fn new(points: Vec<Pos2>) -> Self {
        Self {
            points,
            fill: Color32::WHITE,
            stroke: Stroke::none(),
            size: 1.,
            shape: MarkerShape::Circle,
            stems: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn stems(mut self, reference: YReference, stroke: Stroke) -> Self {
        if let YReference::Series(series) = &reference {
            assert!(
                series.len() == self.points.len(),
                "The numer of y-axis reference values needs to match the data!"
            );
        }
        self.stems = Some((reference, stroke));
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

impl PlotItem for Scatter {
    fn paint(self, painter: &mut Painter, transform: &dyn Fn(&Pos2) -> Pos2) {
        let Self {
            points,
            fill,
            stroke,
            size,
            shape,
            stems,
        } = self;

        points.iter().enumerate().for_each(|(i, p)| {
            let p_tf = transform(p);

            if let Some((reference, stroke)) = &stems {
                let current_ref = match reference {
                    YReference::Constant(c) => *c,
                    YReference::Series(s) => *s.get(i).unwrap(),
                };

                let p_ref_tf = transform(&Pos2::new(p.x, current_ref));

                painter.line_segment([p_ref_tf, p_tf], *stroke);
            }

            match shape {
                MarkerShape::Circle => painter.circle(p_tf, size, fill, stroke),
                MarkerShape::Square => painter.rect(
                    Rect::from_center_size(p_tf, Vec2::new(2. * size, 2. * size)),
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
                    let points = vec![p_tf + bottom, p_tf + right, p_tf + left];
                    painter.add(Shape::polygon(points, fill, stroke));
                }
                MarkerShape::Plus => {
                    let dx = Vec2::new(size, 0.);
                    painter.line_segment([p_tf - dx, p_tf + dx], stroke);
                    let dy = Vec2::new(0., size);
                    painter.line_segment([p_tf - dy, p_tf + dy], stroke);
                }
                MarkerShape::X => {
                    let diag = Vec2::new(size, size) / std::f32::consts::SQRT_2;
                    painter.line_segment([p_tf - diag, p_tf + diag], stroke);
                    let diag = diag.rot90();
                    painter.line_segment([p_tf - diag, p_tf + diag], stroke);
                }
                MarkerShape::Star => {
                    let spikes = 8; // Has to be be even.
                    use std::f32::consts::TAU;
                    (0..spikes / 2).for_each(|i| {
                        let angle = i as f32 / spikes as f32 * TAU;
                        let diag = Vec2::angled(angle) * size;
                        painter.line_segment([p_tf - diag, p_tf + diag], stroke);
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
    area_fill: Option<(YReference, Color32)>,
}

impl Line {
    pub fn new(points: Vec<Pos2>) -> Self {
        Self {
            points,
            color: Color32::WHITE,
            weight: 1.,
            area_fill: None,
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

    pub fn area_fill(mut self, reference: YReference, color: Color32) -> Self {
        self.area_fill = Some((reference, color));
        self
    }
}

impl PlotItem for Line {
    fn paint(self, painter: &mut Painter, transform: &dyn Fn(&Pos2) -> Pos2) {
        let Self {
            points,
            color,
            weight,
            area_fill,
        } = self;

        // TODO: Ew. Make this better.
        if let Some((reference, color)) = area_fill {
            points.windows(2).enumerate().for_each(|(i, w)| {
                let y_ref = match &reference {
                    YReference::Constant(c) => (*c, *c),
                    YReference::Series(s) => (s[i], s[i + 1]),
                };
                let start_down = transform(&pos2(w[0].x, y_ref.0));
                let end_down = transform(&pos2(w[1].x, y_ref.1));
                painter.add(Shape::polygon(
                    vec![transform(&w[1]), transform(&w[0]), start_down, end_down],
                    color,
                    Stroke::default(),
                ));
            });
        }

        let points: Vec<Pos2> = points.iter().map(|p| transform(p)).collect();

        painter.add(Shape::line(points, Stroke::new(weight, color)));
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

impl PlotItem for Quiver {
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
