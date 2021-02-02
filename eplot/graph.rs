use eframe::egui::*;
use std::ops::RangeInclusive;

use super::drawables::Drawable;

pub struct Graph<'mem> {
    label: String,
    axis_equal: bool,
    show_cursor_pos: bool,
    x_axis_label: Option<String>,
    y_axis_label: Option<String>,
    memory: &'mem mut GraphMemory,
}

pub struct PlotUi {
    shapes: Vec<Shape>,
}

impl PlotUi {
    pub fn plot(&mut self, item: impl Drawable) {
        self.shapes.append(&mut item.to_shapes());
    }
}

pub struct GraphMemory {
    last_drag_pos: Option<Pos2>,
    plot_rect: Rect,
    first_run: bool,
}

impl Default for GraphMemory {
    fn default() -> Self {
        Self {
            last_drag_pos: None,
            plot_rect: Rect::from_center_size(Pos2::zero(), Vec2::new(10., 10.)),
            first_run: true,
        }
    }
}

impl<'mem> Graph<'mem> {
    pub fn new(label: impl Into<String>, memory: &'mem mut GraphMemory) -> Self {
        Self {
            label: label.into(),
            axis_equal: false,
            show_cursor_pos: true,
            x_axis_label: None,
            y_axis_label: None,
            memory,
        }
    }

    pub fn x_range(mut self, range: RangeInclusive<f32>) -> Self {
        if self.memory.first_run {
            self.memory.plot_rect.min.x = *range.start();
            self.memory.plot_rect.max.x = *range.end();
        }
        self
    }

    pub fn y_range(mut self, range: RangeInclusive<f32>) -> Self {
        if self.memory.first_run {
            self.memory.plot_rect.min.y = *range.start();
            self.memory.plot_rect.max.y = *range.end();
        }
        self
    }

    /// Will automatically adjust the y-axis so that the plot has an equal aspect ratio.
    pub fn axis_equal(mut self, on: bool) -> Self {
        self.axis_equal = on;
        self
    }

    /// Show the cursor position in the bottol left corner.
    pub fn show_cursor_position(mut self, on: bool) -> Self {
        self.show_cursor_pos = on;
        self
    }

    /// X-Axis label.
    pub fn x_axis_label(mut self, label: impl Into<String>) -> Self {
        self.x_axis_label = Some(label.into());
        self
    }

    // Y-Axis label. Not working yet since text can't be rotated.
    pub fn y_axis_label(mut self, label: impl Into<String>) -> Self {
        self.y_axis_label = Some(label.into());
        self
    }

    /// Draw the plot. Takes a closure where contents can be added to the plot.
    pub fn show<R>(self, ctx: &CtxRef, add_contents: impl FnOnce(&mut PlotUi) -> R) -> Response {
        let layer_id = LayerId::background();
        let id = Id::new(&self.label);

        let panel_rect = ctx.available_rect();

        let clip_rect = ctx.input().screen_rect();
        let mut panel_ui = Ui::new(ctx.clone(), layer_id, id, panel_rect, clip_rect);

        Frame {
            margin: Vec2::new(0.0, 0.0),
            corner_radius: 0.0,
            fill: Color32::from_gray(5),
            stroke: panel_ui.style().visuals.widgets.noninteractive.bg_stroke,
            ..Default::default()
        }
        .show(&mut panel_ui, |ui| {
            let Self {
                axis_equal,
                show_cursor_pos,
                x_axis_label,
                y_axis_label,
                memory,
                ..
            } = self;

            let GraphMemory {
                last_drag_pos,
                plot_rect,
                first_run,
            } = memory;

            let (response, mut painter) =
                ui.allocate_painter(ui.available_size_before_wrap_finite(), Sense::drag());

            // Plotting area
            let mut left_margin = 40.;
            let right_margin = 10.;
            let mut bottom_margin = 40.;
            let top_margin = 10.;
            if x_axis_label.is_some() {
                bottom_margin += 10.
            }
            if y_axis_label.is_some() {
                left_margin += 10.
            }
            let full_rect = response.rect;
            let painter_rect = Rect::from_min_max(
                full_rect.min + Vec2::new(left_margin, top_margin),
                full_rect.max - Vec2::new(right_margin, bottom_margin),
            );
            painter.rect(
                painter_rect,
                0.,
                Color32::from_gray(10),
                Stroke::new(1.0, Color32::from_white_alpha(150)),
            );

            if let Some(label) = &x_axis_label {
                painter.text(
                    painter_rect.center_bottom() + Vec2::new(0., 25.),
                    Align2::CENTER_TOP,
                    label,
                    TextStyle::Monospace,
                    Color32::WHITE,
                );
            }

            // Adjust the Y-axis so that the aspect ratio is equal.
            if axis_equal {
                let painter_height = painter_rect.height();
                let painter_width = painter_rect.width();
                let plot_width = plot_rect.width();
                let painter_ratio = painter_height / painter_width;
                let center = plot_rect.center();
                plot_rect.min.y = center.y - plot_width / 2. * painter_ratio;
                plot_rect.max.y = center.y + plot_width / 2. * painter_ratio;
            }

            // Dragging
            let new_drag_pos = response.interact_pointer_pos();
            if let Some(pos) = new_drag_pos {
                let mut pos_tf = pos;
                Self::transform_position(&mut pos_tf, &painter_rect, &plot_rect);
                if let Some(last_pos) = last_drag_pos {
                    ui.output().cursor_icon = CursorIcon::Grabbing;
                    let mut last_pos_tf = last_pos;
                    Self::transform_position(&mut last_pos_tf, &painter_rect, &plot_rect);
                    let delta = *last_pos_tf - pos_tf;
                    *plot_rect = plot_rect.translate(delta);
                }
                *last_drag_pos = Some(pos);
            } else {
                *last_drag_pos = None;
            }

            // Zooming
            let scrolled = ui.input().scroll_delta.y.max(-10.0).min(10.0);
            if let Some(mouse_pos) = ui.input().pointer.interact_pos() {
                if scrolled != 0. {
                    let left_distance = (mouse_pos.x - painter_rect.left()) / painter_rect.width();
                    let right_distance = 1. - left_distance;
                    let top_distance = (mouse_pos.y - painter_rect.top()) / painter_rect.height();
                    let bottom_distance = 1. - top_distance;
                    *plot_rect = Rect::from_min_max(
                        Pos2::new(
                            plot_rect.min.x + 0.01 * scrolled * plot_rect.width() * left_distance,
                            plot_rect.min.y
                                + 0.01 * scrolled * plot_rect.height() * bottom_distance,
                        ),
                        Pos2::new(
                            plot_rect.max.x - 0.01 * scrolled * plot_rect.width() * right_distance,
                            plot_rect.max.y - 0.01 * scrolled * plot_rect.height() * top_distance,
                        ),
                    );
                }
            }

            // Ticks and tick labels
            let ticks_on_smaller_axis = 5; // The lower limit of ticks on the smaller axis.
            let smaller_axis_size = plot_rect.width().min(plot_rect.height());
            let rough_increment = smaller_axis_size / ticks_on_smaller_axis as f32;
            let increment = emath::smart_aim::best_in_range_f64(
                (rough_increment * 0.5) as f64,
                (rough_increment * 1.5) as f64,
            ) as f32;

            // X-Axis ticks
            let mut i_start = (plot_rect.left() / increment) as i32;
            if i_start >= 0 {
                i_start += 1;
            }
            loop {
                let tick_pos_x = i_start as f32 * increment;
                if tick_pos_x > plot_rect.right() {
                    break;
                }
                let mut x_tick = Pos2::new(tick_pos_x, plot_rect.top());
                Self::transform_position(&mut x_tick, &plot_rect, &painter_rect);
                painter.line_segment(
                    [x_tick, x_tick - 5. * Vec2::Y],
                    Stroke::new(1.0, Color32::WHITE),
                );
                painter.line_segment(
                    [x_tick, x_tick - painter_rect.height() * Vec2::Y],
                    Stroke::new(0.5, Color32::from_white_alpha(5)),
                );
                painter.text(
                    x_tick + 15. * Vec2::Y,
                    Align2::CENTER_CENTER,
                    format!("{:.1}", tick_pos_x),
                    TextStyle::Monospace,
                    Color32::WHITE,
                );
                i_start += 1;
            }
            // Y-Axis ticks
            let mut i_start = (plot_rect.top() / increment) as i32;
            if i_start >= 0 {
                i_start += 1;
            }
            loop {
                let tick_pos_y = i_start as f32 * increment;
                if tick_pos_y > plot_rect.bottom() {
                    break;
                }
                let mut y_tick = Pos2::new(plot_rect.left(), tick_pos_y);
                Self::transform_position(&mut y_tick, &plot_rect, &painter_rect);
                painter.line_segment(
                    [y_tick, y_tick + 5. * Vec2::X],
                    Stroke::new(1.0, Color32::WHITE),
                );
                painter.line_segment(
                    [y_tick, y_tick + painter_rect.width() * Vec2::X],
                    Stroke::new(0.5, Color32::from_white_alpha(5)),
                );
                painter.text(
                    y_tick - 15. * Vec2::X,
                    Align2::CENTER_CENTER,
                    format!("{:.1}", tick_pos_y),
                    TextStyle::Monospace,
                    Color32::WHITE,
                );
                i_start += 1;
            }

            // Restrict painting to the painter area
            painter.set_clip_rect(painter_rect);

            // Call the function provided by the user to add the shapes.
            let mut plot_ui = PlotUi { shapes: Vec::new() };
            add_contents(&mut plot_ui);

            // Transform all shapes
            plot_ui.shapes.iter_mut().for_each(|shape| match shape {
                Shape::Noop => {}
                Shape::Vec(_) => {}
                Shape::Circle { center, .. } => {
                    Self::transform_position(center, &plot_rect, &painter_rect)
                }
                Shape::LineSegment { points, .. } => points
                    .iter_mut()
                    .for_each(|point| Self::transform_position(point, &plot_rect, &painter_rect)),
                Shape::Path { points, .. } => points
                    .iter_mut()
                    .for_each(|point| Self::transform_position(point, &plot_rect, &painter_rect)),
                Shape::Rect { .. } => {}
                Shape::Text { pos, .. } => Self::transform_position(pos, &plot_rect, &painter_rect),
                Shape::Mesh(_) => {}
            });

            // Show mouse position
            if show_cursor_pos {
                if let Some(mut mouse_pos) = ui.input().pointer.interact_pos() {
                    Self::transform_position(&mut mouse_pos, &painter_rect, &plot_rect);
                    painter.text(
                        painter_rect.right_bottom() + Vec2::new(-10., -10.),
                        Align2::RIGHT_BOTTOM,
                        format!("{:?}", mouse_pos),
                        TextStyle::Monospace,
                        Color32::WHITE,
                    );
                }
            }
            // Add shapes to the painter
            painter.extend(plot_ui.shapes);

            *first_run = false;

            response
        })
    }

    /// Transforms a position from one rectangle to another.
    fn transform_position(pos: &mut Pos2, plot_rect: &Rect, screen_rect: &Rect) {
        let from_x_range = plot_rect.x_range();
        let from_y_range = plot_rect.y_range();
        let to_x_range = screen_rect.x_range();
        let to_y_range = screen_rect.bottom_up_range();

        pos.x = remap(pos.x, from_x_range, to_x_range);
        pos.y = remap(pos.y, from_y_range, to_y_range);
    }
}
