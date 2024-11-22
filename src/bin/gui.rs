use std::{
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
    time::Duration,
};

use eframe::{
    egui::{CentralPanel, FontData, FontDefinitions, FontFamily, Key, Modifiers},
    NativeOptions,
};
use wapl::State;

#[no_mangle]
pub static OFL: &str = include_str!("./OFL.txt");

const FIRA: &[u8] = include_bytes!("./FiraCode-VariableFont_wght.ttf");

fn main() {
    eframe::run_native(
        "WAPL GUI",
        NativeOptions::default(),
        Box::new(|ctx| {
            let mut fonts = FontDefinitions::empty();
            fonts
                .font_data
                .insert("Fira Code".to_string(), FontData::from_static(FIRA));
            fonts
                .families
                .insert(FontFamily::Proportional, vec!["Fira Code".to_string()]);
            fonts
                .families
                .insert(FontFamily::Monospace, vec!["Fira Code".to_string()]);
            ctx.egui_ctx.set_fonts(fonts);
            Ok(Box::new(App::default()))
        }),
    )
    .unwrap();
}

struct App {
    pub program: String,
    pub interpreter: State,
    pub recv_intervening_states: Receiver<State>,
    pub send_cmds: Sender<char>,
}

impl Default for App {
    fn default() -> Self {
        let (cmd_sender, cmd_receiver) = std::sync::mpsc::channel();
        let (state_sender, state_receiver) = std::sync::mpsc::sync_channel(256);
        let interpreter = State {
            send_intervening_states: Some(state_sender),
            ..Default::default()
        };
        std::thread::spawn({
            let mut interpreter = interpreter.clone();
            move || {
                while let Ok(c) = cmd_receiver.recv() {
                    interpreter.cmd(c);
                }
            }
        });
        Self {
            program: Default::default(),
            interpreter,
            recv_intervening_states: state_receiver,
            send_cmds: cmd_sender,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let mut removed = false;
        ctx.input(|input| {
            for event in &input.events {
                match event {
                    eframe::egui::Event::Paste(txt) | eframe::egui::Event::Text(txt) => {
                        for char in txt.chars() {
                            self.send_cmds.send(char).unwrap();
                        }
                        self.program.push_str(txt);
                    }
                    eframe::egui::Event::Key {
                        key: Key::Escape,
                        pressed: true,
                        ..
                    } => {
                        self.program.push(wapl::magic::ESC);
                        self.send_cmds.send(wapl::magic::ESC).unwrap();
                    }
                    eframe::egui::Event::Key {
                        key: Key::Enter,
                        pressed: true,
                        ..
                    } => {
                        self.program.push('\n');
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
            let program = self.program.clone();
            *self = App {
                program: program.clone(),
                ..Default::default()
            };
            for char in program.chars() {
                self.send_cmds.send(char).unwrap();
            }
        }
        if let Some(new_state) = self.recv_intervening_states.try_iter().last() {
            self.interpreter = new_state;
        }
        ctx.request_repaint();
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
