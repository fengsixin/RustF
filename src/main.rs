use eframe::{egui, App, Frame, NativeOptions};
use std::sync::Arc;

// 1. 定义你的应用结构体
struct MyApp {
    name: String,
    age: u32,
}

// 2. 实现 eframe::App trait
impl App for MyApp {
    // 这个方法定义了你的 UI 如何渲染
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
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
    eframe::run_native(
        "Egui 示例",
        native_options,
        Box::new(|cc| {
            // 加载中文字体
            setup_chinese_fonts(&cc.egui_ctx);
            
            let app = MyApp {
                name: "世界".to_owned(),
                age: 25,
            };
            Ok(Box::new(app) as Box<dyn App>)
        }),
    ).unwrap();
}

/// 加载系统中文字体并设置到 egui
fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试从系统加载中文字体
    if let Some(chinese_font_data) = load_system_chinese_font() {
        fonts.font_data.insert("chinese".to_owned(), Arc::new(chinese_font_data));
        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
        fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
    }
    
    ctx.set_fonts(fonts);
}

/// 尝试从 Windows 系统加载中文字体
fn load_system_chinese_font() -> Option<egui::FontData> {
    let font_paths = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyhbd.ttc",
        r"C:\Windows\Fonts\simsun.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simkai.ttf",
        r"C:\Windows\Fonts\simfang.ttf",
    ];
    
    for font_path in &font_paths {
        if let Ok(font_data) = std::fs::read(font_path) {
            return Some(egui::FontData::from_owned(font_data));
        }
    }
    
    None
}