// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = egui_template::TemplateApp::default();
    eframe::run_native(Box::new(app));
}
