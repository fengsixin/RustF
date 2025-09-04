#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 声明新模块
mod app;
mod font_utils;

// 导入需要的项
use app::MyApp;
use eframe::{NativeOptions, App};

fn main() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "文档风格转换器",
        native_options,
        // 使用 app 模块中的构造函数来创建应用实例
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)) as Box<dyn App>)),
    ).unwrap();
}