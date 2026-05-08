use egui::{Color32, Pos2, Rect, CornerRadius};
use egui_plot::{Line, Plot, PlotPoints};

const MAX_POINTS: usize = 60;

pub struct SpeedChart {
    pub up_data: Vec<f64>,
    pub down_data: Vec<f64>,
    pub max_y: f64,
}

impl SpeedChart {
    pub fn new() -> Self {
        Self {
            up_data: Vec::with_capacity(MAX_POINTS),
            down_data: Vec::with_capacity(MAX_POINTS),
            max_y: 10.0,
        }
    }

    pub fn push(&mut self, up_kbps: f64, down_kbps: f64) {
        self.up_data.push(up_kbps);
        self.down_data.push(down_kbps);
        if self.up_data.len() > MAX_POINTS {
            self.up_data.remove(0);
        }
        if self.down_data.len() > MAX_POINTS {
            self.down_data.remove(0);
        }
        let peak = up_kbps.max(down_kbps).max(10.0);
        self.max_y = (peak * 1.3).ceil();
    }

    pub fn ui(&self, ui: &mut egui::Ui, up_color: Color32, down_color: Color32) {
        let up_pts: PlotPoints = self.up_data.iter().enumerate()
            .map(|(i, &v)| [i as f64, v]).collect();
        let down_pts: PlotPoints = self.down_data.iter().enumerate()
            .map(|(i, &v)| [i as f64, v]).collect();

        Plot::new("speed_chart")
            .view_aspect(2.5)
            .show_axes([false, true])
            .show_grid([true, true])
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .show(ui, |plot_ui| {
                if !self.up_data.is_empty() {
                    plot_ui.line(Line::new(up_pts).color(up_color).width(2.0).name("upload"));
                }
                if !self.down_data.is_empty() {
                    plot_ui.line(Line::new(down_pts).color(down_color).width(2.0).name("download"));
                }
            });
    }
}

pub fn draw_gradient_header(
    painter: &egui::Painter,
    rect: Rect,
    from: Color32,
    to: Color32,
) {
    // Top rounded corners
    painter.rect_filled(rect, CornerRadius { nw: 12, ne: 12, sw: 0, se: 0 }, from);

    let n = 32;
    let step = rect.height() / n as f32;
    for i in 1..n {
        let t = i as f32 / (n - 1) as f32;
        let color = lerp_color(from, to, t);
        let y0 = rect.top() + step * i as f32;
        let row = Rect::from_min_max(
            Pos2::new(rect.left(), y0),
            Pos2::new(rect.right(), y0 + step),
        );
        painter.rect_filled(row, CornerRadius::ZERO, color);
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}
