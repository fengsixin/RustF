#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{egui, App, Frame, NativeOptions};
use std::sync::Arc;
use regex::Regex;
use std::collections::{HashMap, HashSet};

struct MyApp {
    markdown_text: String,
    cache: egui_commonmark::CommonMarkCache,
    scroll_linked: bool,
    editor_scroll_offset: egui::Vec2,
    
    assignment_window_open: bool,
    template_markers: Vec<String>,
    marker_values: HashMap<String, String>,
}

impl App for MyApp {
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

        if request_repaint {
            ctx.request_repaint();
        }

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
                    if ui.button("合并文件").clicked() {
                        ui.close();
                        self.merge_files();
                    }
                });
                
                ui.menu_button("视图", |ui| {
                    ui.checkbox(&mut self.scroll_linked, "同步滚动");
                });
                
                ui.menu_button("工具", |ui| {
                    if ui.button("模板赋值").clicked() {
                        self.open_assignment_window();
                        ui.close();
                    }
                });
            });
        });
        
        if self.assignment_window_open {
            self.show_assignment_window(ctx);
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            let stroke_color = ui.style().visuals.widgets.noninteractive.bg_stroke.color;
            
            ui.columns(2, |columns| {
                egui::Frame::new()
                    .inner_margin(egui::Margin { left: 10, right: 10, top: 10, bottom: 10 })
                    .stroke(egui::Stroke::new(1.0, stroke_color))
                    .show(&mut columns[0], |ui| {
                        ui.vertical(|ui| {
                            ui.label("编辑区:");
                            ui.add_space(5.0);

                            let editor_scroll_response = egui::ScrollArea::vertical()
                                .id_salt("editor_scroll_area")
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                                    let char_width = ui.fonts(|f| f.glyph_width(&font_id, '0'));
                                    let line_count = self.markdown_text.lines().count().max(1);
                                    let num_digits = line_count.to_string().len();
                                    let line_number_width = (num_digits as f32 * char_width) + 15.0;

                                    let galley = {
                                        let mut job = egui::text::LayoutJob::default();
                                        job.append(&self.markdown_text, 0.0, egui::TextFormat::simple(font_id.clone(), ui.style().visuals.text_color()));
                                        job.wrap.max_width = ui.available_width() - line_number_width;
                                        ui.fonts(|f| f.layout_job(job))
                                    };

                                    ui.horizontal(|ui| {
                                        let line_number_painter = |ui: &mut egui::Ui| {
                                            let (rect, _) = ui.allocate_exact_size(
                                                egui::vec2(line_number_width, galley.size().y),
                                                egui::Sense::hover(),
                                            );

                                            let mut logical_line = 1;
                                            for row in galley.rows.iter() {
                                                // 检查是否是新段落的开始（通过检查第一个字符是否是行首）
                                                if row.row.glyphs.len() > 0 && row.row.glyphs[0].pos.x == 0.0 {
                                                    let line_y = rect.min.y + row.rect().min.y;
                                                    let line_rect = egui::Rect::from_min_size(
                                                        egui::pos2(rect.left(), line_y),
                                                        egui::vec2(rect.width(), row.rect().height()),
                                                    );

                                                    ui.painter().text(
                                                        line_rect.right_center(),
                                                        egui::Align2::RIGHT_CENTER,
                                                        logical_line.to_string(),
                                                        font_id.clone(),
                                                        egui::Color32::GRAY,
                                                    );
                                                    
                                                    logical_line += 1;
                                                }
                                            }
                                        };
                                        ui.scope(line_number_painter);

                                        let editor_response = egui::TextEdit::multiline(&mut self.markdown_text)
                                            .id(egui::Id::new("main_editor_id"))
                                            .code_editor()
                                            .desired_width(ui.available_width() - line_number_width)
                                            .desired_rows(1)
                                            .show(ui)
                                            .response;
                                        
                                        editor_response
                                    });
                                });

                            self.editor_scroll_offset = editor_scroll_response.state.offset;
                        });
                    });
                
                egui::Frame::new()
                    .inner_margin(egui::Margin { left: 10, right: 10, top: 10, bottom: 10 })
                    .stroke(egui::Stroke::new(1.0, stroke_color))
                    .show(&mut columns[1], |ui| {
                        ui.vertical(|ui| {
                            ui.label("预览区:");
                            ui.add_space(5.0);
                            
                            let mut preview_scroll_area = egui::ScrollArea::vertical()
                                .id_salt("preview_scroll_area")
                                .auto_shrink([false; 2]);
                            
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
    
    fn apply_formatting_to_selection(&mut self, ctx: &egui::Context, prefix: &str, suffix: &str) {
        let editor_id = egui::Id::new("main_editor_id");
        if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
            if let Some(char_range) = state.cursor.char_range() {
                let (primary_idx, secondary_idx) = (char_range.primary.index, char_range.secondary.index);

                if primary_idx != secondary_idx {
                    let (start_char, end_char) = (primary_idx.min(secondary_idx), primary_idx.max(secondary_idx));

                    let char_to_byte: Vec<usize> = self.markdown_text.char_indices().map(|(i, _)| i).collect();
                    
                    if let Some(&start_byte) = char_to_byte.get(start_char) {
                        let end_byte = char_to_byte.get(end_char).copied().unwrap_or(self.markdown_text.len());

                        let new_text = format!("{}{}{}", prefix, &self.markdown_text[start_byte..end_byte], suffix);
                        self.markdown_text.replace_range(start_byte..end_byte, &new_text);

                        let new_text_char_len = new_text.chars().count();
                        let new_cursor_pos_char = start_char + new_text_char_len;
                        
                        state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(new_cursor_pos_char),
                        )));
                        state.store(ctx, editor_id);
                    }
                }
            }
        }
    }
    
    fn merge_files(&mut self) {
        let files = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .add_filter("Text", &["txt"])
            .pick_files();

        if let Some(paths) = files {
            if paths.len() <= 1 {
                return;
            }

            let mut combined_content = String::new();

            for (index, path) in paths.iter().enumerate() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    combined_content.push_str(&content);

                    if index < paths.len() - 1 {
                        combined_content.push_str("\n\n---\n\n");
                    }
                }
            }

            if !combined_content.is_empty() {
                self.markdown_text = combined_content;
            }
        }
    }
    
    fn open_assignment_window(&mut self) {
        let re = Regex::new(r"\{\{([^}]+?)\}\}").unwrap();
        
        let mut unique_markers = HashSet::new();
        for mat in re.find_iter(&self.markdown_text) {
            unique_markers.insert(mat.as_str().to_string());
        }
        
        self.template_markers = unique_markers.into_iter().collect();
        self.template_markers.sort();

        self.marker_values.clear();
        for marker in &self.template_markers {
            self.marker_values.insert(marker.clone(), String::new());
        }

        self.assignment_window_open = true;
    }
    
    fn show_assignment_window(&mut self, ctx: &egui::Context) {
        let mut open = self.assignment_window_open;
        egui::Window::new("模板变量赋值")
            .open(&mut open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                if self.template_markers.is_empty() {
                    ui.label("在文档中没有找到 {{...}} 格式的标记。");
                    return;
                }

                ui.label("请为以下标记赋新值：");
                ui.add_space(10.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("assignment_grid").num_columns(2).show(ui, |ui| {
                        for marker in &self.template_markers {
                            ui.label(marker);
                            if let Some(value) = self.marker_values.get_mut(marker) {
                                ui.text_edit_singleline(value);
                            }
                            ui.end_row();
                        }
                    });
                });

                ui.add_space(10.0);
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("全部替换").clicked() {
                        for (marker, value) in &self.marker_values {
                            if !value.is_empty() {
                                self.markdown_text = self.markdown_text.replace(marker, value);
                            }
                        }
                        self.assignment_window_open = false;
                        self.template_markers.clear();
                        self.marker_values.clear();
                    }

                    if ui.button("取消").clicked() {
                        self.assignment_window_open = false;
                    }
                });
            });
        self.assignment_window_open = open;
    }
}

fn main() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "Markdown 编辑器",
        native_options,
        Box::new(|cc| {
            setup_chinese_fonts(&cc.egui_ctx);
            
            let app = MyApp {
                markdown_text: "# 欢迎使用 Markdown 编辑器\n\n这是一个简单的 Markdown 编辑器示例。\n\n## 功能\n\n- 实时预览\n- 支持标题、列表、代码块等\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```\n\n*斜体* 和 **粗体** 也支持。\n\n1. 第一项\n2. 第二项\n3. 第三项\n\n".to_owned() + 
                &"# 更多内容\n\n这是更多的内容，用于测试滚动同步功能。\n\n".repeat(50),
                cache: egui_commonmark::CommonMarkCache::default(),
                scroll_linked: true,
                editor_scroll_offset: egui::Vec2::ZERO,
                assignment_window_open: false,
                template_markers: Vec::new(),
                marker_values: HashMap::new(),
            };
            Ok(Box::new(app) as Box<dyn App>)
        }),
    ).unwrap();
}

fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    if let Some(chinese_font_data) = load_system_chinese_font() {
        fonts.font_data.insert("chinese".to_owned(), Arc::new(chinese_font_data));
        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
        fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
    }
    
    ctx.set_fonts(fonts);
}

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