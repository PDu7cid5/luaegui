use eframe::egui;
use egui::Context;
use mlua::Function;

const LUA_CODE: &str = include_str!("script.lua");

fn main() -> eframe::Result {
    let lua = mlua::Lua::new();
    luaegui::register_egui_bindings(&lua).unwrap();

    let app = App {
        lua,
        code: LUA_CODE.to_string(),
        script_time: std::time::Duration::ZERO,
    };

    let options = eframe::NativeOptions::default();
    eframe::run_native("basic_example", options, Box::new(|_cc| Ok(Box::new(app))))
}

struct App {
    script_time: std::time::Duration,
    lua: mlua::Lua,
    code: String,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let ctx = ctx.clone();

        egui::CentralPanel::default().show(&ctx, |ui| {
            use egui::*;

            if ui.button("run").clicked() {
                if let Err(e) = self.lua.load(&self.code).exec() {
                    eprintln!("lua load error: {e:?}");
                }
            }
            if !self.lua.globals().contains_key("gui_run").unwrap() {
                ui.colored_label(Color32::RED, "gui_run fn is not defined");
            }
            ui.add(
                egui::TextEdit::multiline(&mut self.code)
                .code_editor()
                .desired_width(400.0),
            );
            ui.horizontal(|ui| {
                ui.label("script execution time (micros): ");
                ui.label(format!("{}", self.script_time.as_micros()));
            });

        });
        let start = std::time::Instant::now();
        if let Ok(f) = self.lua.globals().get::<Function>("gui_run") {
            let c = self.lua.create_any_userdata(ctx).unwrap();
            let _: () = f.call(c).unwrap();
        }
        self.script_time = start.elapsed();
    }
}