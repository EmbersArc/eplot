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

#[derive(Clone, Copy)]
enum AxisScaling {
    Linear,
    Logarithmic,
}

impl Default for AxisScaling {
    fn default() -> Self {
        Self::Linear
    }
}

#[derive(Clone, Copy)]
struct AxisRange {
    start: f32,
    end: f32,
    scaling: AxisScaling,
}

impl Default for AxisRange {
    fn default() -> Self {
        Self {
            start: -10.,
            end: -10.,
            scaling: AxisScaling::Linear,
        }
    }
}

impl AxisRange {
    fn new(range: RangeInclusive<f32>) -> Self {
        Self {
            start: *range.start(),
            end: *range.end(),
            scaling: AxisScaling::Linear,
        }
    }

    fn extent(&self) -> f32 {
        self.end - self.start
    }

    fn middle(&self) -> f32 {
        (self.start + self.end) / 2.
    }

    fn translate(&mut self, delta: f32) {
        self.start += delta;
        self.end += delta;
    }

    fn zoom(&mut self, amount: f32, center: f32) {
        self.start -= amount * center * self.extent();
        self.end += amount * (1. - center) * self.extent();
    }

    fn pixel_to_axis(&self, pixel_range: RangeInclusive<f32>, pixel: f32, flip: bool) -> f32 {
        let pixel_tf = if flip {
            remap(pixel, pixel_range.clone(), self.end..=self.start)
        } else {
            remap(pixel, pixel_range.clone(), self.start..=self.end)
        };
        match self.scaling {
            AxisScaling::Linear => pixel_tf,
            AxisScaling::Logarithmic => {
                let den = (self.end / self.start).log10();
                let t = (pixel_tf - self.start) / self.extent();
                (t * den).powi(10) * self.start
            }
        }
    }

    fn axis_to_pixel(&self, pixel_range: RangeInclusive<f32>, axis_pos: f32, flip: bool) -> f32 {
        match self.scaling {
            AxisScaling::Linear => {
                if flip {
                    remap(axis_pos, self.end..=self.start, pixel_range)
                } else {
                    remap(axis_pos, self.start..=self.end, pixel_range)
                }
            }
            AxisScaling::Logarithmic => {
                let t = (axis_pos / self.start).log(self.end / self.start);
                if flip {
                    lerp(self.end..=self.start, t)
                } else {
                    lerp(self.start..=self.end, t)
                }
            }
        }
    }
}

pub struct Axis {
    label: String,
    range: AxisRange,
}

impl Default for Axis {
    fn default() -> Self {
        Self {
            label: "".to_string(),
            range: AxisRange::new((-10.)..=10.),
        }
    }
}

pub struct Plot<'mem> {
    title: Option<String>,
    show_cursor_pos: bool,
    memory: &'mem mut PlotMemory,
    size: Vec2,
    x_axis: Axis,
    y_axis: Axis,
}

pub(crate) struct PlotMemory {
    last_drag_pos: Option<Pos2>,
    x_axis_range: AxisRange,
    y_axis_range: AxisRange,
}

impl Default for PlotMemory {
    fn default() -> Self {
        Self {
            last_drag_pos: None,
            x_axis_range: AxisRange::new((-10.)..=10.),
            y_axis_range: AxisRange::new((-10.)..=10.),
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
        let memory = self.memory.entry(id).or_default();
        Plot::new_with_memory(memory)
    }
}

impl<'mem> Plot<'mem> {
    fn new_with_memory(memory: &'mem mut PlotMemory) -> Self {
        Self {
            title: None,
            show_cursor_pos: true,
            memory,
            size: vec2(100., 100.),
            x_axis: Axis::default(),
            y_axis: Axis::default(),
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

    pub fn x_axis_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.x_axis.range = AxisRange::new(range);
        self
    }

    pub fn y_axis_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.y_axis.range = AxisRange::new(range);
        self
    }

    /// Show the cursor position in the bottol left corner.
    pub fn show_cursor_position(mut self, on: bool) -> Self {
        self.show_cursor_pos = on;
        self
    }

    /// Draw the plot. Takes a closure where contents can be added to the plot.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut PlotUi) -> R) -> Response {
        let Self {
            show_cursor_pos,
            memory,
            title,
            size,
            mut x_axis,
            mut y_axis,
        } = self;

        Resize::default().default_size(size).show(ui, |ui| {
            let PlotMemory {
                last_drag_pos,
                x_axis_range,
                y_axis_range,
            } = memory;

            x_axis.range = *x_axis_range;
            y_axis.range = *y_axis_range;

            let (response, mut painter) =
                ui.allocate_painter(ui.available_size_before_wrap_finite(), Sense::drag());

            // Plotting area
            let left_margin = 40.;
            let right_margin = 10.;
            let mut bottom_margin = 40.;
            let mut top_margin = 10.;
            if title.is_some() {
                top_margin += 10.
            }
            if !x_axis.label.is_empty() {
                bottom_margin += 10.
            }

            // The full plot rectangle, including title, axes, and their labels.
            let full_rect = response.rect;

            // The rectangle that contains the plot items.
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

            if !x_axis.label.is_empty() {
                painter.text(
                    painter_rect.center_bottom() + vec2(0., 25.),
                    Align2::CENTER_TOP,
                    x_axis.label.clone(),
                    TextStyle::Monospace,
                    Color32::WHITE,
                );
            }

            // TODO: Y-axis label.

            // Adjust the axes so that the aspect ratio is equal.
            let painter_height = painter_rect.height();
            let painter_width = painter_rect.width();
            let plot_width = x_axis.range.extent();
            let plot_height = y_axis.range.extent();
            let max_half_extent = plot_width.max(plot_height) / 2.;
            let painter_ratio = painter_height / painter_width;
            if painter_ratio > 1. {
                let x_center = x_axis.range.middle();
                x_axis.range.start = x_center - max_half_extent / painter_ratio;
                x_axis.range.end = x_center + max_half_extent / painter_ratio;
            } else {
                let y_center = y_axis.range.middle();
                y_axis.range.start = y_center - max_half_extent * painter_ratio;
                y_axis.range.end = y_center + max_half_extent * painter_ratio;
            }

            // Dragging
            let new_drag_pos = response.interact_pointer_pos();
            if let Some(pos) = new_drag_pos {
                let x_tf = x_axis
                    .range
                    .pixel_to_axis(painter_rect.x_range(), pos.x, false);
                let y_tf = y_axis
                    .range
                    .pixel_to_axis(painter_rect.y_range(), pos.y, true);
                let pos_tf = Pos2::new(x_tf, y_tf);

                if let Some(last_pos) = last_drag_pos {
                    ui.output().cursor_icon = CursorIcon::Grabbing;
                    let x_tf =
                        x_axis
                            .range
                            .pixel_to_axis(painter_rect.x_range(), last_pos.x, false);
                    let y_tf = y_axis
                        .range
                        .pixel_to_axis(painter_rect.y_range(), last_pos.y, true);
                    let last_pos_tf = Pos2::new(x_tf, y_tf);

                    let delta = last_pos_tf - pos_tf;
                    x_axis.range.translate(delta.x);
                    y_axis.range.translate(delta.y);
                }
                *last_drag_pos = Some(pos);
            } else {
                *last_drag_pos = None;
            }

            // Zooming
            let scrolled = ui.input().scroll_delta.y.clamp(-10., 10.);
            if let Some(mouse_pos) = ui
                .input()
                .pointer
                .interact_pos()
                .filter(|pos| painter_rect.contains(*pos))
            {
                if scrolled != 0. {
                    let left_distance = (mouse_pos.x - painter_rect.left()) / painter_rect.width();
                    let bottom_distance =
                        (painter_rect.bottom() - mouse_pos.y) / painter_rect.height();
                    let zoom_factor = -0.01 * scrolled;
                    x_axis.range.zoom(zoom_factor, left_distance);
                    y_axis.range.zoom(zoom_factor, bottom_distance);
                }
            }

            let plot_to_screen = |pos: &Pos2| -> Pos2 {
                Self::plot_to_pixels(pos, &x_axis.range, &y_axis.range, &painter_rect)
            };
            let screen_to_plot = |pos: &Pos2| -> Pos2 {
                Self::pixels_to_plot(pos, &x_axis.range, &y_axis.range, &painter_rect)
            };

            // Ticks and tick labels
            let ticks_on_smaller_axis = 5; // The lower limit of ticks on the smaller axis.
            let smaller_axis_size = x_axis.range.extent().min(y_axis.range.extent());
            let rough_increment = smaller_axis_size / ticks_on_smaller_axis as f32;
            let increment = emath::smart_aim::best_in_range_f64(
                (rough_increment * 0.5) as f64,
                (rough_increment * 1.5) as f64,
            ) as f32;

            // X-Axis ticks
            let mut i_start = (x_axis.range.start / increment) as i32;
            if i_start >= 0 {
                i_start += 1;
            }
            loop {
                let tick_pos_x = i_start as f32 * increment;
                if tick_pos_x > x_axis.range.end {
                    break;
                }
                let x_tick = plot_to_screen(&pos2(tick_pos_x, y_axis.range.start));
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
            let mut i_start = (y_axis.range.start / increment) as i32;
            if i_start >= 0 {
                i_start += 1;
            }
            loop {
                let tick_pos_y = i_start as f32 * increment;
                if tick_pos_y > painter_rect.bottom() {
                    break;
                }
                let y_tick = plot_to_screen(&pos2(x_axis.range.start, tick_pos_y));
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

            *x_axis_range = x_axis.range;
            *y_axis_range = y_axis.range;

            response
        })
    }

    fn pixels_to_plot(
        pixel_pos: &Pos2,
        x_pixel_range: &AxisRange,
        y_pixel_range: &AxisRange,
        plot_rect: &Rect,
    ) -> Pos2 {
        let x_tf = x_pixel_range.pixel_to_axis(plot_rect.x_range(), pixel_pos.x, false);
        let y_tf = y_pixel_range.pixel_to_axis(plot_rect.y_range(), pixel_pos.y, true);
        pos2(x_tf, y_tf)
    }

    fn plot_to_pixels(
        plot_pos: &Pos2,
        x_plot_range: &AxisRange,
        y_plot_range: &AxisRange,
        plot_rect: &Rect,
    ) -> Pos2 {
        let x_tf = x_plot_range.axis_to_pixel(plot_rect.x_range(), plot_pos.x, false);
        let y_tf = y_plot_range.axis_to_pixel(plot_rect.y_range(), plot_pos.y, true);
        pos2(x_tf, y_tf)
    }
}
