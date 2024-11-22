use eframe::{
    egui::{CentralPanel, Key, Modifiers},
    NativeOptions,
};
use wapl::State;

fn main() {
    eframe::run_native(
        "WAPL GUI",
        NativeOptions::default(),
        Box::new(|_| Ok(Box::new(App::default()))),
    )
    .unwrap();
}

#[derive(Default)]
struct App {
    pub program: String,
    pub interpreter: State,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let mut removed = false;
        ctx.input(|input| {
            for event in &input.events {
                match event {
                    eframe::egui::Event::Paste(txt) | eframe::egui::Event::Text(txt) => {
                        self.program.push_str(txt);
                        for c in txt.chars() {
                            self.interpreter.cmd(c);
                        }
                    }
                    eframe::egui::Event::Key {
                        key: Key::Escape,
                        pressed: true,
                        ..
                    } => {
                        self.program.push(wapl::magic::ESC);
                        self.interpreter.cmd(wapl::magic::ESC)
                    }
                    eframe::egui::Event::Key {
                        key: Key::Backspace,
                        pressed: true,
                        ..
                    } => {
                        self.program.pop();
                        removed = true;
                    }
                    _ => {}
                }
            }
        });
        if removed {
            self.interpreter = State::default();
            for c in self.program.chars() {
                self.interpreter.cmd(c);
            }
        }
        CentralPanel::default().show(ctx, |ui| {
            ui.label("Program:");
            ui.label(&self.program);
            ui.separator();
            ui.label("Stack:");
            ui.label(format!("{:?}", self.interpreter.stack));
            ui.separator();
            ui.label("Names:");
            ui.label(format!("{:#?}", self.interpreter.names));
        });
    }
}
