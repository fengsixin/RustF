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
        let mut request_repaint = false;

        if ctx.input(|i| i.key_pressed(egui::Key::B) && i.modifiers.ctrl) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::B));
            self.apply_formatting_to_selection(ctx, "**", "**");
            request_repaint = true;
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::I) && i.modifiers.ctrl) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::I));
            self.apply_formatting_to_selection(ctx, "*", "*");
            request_repaint = true;
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::U) && i.modifiers.ctrl) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::U));
            self.apply_formatting_to_selection(ctx, "[", "]{.underline}");
            request_repaint = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::H) && i.modifiers.ctrl) {
            ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::H));
            self.apply_formatting_to_selection(ctx, "{{", "}}");
            request_repaint = true;
        }

        if request_repaint {
            ctx.request_repaint();
        }

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
                    // --- 新增代码开始 ---
                    if ui.button("合并文件").clicked() {
                        // 点击后关闭菜单
                        ui.close();
                        // 调用我们将要创建的合并文件逻辑
                        self.merge_files();
                    }
                    // --- 新增代码结束 ---
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
                                    // --- 最终方案：分离渲染，但统一布局信息源 ---

                                    // 1. 计算行号区域的宽度
                                    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                                    let char_width = ui.fonts(|f| f.glyph_width(&font_id, '0'));
                                    let line_count = self.markdown_text.lines().count().max(1);
                                    let num_digits = line_count.to_string().len();
                                    // 留出一些额外的空间以避免拥挤
                                    let line_number_width = (num_digits as f32 * char_width) + 15.0;

                                    // 2. 创建布局“真理之源” (Galley)
                                    let available_width = ui.available_width() - line_number_width;
                                    // 使用 LayoutJob 来避免克隆字符串
                                    let galley = {
                                        let mut job = egui::text::LayoutJob::default();
                                        job.append(&self.markdown_text, 0.0, egui::TextFormat::simple(font_id.clone(), ui.style().visuals.text_color()));
                                        job.wrap.max_width = available_width; // 手动设置换行宽度
                                        ui.fonts(|f| f.layout_job(job))
                                    };

                                    ui.horizontal(|ui| {
                                // 3. 根据 Galley 绘制行号
                                let line_number_painter = |ui: &mut egui::Ui| {
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(line_number_width, galley.size().y),
                                        egui::Sense::hover(),
                                    );

                                    // 使用 logical_line 来跟踪逻辑行号
                                    let mut logical_line = 1;
                                    // 跟踪上一个字符的位置，用于检测换行
                                    let mut last_char_pos = egui::Pos2::new(0.0, 0.0);
                                    for (i, row) in galley.rows.iter().enumerate() {
                                        // 检查是否是新的一行（通过y坐标变化判断）
                                        if i == 0 || row.rect().min.y > last_char_pos.y + 1.0 {
                                            let line_y = rect.min.y + row.rect().min.y;
                                            let line_rect = egui::Rect::from_min_size(
                                                egui::pos2(rect.left(), line_y),
                                                egui::vec2(rect.width(), row.rect().height()),
                                            );

                                            ui.painter().text(
                                                line_rect.right_center(),
                                                egui::Align2::RIGHT_CENTER,
                                                logical_line.to_string(), // 绘制正确的逻辑行号
                                                font_id.clone(),
                                                egui::Color32::GRAY,
                                            );
                                            
                                            // 增加逻辑行号
                                            logical_line += 1;
                                        }
                                        // 更新上一个字符的位置
                                        last_char_pos = row.rect().min;
                                    }
                                };
                                // 使用一个辅助UI来绘制行号，确保布局正确
                                ui.scope(line_number_painter);

                                        // 4. 绘制标准的 TextEdit
                                        let editor_response = egui::TextEdit::multiline(&mut self.markdown_text)
                                            .id(egui::Id::new("main_editor_id"))
                                            .code_editor()
                                            .desired_width(available_width)
                                            .desired_rows(1) // 阻止 TextEdit 自身请求额外空间
                                            .show(ui)
                                            .response;
                                        
                                        // 将编辑器的响应与行号区域的响应结合（可选，但有助于整体UI行为）
                                        editor_response
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
    
    /// 应用格式化到选中的文本
    fn apply_formatting_to_selection(&mut self, ctx: &egui::Context, prefix: &str, suffix: &str) {
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
                        // 使用 .get() 来安全地处理 end_char，即使它等于 char_to_byte.len()
                        // 当 .get(end_char) 返回 None 时（即选中到末尾），回退到字符串的总字节长度
                        let end_byte = char_to_byte.get(end_char).copied().unwrap_or(self.markdown_text.len());

                        // 3. 使用正确的字节索引进行字符串操作
                        let new_text = format!("{}{}{}", prefix, &self.markdown_text[start_byte..end_byte], suffix);
                        self.markdown_text.replace_range(start_byte..end_byte, &new_text);

                        // 4. 更新光标位置（egui 的 CCursor::new 需要字符索引）
                        // 首先，计算新插入的文本片段有多少个字符
                        let new_text_char_len = new_text.chars().count();
                        
                        // 新的光标字符索引 = 开始位置的字符索引 + 新文本的字符长度
                        let new_cursor_pos_char = start_char + new_text_char_len;
                        
                        // 使用正确的字符索引来创建 CCursor
                        state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(new_cursor_pos_char),
                        )));
                        state.store(ctx, editor_id);
                    }
                }
            }
        }
    }
    
    /// 处理合并多个 Markdown 文件的逻辑
    fn merge_files(&mut self) {
        // 1. 打开一个可以选择"多个"文件的对话框
        let files = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .add_filter("Text", &["txt"])
            .pick_files(); // <--- 关键：使用 pick_files() 而不是 pick_file()

        // 2. 检查用户是否选择了文件
        if let Some(paths) = files {
            // 如果用户只选了一个或没选，就不做任何事
            if paths.len() <= 1 {
                return;
            }

            let mut combined_content = String::new();

            // 3. 遍历所有选择的文件路径
            for (index, path) in paths.iter().enumerate() {
                // 尝试读取每个文件的内容
                if let Ok(content) = std::fs::read_to_string(path) {
                    // 将文件内容附加到合并字符串中
                    combined_content.push_str(&content);

                    // 4. 在文件之间添加一个清晰的分隔符，但不在最后一个文件后面添加
                    if index < paths.len() - 1 {
                        // 使用换行符和 Markdown 的水平线作为分隔
                        // 这可以防止文件内容黏在一起，并在视觉上区分
                        combined_content.push_str("\n\n---\n\n");
                    }
                }
                // 如果某个文件读取失败，我们会忽略它并继续处理下一个
            }

            // 5. 如果成功合并了内容，则更新编辑器的主文本
            if !combined_content.is_empty() {
                self.markdown_text = combined_content;
                // UI 将在下一帧自动刷新显示新内容
            }
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