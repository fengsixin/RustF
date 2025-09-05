use eframe::{egui, App, Frame};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::Builder;
use crossbeam_channel;

use crate::font_utils;

pub struct MyApp {
    markdown_text: String,
    cache: egui_commonmark::CommonMarkCache,
    scroll_linked: bool,
    scroll_proportion: f32,
    preview_max_scroll: f32,
    
    assignment_window_open: bool,
    template_markers: Vec<String>,
    marker_values: HashMap<String, String>,
    conversion_receiver: Option<crossbeam_channel::Receiver<Result<String, String>>>, 
    import_receiver: Option<crossbeam_channel::Receiver<Result<String, String>>>, 
    reference_doc_path: Option<std::path::PathBuf>,
    about_window_open: bool,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        font_utils::setup_chinese_fonts(&cc.egui_ctx);

        Self {
            markdown_text: include_str!("../user_guide.md").to_owned(),
            cache: egui_commonmark::CommonMarkCache::default(),
            scroll_linked: true,
            scroll_proportion: 0.0,
            preview_max_scroll: 0.0,
            
            assignment_window_open: false,
            template_markers: Vec::new(),
            marker_values: HashMap::new(),
            conversion_receiver: None,
            import_receiver: None,
            reference_doc_path: None,
            about_window_open: false,
        }
    }

    fn check_for_conversion_result(&mut self) {
        if let Some(receiver) = &self.conversion_receiver {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    Ok(success_message) => {
                        rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Info)
                            .set_title("成功")
                            .set_description(&success_message)
                            .show();
                    }
                    Err(error_message) => {
                        rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("导出失败")
                            .set_description(&error_message)
                            .show();
                    }
                }
                self.conversion_receiver = None;
            }
        }
    }

    fn check_for_import_result(&mut self) {
        if let Some(receiver) = &self.import_receiver {
            if let Ok(result) = receiver.try_recv() {
                match result {
                    Ok(markdown_content) => {
                        self.markdown_text = markdown_content;
                        rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Info)
                            .set_title("成功")
                            .set_description("DOCX 文件已成功导入。")
                            .show();
                    }
                    Err(error_message) => {
                        rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("导入失败")
                            .set_description(&error_message)
                            .show();
                    }
                }
                self.import_receiver = None;
            }
        }
    }

    fn set_reference_doc(&mut self) {
        let handle = rfd::FileDialog::new()
            .add_filter("Word 文档", &["docx"])
            .set_title("选择一个 DOCX 模板文件")
            .pick_file();
            
        if let Some(path) = handle {
            self.reference_doc_path = Some(path);
        }
    }

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

    fn import_from_docx(&mut self) {
        if self.import_receiver.is_some() || self.conversion_receiver.is_some() {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Warning)
                .set_title("请稍候")
                .set_description("当前有另一个文件操作任务正在进行中。")
                .show();
            return;
        }

        let input_path = match rfd::FileDialog::new()
            .add_filter("Word 文档", &["docx"])
            .pick_file() {
            Some(path) => path,
            None => return,
        };

        let (sender, receiver) = crossbeam_channel::unbounded();
        self.import_receiver = Some(receiver);

        std::thread::spawn(move || {
            let pandoc_executable_name = if cfg!(target_os = "windows") { "pandoc.exe" } else { "pandoc" };
            let pandoc_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join(pandoc_executable_name)))
                .filter(|p| p.exists())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "pandoc".to_string());

            let pandoc_output = Command::new(pandoc_path)
                .arg(&input_path)
                .arg("-f")
                .arg("docx")
                .arg("-t")
                .arg("markdown")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();

            let result = match pandoc_output {
                Ok(output) => {
                    if output.status.success() {
                        String::from_utf8(output.stdout)
                            .map_err(|e| format!("解析 Pandoc 输出失败: {}", e))
                    } else {
                        let error_message = String::from_utf8_lossy(&output.stderr);
                        Err(format!("Pandoc 转换失败:\n{}", error_message))
                    }
                }
                Err(e) => {
                    Err(format!("无法执行 Pandoc 命令。\n请确保 Pandoc 已正确安装。\n\n错误详情: {}", e))
                }
            };
            
            let _ = sender.send(result);
        });
    }

    fn export_as_docx(&mut self) {
        if self.conversion_receiver.is_some() {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Warning)
                .set_title("请稍候")
                .set_description("上一个转换任务仍在进行中。")
                .show();
            return;
        }

        let output_path = match rfd::FileDialog::new()
            .add_filter("Word 文档", &["docx"])
            .save_file() {
            Some(path) => path,
            None => return,
        };

        let (sender, receiver) = crossbeam_channel::unbounded();
        self.conversion_receiver = Some(receiver);
        let markdown_content = self.markdown_text.clone();
        let reference_doc = self.reference_doc_path.clone();

        std::thread::spawn(move || {
            let mut temp_file = match Builder::new().prefix("pandoc_input").suffix(".md").tempfile() {
                Ok(file) => file,
                Err(_) => {
                    let _ = sender.send(Err("无法创建临时文件。".to_string()));
                    return;
                }
            };

            if let Err(_) = temp_file.write_all(markdown_content.as_bytes()) {
                let _ = sender.send(Err("无法写入临时文件。".to_string()));
                return;
            }

            let pandoc_executable_name = if cfg!(target_os = "windows") { "pandoc.exe" } else { "pandoc" };
            let pandoc_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join(pandoc_executable_name)))
                .filter(|p| p.exists())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "pandoc".to_string());

            let mut command = Command::new(pandoc_path);
            
            command.arg(temp_file.path())
                   .arg("-o")
                   .arg(&output_path);

            if let Some(ref_path) = reference_doc {
                command.arg("--reference-doc").arg(ref_path);
            }

            let pandoc_output = command.stdout(Stdio::piped())
                                     .stderr(Stdio::piped())
                                     .output();

            let result = match pandoc_output {
                Ok(output) => {
                    if output.status.success() {
                        Ok("文件已成功导出为 DOCX。".to_string())
                    } else {
                        let error_message = String::from_utf8_lossy(&output.stderr);
                        Err(format!("Pandoc 转换失败:\n{}", error_message))
                    }
                }
                Err(e) => {
                    Err(format!("无法执行 Pandoc 命令。\n请确保 Pandoc 已正确安装并位于系统 PATH 中，或与本程序在同一目录下。\n\n错误详情: {}", e))
                }
            };
            
            let _ = sender.send(result);
        });
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

    fn show_about_window(&mut self, ctx: &egui::Context) {
        let mut close_button_clicked = false;
        egui::Window::new("关于 文档风格转换器")
            .collapsible(false)
            .resizable(false)
            .open(&mut self.about_window_open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("文档风格转换器");
                    
                    ui.label(format!("版本: {}", env!("CARGO_PKG_VERSION")));

                    ui.add_space(10.0);
                    ui.label("作者: 冯思昕");
                    ui.add_space(10.0);
                    
                    ui.separator();

                    ui.label("鸣谢以下优秀项目:");
                    ui.hyperlink_to("Rust 语言", "https://www.rust-lang.org/");
                    ui.hyperlink_to("egui 图形库", "https://github.com/emilk/egui");
                    ui.hyperlink_to("Pandoc 文档转换工具", "https://pandoc.org/");
                    
                    ui.add_space(20.0);

                    if ui.button("关闭").clicked() {
                        close_button_clicked = true;
                    }
                });
            });
        if close_button_clicked {
            self.about_window_open = false;
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.check_for_conversion_result();
        self.check_for_import_result();

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
            self.apply_formatting_to_selection(ctx, "[", "]{{.underline}}");
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
                    ui.separator();
                    if ui.button("导入 DOCX...").clicked() {
                        ui.close();
                        self.import_from_docx();
                    }
                    if ui.button("导出为 DOCX").clicked() {
                        ui.close();
                        self.export_as_docx();
                    }

                    ui.separator();

                    if ui.button("设置导出模板...").clicked() {
                        ui.close();
                        self.set_reference_doc();
                    }

                    let mut clear_template = false;
                    if let Some(path) = &self.reference_doc_path {
                        ui.horizontal(|ui| {
                            let filename = path.file_name()
                                .map(|s| s.to_string_lossy())
                                .unwrap_or_default();
                            
                            ui.label(format!("当前模板: {}", filename));

                            if ui.button("清除").clicked() {
                                clear_template = true;
                                ui.close();
                            }
                        });
                    }
                    if clear_template {
                        self.reference_doc_path = None;
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

                ui.menu_button("帮助", |ui| {
                    if ui.button("关于...").clicked() {
                        self.about_window_open = true;
                        ui.close();
                    }
                });
            });
        });
        
        if self.about_window_open {
            self.show_about_window(ctx);
        }

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
                                            let font_id_clone = font_id.clone();
                                            for (i, row) in galley.rows.iter().enumerate() {
                                                if i == 0 || galley.rows.get(i.saturating_sub(1)).map_or(false, |prev_row| prev_row.row.ends_with_newline) {
                                                    let line_y = rect.min.y + row.pos.y;
                                                    let row_height = row.row.size.y;
                                                
                                                    let line_rect = egui::Rect::from_min_size(
                                                        egui::pos2(rect.left(), line_y),
                                                        egui::vec2(rect.width(), row_height),
                                                    );
                                                
                                                    ui.painter().text(
                                                        line_rect.right_center(),
                                                        egui::Align2::RIGHT_CENTER,
                                                        logical_line.to_string(),
                                                        font_id_clone.clone(),
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

                            let max_offset_y = editor_scroll_response.content_size.y - editor_scroll_response.inner_rect.height();
                            if max_offset_y > 0.0 {
                                self.scroll_proportion = editor_scroll_response.state.offset.y / max_offset_y;
                            }
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
                                let target_offset_y = self.scroll_proportion * self.preview_max_scroll;
                                preview_scroll_area = preview_scroll_area.vertical_scroll_offset(target_offset_y);
                            }

                            let preview_scroll_response = preview_scroll_area.show(ui, |ui| {
                                egui::Frame::NONE
                                    .inner_margin(egui::Margin::same(10))
                                    .show(ui, |ui| {
                                        egui_commonmark::CommonMarkViewer::new().show(ui, &mut self.cache, &self.markdown_text);
                                    });
                            });

                            self.preview_max_scroll = preview_scroll_response.content_size.y - preview_scroll_response.inner_rect.height();
                        });
                    });
            });
        });
    }
}
