#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{egui, App, Frame, NativeOptions};
use std::sync::Arc;

// 1. 定义你的应用结构体
struct MyApp {
    markdown_text: String,
    cache: egui_commonmark::CommonMarkCache,
    // 添加滚动状态
    scroll_linked: bool, // 是否启用同步滚动
}

// 2. 实现 eframe::App trait
impl App for MyApp {
    // 这个方法定义了你的 UI 如何渲染
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // 创建一个共享的滚动 ID
        let scroll_id = egui::Id::new("shared_scroll_area");
        
        // 在顶部创建菜单栏
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::containers::menu::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("打开").clicked() {
                        ui.close();
                        self.load_file();
                    }
                    if ui.button("保存").clicked() {
                        ui.close();
                        self.save_file();
                    }
                });
                
                // 添加同步滚动选项
                ui.menu_button("视图", |ui| {
                    ui.checkbox(&mut self.scroll_linked, "同步滚动");
                });
            });
        });
        
        // 在菜单栏下方创建主内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            // 创建水平布局，左侧编辑器，右侧预览
            ui.columns(2, |columns| {
                if self.scroll_linked {
                    // 使用共享的 ScrollArea 实现同步滚动
                    // 左侧编辑区域
                    egui::ScrollArea::vertical()
                        .id_salt(scroll_id)
                        .show(&mut columns[0], |ui| {
                            ui.label("编辑器:");
                            egui::TextEdit::multiline(&mut self.markdown_text)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(20)
                                .show(ui)
                        });
                    
                    // 右侧预览区域
                    egui::ScrollArea::vertical()
                        .id_salt(scroll_id)
                        .show(&mut columns[1], |ui| {
                            ui.label("预览:");
                            // 使用 egui_commonmark 来渲染预览
                            egui::Frame::NONE
                                .inner_margin(egui::Margin::same(10))
                                .show(ui, |ui| {
                                    egui_commonmark::CommonMarkViewer::new().show(ui, &mut self.cache, &self.markdown_text);
                                })
                        });
                } else {
                    // 独立滚动
                    // 左侧编辑区域
                    egui::ScrollArea::vertical()
                        .id_salt("editor_scroll")
                        .show(&mut columns[0], |ui| {
                            ui.label("编辑器:");
                            egui::TextEdit::multiline(&mut self.markdown_text)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(20)
                                .show(ui)
                        });
                    
                    // 右侧预览区域
                    egui::ScrollArea::vertical()
                        .id_salt("preview_scroll")
                        .show(&mut columns[1], |ui| {
                            ui.label("预览:");
                            // 使用 egui_commonmark 来渲染预览
                            egui::Frame::NONE
                                .inner_margin(egui::Margin::same(10))
                                .show(ui, |ui| {
                                    egui_commonmark::CommonMarkViewer::new().show(ui, &mut self.cache, &self.markdown_text);
                                })
                        });
                }
            });
        });
    }
}

impl MyApp {
    fn load_file(&mut self) {
        let handle = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .add_filter("Text", &["txt"])
            .pick_file();
            
        if let Some(path) = handle {
            if let Ok(content) = std::fs::read_to_string(path) {
                self.markdown_text = content;
            }
        }
    }
    
    fn save_file(&self) {
        let handle = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .add_filter("Text", &["txt"])
            .save_file();
            
        if let Some(path) = handle {
            let _ = std::fs::write(path, &self.markdown_text);
        }
    }
}

// 3. 应用入口点
fn main() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Markdown 编辑器",
        native_options,
        Box::new(|cc| {
            // 加载中文字体
            setup_chinese_fonts(&cc.egui_ctx);
            
            let app = MyApp {
                markdown_text: "# 欢迎使用 Markdown 编辑器\n\n这是一个简单的 Markdown 编辑器示例。\n\n## 功能\n\n- 实时预览\n- 支持标题、列表、代码块等\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n\n*斜体* 和 **粗体** 也支持。\n\n1. 第一项\n2. 第二项\n3. 第三项\n\n".to_owned() + 
                &"# 更多内容\n\n这是更多的内容，用于测试滚动同步功能。\n\n".repeat(50),
                cache: egui_commonmark::CommonMarkCache::default(),
                scroll_linked: true, // 默认启用同步滚动
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