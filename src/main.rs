#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{egui, App, Frame, NativeOptions};
use std::sync::Arc;

// 1. Define your application structure
struct MyApp {
    markdown_text: String,
    cache: egui_commonmark::CommonMarkCache,
}

// 2. Implement eframe::App trait
impl App for MyApp {
    // This method defines how your UI renders
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // Create a central panel, this is the most common layout method
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Markdown Editor");
            
            // Create horizontal layout, editor on the left, preview on the right
            ui.columns(2, |columns| {
                // Left editor area
                egui::ScrollArea::vertical().id_salt("editor").show(&mut columns[0], |ui| {
                    ui.label("Editor:");
                    egui::TextEdit::multiline(&mut self.markdown_text)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(20)
                        .show(ui);
                });
                
                // Right preview area
                egui::ScrollArea::vertical().id_salt("preview").show(&mut columns[1], |ui| {
                    ui.label("Preview:");
                    // Use egui_commonmark to render preview
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::same(10))  // Changed to integer 10
                        .show(ui, |ui| {
                            egui_commonmark::CommonMarkViewer::new().show(ui, &mut self.cache, &self.markdown_text);
                        });
                });
            });
        });
    }
}

// 3. Application entry point
fn main() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Markdown Editor",
        native_options,
        Box::new(|cc| {
            // Load Chinese fonts
            setup_chinese_fonts(&cc.egui_ctx);
            
            let app = MyApp {
                markdown_text: "# Welcome to Markdown Editor\n\nThis is a simple Markdown editor example.\n\n## Features\n\n- Real-time preview\n- Support for headings, lists, code blocks, etc.\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n\n*Italic* and **bold** are also supported.\n\n1. First item\n2. Second item\n3. Third item".to_owned(),
                cache: egui_commonmark::CommonMarkCache::default(),
            };
            Ok(Box::new(app) as Box<dyn App>)
        }),
    ).unwrap();
}

/// Load system Chinese fonts and set them to egui
fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // Try to load Chinese fonts from the system
    if let Some(chinese_font_data) = load_system_chinese_font() {
        fonts.font_data.insert("chinese".to_owned(), Arc::new(chinese_font_data));
        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
        fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
    }
    
    ctx.set_fonts(fonts);
}

/// Try to load Chinese fonts from the Windows system
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