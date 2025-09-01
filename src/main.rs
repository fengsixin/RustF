#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{egui, App, Frame, NativeOptions};
use std::sync::Arc;

// 1. 定义你的应用结构体
struct MyApp {
    markdown_text: String,
    cache: egui_commonmark::CommonMarkCache,
    // 添加滚动状态
    scroll_linked: bool, // 是否启用同步滚动
    // 保存编辑器的滚动位置
    editor_scroll_offset: egui::Vec2,
}

// 2. 实现 eframe::App trait
impl App for MyApp {
    // 这个方法定义了你的 UI 如何渲染
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // --- Ctrl+B 快捷键实现 ---
        // 1. 捕获键盘输入事件
        let ctrl_b_pressed = ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.ctrl);

        if ctrl_b_pressed {
            // 3. 获取编辑器状态和选区
            let editor_id = egui::Id::new("main_editor_id");
            if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
                if let Some(char_range) = state.cursor.char_range() {
                    let (primary_idx, secondary_idx) = (char_range.primary.index, char_range.secondary.index);

                    if primary_idx != secondary_idx {
                        // 正确的实现：将字符索引转换为字节索引
                        let (start_char, end_char) = (primary_idx.min(secondary_idx), primary_idx.max(secondary_idx));

                        // 1. 创建字符索引到字节索引的映射
                        let char_to_byte: Vec<usize> = self.markdown_text.char_indices().map(|(i, _)| i).collect();
                        
                        // 2. 安全地获取起始和结束的字节索引
                        if let Some(&start_byte) = char_to_byte.get(start_char) {
                            let end_byte = char_to_byte.get(end_char).copied().unwrap_or(self.markdown_text.len());

                            // 3. 使用正确的字节索引进行字符串操作
                            let new_text = format!("**{}**", &self.markdown_text[start_byte..end_byte]);
                            self.markdown_text.replace_range(start_byte..end_byte, &new_text);

                            // 4. 更新光标位置（egui 的 CCursor::new 需要字节索引）
                            let new_cursor_pos_byte = start_byte + new_text.len();
                            state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                                egui::text::CCursor::new(new_cursor_pos_byte),
                            )));
                            state.store(ctx, editor_id);
                        }
                    }
                }
            }
        }
        // --- 快捷键实现结束 ---

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
            // 先获取样式信息，避免借用冲突
            let stroke_color = ui.style().visuals.widgets.noninteractive.bg_stroke.color;
            
            // 使用列布局来创建双面板，它会默认填充可用空间
            ui.columns(2, |columns| {
                // 左侧编辑区域
                egui::Frame::new()
                    .inner_margin(egui::Margin { left: 10, right: 10, top: 10, bottom: 10 })
                    .stroke(egui::Stroke::new(1.0, stroke_color))
                    .show(&mut columns[0], |ui| {
                        ui.vertical(|ui| {
                            ui.label("编辑区:");
                            ui.add_space(5.0);
                            
                            // 关键：在这里获取编辑器的滚动响应
                            let editor_scroll_response = egui::ScrollArea::vertical()
                                .id_salt("editor_scroll_area") // 使用唯一 ID
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    // 创建一个包含行号和文本编辑器的布局
                                    ui.horizontal(|ui| {
                                        // 显示行号 - 新的、手动对齐的实现
                                        let monospace_font = egui::TextStyle::Monospace.resolve(ui.style());
                                        let row_height = ctx.fonts(|f| f.row_height(&monospace_font));
                                        // 使用数字 '0' 的宽度来估算，更准确
                                        let char_width = ctx.fonts(|f| f.glyph_width(&monospace_font, '0'));
                                        let line_count = self.markdown_text.lines().count().max(1);
                                        
                                        // 根据最大行号的位数动态计算行号区域的宽度
                                        let num_digits = line_count.to_string().len();
                                        let line_number_width = (num_digits as f32 * char_width) + 10.0; // 10.0 for padding

                                        egui::Frame::new()
                                            .inner_margin(egui::Margin { right: 10, ..Default::default() })
                                            .show(ui, |ui| {
                                                ui.style_mut().visuals.override_text_color = Some(egui::Color32::GRAY);
                                                
                                                // 手动为整个行号区域分配空间
                                                let total_height = row_height * line_count as f32;
                                                let (rect, _) = ui.allocate_exact_size(egui::vec2(line_number_width, total_height), egui::Sense::hover());

                                                // 手动绘制每个行号，确保行高与编辑器完全一致
                                                for i in 1..=line_count {
                                                    let line_y = rect.top() + (i - 1) as f32 * row_height;
                                                    let line_rect = egui::Rect::from_min_size(egui::pos2(rect.left(), line_y), egui::vec2(rect.width(), row_height));
                                                    
                                                    ui.painter().text(
                                                        line_rect.right_center(),
                                                        egui::Align2::RIGHT_CENTER,
                                                        i.to_string(),
                                                        monospace_font.clone(),
                                                        ui.style().visuals.text_color(),
                                                    );
                                                }
                                                ui.style_mut().visuals.override_text_color = None;
                                            });
                                        
                                        // 解决方案：将 TextEdit 控件包裹在一个 ui.vertical 容器中
                                        ui.vertical(|ui| {
                                            egui::TextEdit::multiline(&mut self.markdown_text)
                                                .id(egui::Id::new("main_editor_id")) // 2. 为 TextEdit 设置唯一 ID
                                                .code_editor()
                                                .desired_width(f32::INFINITY)
                                                .show(ui);
                                        });
                                    });
                                });

                            // 关键：在每一帧都更新滚动位置
                            self.editor_scroll_offset = editor_scroll_response.state.offset;
                        });
                    });
                
                // 右侧预览区域
                egui::Frame::new()
                    .inner_margin(egui::Margin { left: 10, right: 10, top: 10, bottom: 10 })
                    .stroke(egui::Stroke::new(1.0, stroke_color))
                    .show(&mut columns[1], |ui| {
                        ui.vertical(|ui| {
                            ui.label("预览区:");
                            ui.add_space(5.0);
                            
                            let mut preview_scroll_area = egui::ScrollArea::vertical()
                                .id_salt("preview_scroll_area") // 使用另一个唯一的 ID
                                .auto_shrink([false; 2]);
                            
                            // 关键：如果同步滚动被启用，则设置预览区域的滚动位置
                            if self.scroll_linked {
                                preview_scroll_area = preview_scroll_area.scroll_offset(self.editor_scroll_offset);
                            }
                            
                            preview_scroll_area.show(ui, |ui| {
                                egui::Frame::NONE
                                    .inner_margin(egui::Margin::same(10))
                                    .show(ui, |ui| {
                                        egui_commonmark::CommonMarkViewer::new().show(ui, &mut self.cache, &self.markdown_text);
                                    });
                            });
                        });
                    });
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
                editor_scroll_offset: egui::Vec2::ZERO,
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