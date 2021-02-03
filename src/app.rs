use std::f32::consts::TAU;

use eframe::{egui::*, epi};
use eplot::drawables::{Line, MarkerShape, Polygon, Quiver, Scatter};
use eplot::graph::{Graph, GraphMemory};

pub struct TemplateApp {
    start_time: std::time::Instant,
    graph_memory: GraphMemory,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            graph_memory: GraphMemory::default(),
        }
    }
}

impl epi::App for TemplateApp {
    fn update(&mut self, ctx: &CtxRef, _frame: &mut epi::Frame<'_>) {
        ctx.request_repaint();

        let start_time = self.start_time;

        Graph::new("TestPlot", &mut self.graph_memory)
            .x_axis_label("x-axis label")
            .y_axis_label("y-axis label") // Not working yet
            .x_range(-10f32..=10.)
            .axis_equal(true)
            .show(ctx, |plot_ui| {
                let t = std::time::Instant::now()
                    .duration_since(start_time)
                    .as_secs_f32();

                // Line
                let points: Vec<Pos2> = (-500..=500)
                    .map(|i| {
                        let x = i as f32 / 100.;
                        let y = 3. + (x * 2. + 10. * t).sin();
                        Pos2::new(x, y)
                    })
                    .collect();
                plot_ui.plot(Line::new(points).color(Color32::GREEN));

                // Scatter
                let points: Vec<Pos2> = (-15..=15)
                    .map(|i| {
                        let x = i as f32 / 3.;
                        let y = (3. * t).sin() * (x * 2. + 10.).sin();
                        Pos2::new(x, y)
                    })
                    .collect();
                plot_ui.plot(
                    Scatter::new(points)
                        .fill_color(Color32::RED)
                        .size(3.)
                        .stroke(Stroke::new(1., Color32::RED))
                        .shape(MarkerShape::Star)
                        .stems(true),
                );

                // Arrow polygon
                let points = vec![
                    Pos2::new(0., 1.) + Vec2::new(0., -4.),
                    Pos2::new(0., 2.) + Vec2::new(0., -4.),
                    Pos2::new(2., 0.) + Vec2::new(0., -4.),
                    Pos2::new(0., -2.) + Vec2::new(0., -4.),
                    Pos2::new(0., -1.) + Vec2::new(0., -4.),
                    Pos2::new(-3., -1.) + Vec2::new(0., -4.),
                    Pos2::new(-3., 1.) + Vec2::new(0., -4.),
                ];
                plot_ui.plot(
                    Polygon::new(points)
                        .fill_color(Color32::from_rgba_unmultiplied(255, 0, 255, 30))
                        .stroke(Stroke::new(
                            1.,
                            Color32::from_rgba_unmultiplied(255, 0, 255, 255),
                        )),
                );

                let markers_position = Pos2::new(7., -3.);
                let markers = [
                    MarkerShape::Circle,
                    MarkerShape::Triangle,
                    MarkerShape::Square,
                    MarkerShape::Plus,
                    MarkerShape::X,
                    MarkerShape::Star,
                ];
                let colors = [
                    Color32::WHITE,
                    Color32::LIGHT_BLUE,
                    Color32::BLUE,
                    Color32::GREEN,
                    Color32::YELLOW,
                    Color32::RED,
                ];
                markers
                    .iter()
                    .zip(colors.iter())
                    .enumerate()
                    .for_each(|(i, (marker, color))| {
                        let points = vec![
                            markers_position + Vec2::new(0., i as f32),
                            markers_position + Vec2::new(3., i as f32 + 0.5),
                        ];

                        plot_ui.plot(Line::new(points.clone()).color(color.linear_multiply(0.25)));
                        plot_ui.plot(
                            Scatter::new(points)
                                .shape(*marker)
                                .size(5.)
                                .fill_color(*color)
                                .stroke(Stroke::new(1., *color)),
                        );
                    });

                let center = Pos2::new(-12., 0.);
                let mut points = Vec::new();
                let mut directions = Vec::new();
                (-5..=5).for_each(|i| {
                    (-5..=5).for_each(|j| {
                        points.push(center + Vec2::new(i as f32, j as f32));
                        directions.push(Vec2::new(
                            ((i as f32) / 10. * TAU).sin(),
                            ((j as f32) / 10. * TAU).cos(),
                        ));
                    });
                });
                plot_ui.plot(Quiver::new(points, directions));
            });
    }

    fn name(&self) -> &str {
        "egui template"
    }
}

// ----------------------------------------------------------------------------
