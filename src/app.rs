use eframe::{egui::*, epi};
use eplot::{
    items::{Line, MarkerShape, Polygon, Quiver, Scatter, Text, YReference},
    plot::PlotCtx,
};

pub struct TemplateApp {
    start_time: std::time::Instant,
    plot_ctx: PlotCtx,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            plot_ctx: PlotCtx::default(),
        }
    }
}

impl epi::App for TemplateApp {
    fn update(&mut self, ctx: &CtxRef, _frame: &mut epi::Frame<'_>) {
        ctx.request_repaint();

        let Self {
            plot_ctx,
            start_time,
        } = self;

        let t = std::time::Instant::now()
            .duration_since(*start_time)
            .as_secs_f32();

        CentralPanel::default().show(ctx, |ui| {
            plot_ctx
                .plot("TestPlot")
                .title("eplot showcase")
                .size(vec2(1280., 720.))
                .x_axis_range(-10f32..=10.)
                .y_axis_range(-10f32..=10.)
                .show(ui, |plot_ui| {
                    // Line
                    [4., 3., 2., 1., 0.5]
                        .iter()
                        .enumerate()
                        .for_each(|(j, weight)| {
                            let points: Vec<Pos2> = (-200..=200)
                                .map(|i| {
                                    let x = i as f32 / 40.;
                                    let y = 3. + 0.5 * (j as f32) + (x * 2. + 10. * t).sin();
                                    pos2(x, y)
                                })
                                .collect();
                            plot_ui.add(Line::new(points).color(Color32::GREEN).weight(*weight));
                        });

                    // Line with filled area
                    let points: Vec<Pos2> = (-200..=200)
                        .map(|i| {
                            let x = 25. + i as f32 / 40.;
                            let y = 2. + (x * 2. + 10. * t).sin();
                            pos2(x, y)
                        })
                        .collect();
                    plot_ui.add(
                        Line::new(points).color(Color32::GOLD).area_fill(
                            YReference::Constant(0.),
                            Color32::GOLD.linear_multiply(0.1),
                        ),
                    );

                    // Scatter
                    let points: Vec<Pos2> = (-15..=15)
                        .map(|i| {
                            let x = i as f32 / 3.;
                            let y = (3. * t).sin() * (x * 2. + 10.).sin();
                            pos2(x, y)
                        })
                        .collect();
                    plot_ui.add(
                        Scatter::new(points)
                            .fill_color(Color32::RED)
                            .size(3.)
                            .stroke(Stroke::new(1., Color32::RED))
                            .shape(MarkerShape::Circle)
                            .stems(YReference::Constant(0.), Stroke::new(1., Color32::WHITE)),
                    );

                    // Arrow polygon
                    let points = vec![
                        pos2(0., 1.) + vec2(0., -4.),
                        pos2(0., 2.) + vec2(0., -4.),
                        pos2(2., 0.) + vec2(0., -4.),
                        pos2(0., -2.) + vec2(0., -4.),
                        pos2(0., -1.) + vec2(0., -4.),
                        pos2(-3., -1.) + vec2(0., -4.),
                        pos2(-3., 1.) + vec2(0., -4.),
                    ];
                    plot_ui.add(
                        Polygon::new(points)
                            .fill_color(Color32::from_rgba_unmultiplied(255, 0, 255, 30))
                            .stroke(Stroke::new(
                                1.,
                                Color32::from_rgba_unmultiplied(255, 0, 255, 255),
                            )),
                    );

                    // Scatter markers
                    let markers_position = pos2(7., -3.);
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
                    markers.iter().zip(colors.iter()).enumerate().for_each(
                        |(i, (marker, color))| {
                            let points = vec![
                                markers_position + vec2(0., i as f32),
                                markers_position + vec2(3., i as f32 + 0.5),
                                markers_position + vec2(6., i as f32),
                                markers_position + vec2(9., i as f32 + 0.5),
                            ];

                            plot_ui
                                .add(Line::new(points.clone()).color(color.linear_multiply(0.25)));
                            plot_ui.add(
                                Scatter::new(points)
                                    .shape(*marker)
                                    .size(5.)
                                    .fill_color(*color)
                                    .stroke(Stroke::new(1., *color)),
                            );
                        },
                    );

                    // Quiver
                    let center = pos2(-12., 0.);
                    let mut points = Vec::new();
                    let mut directions = Vec::new();
                    let maybe_mouse_pos = plot_ui
                        .plot_mouse_position()
                        .filter(|_| plot_ui.plot_hovered())
                        .filter(|pos| {
                            Rect::from_center_size(center, vec2(11., 11.)).contains(*pos)
                        });
                    (-5..=5).for_each(|i| {
                        (-5..=5).for_each(|j| {
                            points.push(center + vec2(i as f32, j as f32));

                            let ref_pos = maybe_mouse_pos.unwrap_or(center + 3.5 * Vec2::angled(t));
                            let dir = ref_pos - center - vec2(i as f32, j as f32);
                            directions.push(-1. / dir.length().max(1.) * dir.normalized());
                        });
                    });
                    plot_ui.add(Quiver::new(points, directions));

                    // Text
                    plot_ui.add(
                        Text::new(pos2(-12., -6.), "^ Move the cursor here ^")
                            .anchor(Align2::CENTER_TOP),
                    );
                });
        });
    }

    fn name(&self) -> &str {
        "eplot showcase"
    }
}

// ----------------------------------------------------------------------------
