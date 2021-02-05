use eframe::egui::*;
use std::{collections::HashMap, ops::RangeInclusive};

use super::items::PlotItem;

pub struct PlotUi<'p> {
    painter: &'p mut Painter,
    plot_to_screen: &'p dyn Fn(&Pos2) -> Pos2,
    mouse_position: Option<Pos2>,
    hovered: bool,
}

impl<'p> PlotUi<'p> {
    pub fn add<D: PlotItem>(&mut self, item: D) {
        item.paint(self.painter, self.plot_to_screen);
    }

    pub fn plot_mouse_position(&self) -> Option<Pos2> {
        self.mouse_position
    }

    pub fn plot_hovered(&self) -> bool {
        self.hovered
    }
}

pub struct Plot<'mem> {
    title: Option<String>,
    axis_equal: bool,
    show_cursor_pos: bool,
    x_axis_label: Option<String>,
    y_axis_label: Option<String>,
    memory: &'mem mut PlotMemory,
    size: Vec2,
}

pub(crate) struct PlotMemory {
    last_drag_pos: Option<Pos2>,
    plot_rect: Rect,
    first_run: bool,
}

impl Default for PlotMemory {
    fn default() -> Self {
        Self {
            last_drag_pos: None,
            plot_rect: Rect::from_center_size(Pos2::ZERO, vec2(10., 10.)),
            first_run: true,
        }
    }
}

#[derive(Default)]
pub struct PlotCtx {
    pub(crate) memory: HashMap<Id, PlotMemory>,
}

impl PlotCtx {
    pub fn plot(&mut self, label: impl Into<String>) -> Plot {
        let id = Id::new(label.into());
        let mem = self.memory.entry(id).or_default();
        Plot::new(mem)
    }
}

impl<'mem> Plot<'mem> {
    fn new(memory: &'mem mut PlotMemory) -> Self {
        Self {
            title: None,
            axis_equal: false,
            show_cursor_pos: true,
            x_axis_label: None,
            y_axis_label: None,
            memory,
            size: vec2(100., 100.),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
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
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut PlotUi) -> R) -> Response {
        let Self {
            axis_equal,
            show_cursor_pos,
            x_axis_label,
            y_axis_label,
            memory,
            title,
            size,
        } = self;
        Resize::default().default_size(size).show(ui, |ui| {
            let PlotMemory {
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
            let mut top_margin = 10.;
            if title.is_some() {
                top_margin += 10.
            }
            if x_axis_label.is_some() {
                bottom_margin += 10.
            }
            if y_axis_label.is_some() {
                left_margin += 10.
            }
            let full_rect = response.rect;
            let painter_rect = Rect::from_min_max(
                full_rect.min + vec2(left_margin, top_margin),
                full_rect.max - vec2(right_margin, bottom_margin),
            );
            painter.rect(
                painter_rect,
                0.,
                Color32::from_gray(10),
                Stroke::new(1.0, Color32::from_white_alpha(150)),
            );

            if let Some(title) = title {
                painter.text(
                    painter_rect.center_top() - vec2(0., 2.),
                    Align2::CENTER_BOTTOM,
                    title,
                    TextStyle::Monospace,
                    Color32::WHITE,
                );
            }

            if let Some(label) = x_axis_label {
                painter.text(
                    painter_rect.center_bottom() + vec2(0., 25.),
                    Align2::CENTER_TOP,
                    label,
                    TextStyle::Monospace,
                    Color32::WHITE,
                );
            }

            // TODO: Y-axis label.

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
                let pos_tf = Self::transform_position(&pos, &painter_rect, &plot_rect);
                if let Some(last_pos) = last_drag_pos {
                    ui.output().cursor_icon = CursorIcon::Grabbing;
                    let last_pos_tf =
                        Self::transform_position(&last_pos, &painter_rect, &plot_rect);
                    let delta = last_pos_tf - pos_tf;
                    *plot_rect = plot_rect.translate(delta);
                }
                *last_drag_pos = Some(pos);
            } else {
                *last_drag_pos = None;
            }

            // Zooming
            let scrolled = ui.input().scroll_delta.y.max(-10.0).min(10.0);
            if let Some(mouse_pos) = ui
                .input()
                .pointer
                .interact_pos()
                .filter(|pos| painter_rect.contains(*pos))
            {
                if scrolled != 0. {
                    let left_distance = (mouse_pos.x - painter_rect.left()) / painter_rect.width();
                    let right_distance = 1. - left_distance;
                    let top_distance = (mouse_pos.y - painter_rect.top()) / painter_rect.height();
                    let bottom_distance = 1. - top_distance;
                    *plot_rect = Rect::from_min_max(
                        pos2(
                            plot_rect.min.x + 0.01 * scrolled * plot_rect.width() * left_distance,
                            plot_rect.min.y
                                + 0.01 * scrolled * plot_rect.height() * bottom_distance,
                        ),
                        pos2(
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
                let x_tick = pos2(tick_pos_x, plot_rect.top());
                let x_tick = Self::transform_position(&x_tick, &plot_rect, &painter_rect);
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
                let y_tick = pos2(plot_rect.left(), tick_pos_y);
                let y_tick = Self::transform_position(&y_tick, &plot_rect, &painter_rect);
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

            let plot_to_screen =
                |pos: &Pos2| -> Pos2 { Self::transform_position(pos, &plot_rect, &painter_rect) };
            let screen_to_plot =
                |pos: &Pos2| -> Pos2 { Self::transform_position(pos, &painter_rect, &plot_rect) };

            // Call the function provided by the user to add the shapes.
            let mut plot_ui = PlotUi {
                painter: &mut painter,
                plot_to_screen: &plot_to_screen,
                mouse_position: ui
                    .input()
                    .pointer
                    .interact_pos()
                    .map(|pos| screen_to_plot(&pos)),
                hovered: ui
                    .input()
                    .pointer
                    .interact_pos()
                    .filter(|pos| painter_rect.contains(*pos))
                    .is_some(),
            };
            add_contents(&mut plot_ui);

            // Show mouse position
            if show_cursor_pos {
                if let Some(mouse_pos) = ui
                    .input()
                    .pointer
                    .interact_pos()
                    .filter(|pos| painter_rect.contains(*pos))
                {
                    let mouse_pos = screen_to_plot(&mouse_pos);
                    painter.text(
                        painter_rect.right_bottom() + vec2(-10., -10.),
                        Align2::RIGHT_BOTTOM,
                        format!("{:?}", mouse_pos),
                        TextStyle::Monospace,
                        Color32::WHITE,
                    );
                }
            }

            *first_run = false;

            response
        })
    }

    /// Transforms a position from one rectangle to another.
    fn transform_position(pos: &Pos2, plot_rect: &Rect, screen_rect: &Rect) -> Pos2 {
        let from_x_range = plot_rect.x_range();
        let from_y_range = plot_rect.y_range();
        let to_x_range = screen_rect.x_range();
        let to_y_range = screen_rect.bottom_up_range();

        pos2(
            remap(pos.x, from_x_range, to_x_range),
            remap(pos.y, from_y_range, to_y_range),
        )
    }
}
