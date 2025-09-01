use eframe::{egui, App, Frame, NativeOptions};

// 1. 定义你的应用结构体
struct MyApp {
    name: String,
    age: u32,
}

// 2. 实现 eframe::App trait
impl App for MyApp {
    // 这个方法定义了你的 UI 如何渲染
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        // 创建一个中央面板，这是最常见的布局方式
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("我的第一个 Egui 应用");

            ui.horizontal(|ui| {
                ui.label("你的名字:");
                ui.text_edit_singleline(&mut self.name);
            });

            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("你的年龄"));

            if ui.button("点击我").clicked() {
                // 按钮被点击后执行的逻辑
                println!("你好，{}！你 {} 岁了。", self.name, self.age);
            }
        });
    }
}

// 3. 应用入口点
fn main() {
    let native_options = NativeOptions::default();
    let app = MyApp {
        name: "世界".to_owned(),
        age: 25,
    };

    eframe::run_native(
        "Egui 示例",
        native_options,
        Box::new(|cc| Box::new(app)),
    ).unwrap();
}