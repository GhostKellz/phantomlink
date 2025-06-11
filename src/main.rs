mod phantomlink;
mod gui;
mod rnnoise;
mod audio;
mod scarlett;
mod config;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Phantomlink Mixer",
        options,
        Box::new(|_cc| Box::new(gui::PhantomlinkApp::default())),
    ).unwrap();
}
