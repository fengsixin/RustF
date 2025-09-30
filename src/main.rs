#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod font_utils {
    use eframe::egui;

    pub fn setup_chinese_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        
        if let Some(chinese_font_data) = load_system_chinese_font() {
            fonts.font_data.insert("chinese".to_owned(), chinese_font_data.into());
            fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
            fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
        }
        
        ctx.set_fonts(fonts);
    }

    fn load_system_chinese_font() -> Option<egui::FontData> {
        #[cfg(target_os = "windows")]
        {
            let font_paths = [
                r"C:\\Windows\\Fonts\\msyh.ttc",
                r"C:\\Windows\\Fonts\\simsun.ttc",
                r"C:\\Windows\\Fonts\\msyhbd.ttc",
                r"C:\\Windows\\Fonts\\simhei.ttf",
                r"C:\\Windows\\Fonts\\simkai.ttf",
            ];
            for font_path in &font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    return Some(egui::FontData::from_owned(font_data));
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let font_paths = [
                "/System/Library/Fonts/PingFang.ttc",
                "/System/Library/Fonts/STHeiti Medium.ttc",
                "/Library/Fonts/Arial Unicode.ttf",
            ];
            for font_path in &font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    return Some(egui::FontData::from_owned(font_data));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let font_paths = [
                "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",
                "/usr/share/fonts/wenquan-micro-hei/wqy-microhei.ttc",
                "/usr/share/fonts/wqy-microhei/wqy-microhei.ttc",
                "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
            ];
            for font_path in &font_paths {
                if let Ok(font_data) = std::fs::read(font_path) {
                    return Some(egui::FontData::from_owned(font_data));
                }
            }
        }

        None
    }
}

mod state {
    use std::collections::{HashMap, HashSet};
    use regex::Regex;
    use eframe::{egui, App, Frame};
    use crate::font_utils;
    use std::io::Write;
    use std::process::{Command, Stdio};
    use tempfile::Builder;

    pub struct MyApp {
        pub markdown_text: String,
        pub cache: egui_commonmark::CommonMarkCache,
        pub scroll_linked: bool,
        pub scroll_proportion: f32,
        pub preview_max_scroll: f32,
        
        pub assignment_window_open: bool,
        pub template_markers: Vec<String>,
        pub marker_values: HashMap<String, String>,
        pub conversion_receiver: Option<crossbeam_channel::Receiver<Result<String, String>>>,
        pub import_receiver: Option<crossbeam_channel::Receiver<Result<String, String>>>,
        pub reference_doc_path: Option<std::path::PathBuf>,
        pub about_window_open: bool,
        pub paragraph_styles: Vec<String>,
        pub character_styles: Vec<String>,

        pub style_palette_open: bool,
        pub palette_search_text: String,
        pub palette_selected_index: usize,
        pub palette_filtered_styles: Vec<(String, bool)>,
        pub palette_should_scroll_to_selected: bool,
        pub underline_regex: Regex,

        pub info_dialog_open: bool,
        pub info_dialog_title: String,
        pub info_dialog_message: String,

        pub import_dialog_open: bool,
        pub import_text_area: String,
    }

    impl App for MyApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
            self.check_for_conversion_result();
            self.check_for_import_result();

            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                for file in &ctx.input(|i| i.raw.dropped_files.clone()) {
                    if let Some(path) = &file.path {
                        self.process_dropped_path(ctx, path);
                    }
                }
            }

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

            if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.ctrl) {
                ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::T));
                self.apply_image_width_control(ctx);
                request_repaint = true;
            }

            if request_repaint {
                ctx.request_repaint();
            }

            if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl && i.modifiers.shift) {
                ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::S));
                
                self.style_palette_open = !self.style_palette_open;

                if self.style_palette_open {
                    self.palette_search_text.clear();
                    self.palette_selected_index = 0;
                    self.update_filtered_styles(); 
                }
            }

            self.show_menu_bar(ctx);
            
            if self.about_window_open {
                self.show_about_window(ctx);
            }

            if self.assignment_window_open {
                self.show_assignment_window(ctx);
            }

            if self.style_palette_open {
                self.show_style_palette(ctx);
            }

            if self.info_dialog_open {
                self.show_info_dialog(ctx);
            }

            if self.import_dialog_open {
                self.show_import_dialog(ctx);
            }
            
            self.show_panels(ctx);
        }
    }

    impl MyApp {
        pub fn new(cc: &eframe::CreationContext) -> Self {
            font_utils::setup_chinese_fonts(&cc.egui_ctx);

            let underline_regex = Regex::new(r"\{\{.*?\}\}").unwrap();

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
                paragraph_styles: Vec::new(),
                character_styles: Vec::new(),
                style_palette_open: false,
                palette_search_text: String::new(),
                palette_selected_index: 0,
                palette_filtered_styles: Vec::new(),
                palette_should_scroll_to_selected: false,
                underline_regex,
                info_dialog_open: false,
                info_dialog_title: String::new(),
                info_dialog_message: String::new(),
                import_dialog_open: false,
                import_text_area: String::new(),
            }
        }

        pub fn process_dropped_path(&mut self, ctx: &egui::Context, path: &std::path::Path) {
            if path.is_file() {
                if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                    let is_image = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp"].contains(&extension.to_lowercase().as_str());
                    if is_image {
                        self.insert_image_markdown(ctx, path);
                    }
                }
            } else if path.is_dir() {
                self.process_dropped_directory(ctx, path);
            }
        }

        fn process_dropped_directory(&mut self, ctx: &egui::Context, dir_path: &std::path::Path) {
            if let Ok(entries) = std::fs::read_dir(dir_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                            let is_image = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp"].contains(&extension.to_lowercase().as_str());
                            if is_image {
                                self.insert_image_markdown(ctx, &path);
                            }
                        }
                    } else if path.is_dir() {
                        self.process_dropped_directory(ctx, &path);
                    }
                }
            }
        }

        pub fn insert_image_markdown(&mut self, ctx: &egui::Context, file_path: &std::path::Path) {
            let editor_id = egui::Id::new("main_editor_id");
            
            if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
                let filename = file_path.file_name()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_else(|| "image".into());

                let markdown_image = format!("![{}]({})
", filename, file_path.to_string_lossy());
                
                let current_pos = state.cursor.char_range().map(|r| r.primary.index).unwrap_or(self.markdown_text.chars().count());
                
                let text = self.markdown_text.clone();
                let chars = text.chars().collect::<Vec<_>>();
                let (prefix, suffix) = chars.split_at(current_pos);
                self.markdown_text = prefix.iter().collect::<String>() + &markdown_image + &suffix.iter().collect::<String>();
                
                let new_cursor_pos = current_pos + markdown_image.chars().count();
                let new_cursor = egui::text::CCursor::new(new_cursor_pos);
                let new_range = egui::text::CCursorRange::one(new_cursor);
                state.cursor.set_char_range(Some(new_range));
                egui::TextEdit::store_state(ctx, editor_id, state);
            } else {
                let filename = file_path.file_name()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_else(|| "image".into());
                self.markdown_text.push_str(&format!("

![{}]({})
", filename, file_path.to_string_lossy()));
            }
        }

        pub fn load_file(&mut self) {
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
        
        pub fn save_file(&self) {
            let handle = rfd::FileDialog::new()
                .add_filter("Markdown", &["md", "markdown"])
                .add_filter("Text", &["txt"])
                .save_file();
                
            if let Some(path) = handle {
                let _ = std::fs::write(path, &self.markdown_text);
            }
        }
        
        pub fn merge_files(&mut self) {
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
                            combined_content.push_str("\n\n");
                        }
                    }
                }

                if !combined_content.is_empty() {
                    self.markdown_text = combined_content;
                }
            }
        }

        pub fn export_template_variables(&mut self) {
            self.scan_and_update_markers();
            if self.marker_values.is_empty() {
                self.open_info_dialog("导出模板变量", "没有找到任何模板变量，无需导出。");
                return;
            }

            let mut content = String::new();
            for (key, value) in &self.marker_values {
                content.push_str(&format!("{}={}\n", key, value));
            }

            let handle = rfd::FileDialog::new()
                .add_filter("Text", &["txt"])
                .set_file_name("template_variables.txt")
                .save_file();

            if let Some(path) = handle {
                match std::fs::write(path, &content) {
                    Ok(_) => self.open_info_dialog("成功", "模板变量已成功导出。"),
                    Err(e) => self.open_info_dialog("错误", &format!("导出失败：{}", e)),
                }
            }
        }

        pub fn check_for_task_result<T, F>(
            receiver_option: &mut Option<crossbeam_channel::Receiver<Result<T, String>>>,
            success_handler: F,
        ) where
            T: 'static,
            F: FnOnce(T),
        {
            if let Some(receiver) = receiver_option {
                if let Ok(result) = receiver.try_recv() {
                    match result {
                        Ok(value) => success_handler(value),
                        Err(error_message) => {
                            rfd::MessageDialog::new()
                                .set_level(rfd::MessageLevel::Error)
                                .set_title("操作失败")
                                .set_description(&error_message)
                                .show();
                        }
                    }
                    *receiver_option = None;
                }
            }
        }

        pub fn check_for_conversion_result(&mut self) {
            Self::check_for_task_result(&mut self.conversion_receiver, |success_message| {
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Info)
                    .set_title("成功")
                    .set_description(&success_message)
                    .show();
            });
        }

        pub fn check_for_import_result(&mut self) {
            Self::check_for_task_result(&mut self.import_receiver, |markdown_content| {
                self.markdown_text = markdown_content;
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Info)
                    .set_title("成功")
                    .set_description("DOCX 文件已成功导入。")
                    .show();
            });
        }

        pub fn set_reference_doc(&mut self) {
            let handle = rfd::FileDialog::new()
                .add_filter("Word 文档", &["docx"])
                .set_title("选择一个 DOCX 模板文件")
                .pick_file();

            if let Some(path) = handle {
                match std::fs::read(&path) {
                    Ok(data) => {
                        match docx_rs::read_docx(&data) {
                            Ok(docx) => {
                                self.paragraph_styles.clear();
                                self.character_styles.clear();

                                let mut default_style_ids = std::collections::HashSet::new();
                                let common_paragraph_styles = [
                                    "Normal", "Heading1", "Heading2", "Heading3", "Heading4",
                                    "Heading5", "Heading6", "Heading7", "Heading8", "Heading9",
                                    "Title", "Subtitle", "ListParagraph", "Caption",
                                    "TOC1", "TOC2", "TOC3", "TableNormal"
                                ];
                                
                                let common_character_styles = [
                                    "DefaultParagraphFont", "Emphasis", "Strong"
                                ];

                                for style in &common_paragraph_styles {
                                    default_style_ids.insert(style.to_string());
                                }
                                
                                for style in &common_character_styles {
                                    default_style_ids.insert(style.to_string());
                                }


                                for s in docx.styles.styles {
                                    let name = &s.style_id;
                                    let display_name = name.clone();
                                    
                                    if !name.is_empty() && !default_style_ids.contains(name) {
                                        match s.style_type {
                                            docx_rs::StyleType::Paragraph => {
                                                self.paragraph_styles.push(display_name);
                                            }
                                            docx_rs::StyleType::Character => {
                                                self.character_styles.push(display_name);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                self.paragraph_styles.sort();
                                self.character_styles.sort();
                                self.reference_doc_path = Some(path);

                                rfd::MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Info)
                                    .set_title("模板加载成功")
                                    .set_description(format!(
                                        "成功加载模板，发现 {} 个段落样式和 {} 个字符样式。",
                                        self.paragraph_styles.len(),
                                        self.character_styles.len()
                                    ))
                                    .show();
                            }
                            Err(e) => {
                                self.reference_doc_path = None;
                                self.paragraph_styles.clear();
                                self.character_styles.clear();
                                rfd::MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Error)
                                    .set_title("模板加载失败")
                                    .set_description(format!("无法解析DOCX文件: {:?}", e))
                                    .show();
                            }
                        }
                    }
                    Err(e) => {
                        self.reference_doc_path = None;
                        self.paragraph_styles.clear();
                        self.character_styles.clear();
                        rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("模板加载失败")
                            .set_description(format!("无法读取文件: {}", e))
                            .show();
                    }
                }
            }
        }

        pub fn import_from_docx(&mut self) {
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

        pub fn export_as_docx(&mut self) {
            if self.conversion_receiver.is_some() {
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Warning)
                    .set_title("请稍候")
                    .set_description("上一个转换任务仍在进行中。")
                    .show();
                return;
            }

            let current_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));

            let default_file_name = "out01.docx";

            let output_path = match rfd::FileDialog::new()
                .add_filter("Word 文档", &["docx"])
                .set_directory(&current_dir)
                .set_file_name(default_file_name)
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
                    Err(e) => {
                        let _ = sender.send(Err(format!("无法创建临时文件: {}", e)));
                        return;
                    }
                };

                if temp_file.write_all(markdown_content.as_bytes()).is_err() {
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

        pub fn scan_and_update_markers(&mut self) {
            let re = Regex::new(r"\{\{([^}]+?)\}\}").unwrap();
            let mut current_markers = HashSet::new();
            for mat in re.find_iter(&self.markdown_text) {
                current_markers.insert(mat.as_str().to_string());
            }

            let mut new_marker_values = std::collections::HashMap::new();
            for marker in &current_markers {
                if let Some(old_value) = self.marker_values.get(marker) {
                    new_marker_values.insert(marker.clone(), old_value.clone());
                } else {
                    new_marker_values.insert(marker.clone(), String::new());
                }
            }
            
            self.marker_values = new_marker_values;

            self.template_markers = current_markers.into_iter().collect();
            self.template_markers.sort();
        }

        pub fn open_info_dialog(&mut self, title: &str, message: &str) {
            self.info_dialog_title = title.to_owned();
            self.info_dialog_message = message.to_owned();
            self.info_dialog_open = true;
        }

        pub fn import_and_apply_variables(&mut self, text: &str) {
            self.scan_and_update_markers();

            let mut updated_count = 0;

            for line in text.lines() {
                if line.trim().is_empty() {
                    continue;
                }

                if let Some((key, value)) = line.split_once("=") {
                    let key = key.trim();
                    let value = value.trim();

                    let full_marker = if key.starts_with("{{") && key.ends_with("}}") {
                        key.to_string()
                    } else {
                        format!("{{{{{}}}}}", key)
                    };

                    if self.marker_values.contains_key(&full_marker) {
                        self.marker_values.insert(full_marker, value.to_string());
                        updated_count += 1;
                    }
                }
            }

            if updated_count > 0 {
                self.apply_template_variables_to_markdown();
                self.open_info_dialog(
                    "导入完成",
                    &format!("成功更新并应用了 {} 个变量。", updated_count)
                );
            } else {
                self.open_info_dialog(
                    "导入提醒",
                    "没有发现可匹配的变量进行更新。"
                );
            }
        }

        pub fn apply_template_variables_to_markdown(&mut self) {
            for (marker, value) in self.marker_values.clone() {
                if !value.is_empty() {
                    self.markdown_text = self.markdown_text.replace(&marker, &value);
                }
            }
        }

        pub fn apply_underline_to_variables(&mut self, ctx: &egui::Context) {
            let mut replacements = Vec::new();
            let markdown_clone = self.markdown_text.clone();

            for mat in self.underline_regex.find_iter(&markdown_clone) {
                let start = mat.start();
                let end = mat.end();

                let is_preceded = markdown_clone.get(..start)
                    .and_then(|s| s.chars().last()) == Some('[');
                
                let is_followed = markdown_clone.get(end..)
                    .map_or(false, |s| s.starts_with("]{.underline}"));

                if !is_preceded || !is_followed {
                    replacements.push((mat.range(), format!("[{}]{{.underline}}", mat.as_str())));
                }
            }

            let count = replacements.len();

            if count > 0 {
                for (range, replacement) in replacements.iter().rev() {
                    self.markdown_text.replace_range(range.clone(), replacement);
                }
                self.info_dialog_message = format!("成功为 {} 个占位符添加了下划线。", count);
            } else {
                self.info_dialog_message = "未找到需要添加下划线的 {{...}} 标记。".to_string();
            }
            
            self.info_dialog_title = "操作完成".to_string();
            self.info_dialog_open = true;
            ctx.request_repaint();
        }

        pub fn show_about_window(&mut self, ctx: &egui::Context) {
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

        pub fn open_assignment_window(&mut self) {
            self.scan_and_update_markers();
            self.assignment_window_open = true;
        }
        
        pub fn show_assignment_window(&mut self, ctx: &egui::Context) {
            let mut apply_and_close = false;
            let mut cancel_and_close = false;

            egui::Window::new("模板变量赋值")
                .open(&mut self.assignment_window_open)
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
                            apply_and_close = true;
                        }

                        if ui.button("取消").clicked() {
                            cancel_and_close = true;
                        }
                    });
                });

            if apply_and_close {
                self.apply_template_variables_to_markdown();
                self.template_markers.clear();
                self.assignment_window_open = false;
            }
            if cancel_and_close {
                self.assignment_window_open = false;
            }
        }

        pub fn apply_custom_style(&mut self, ctx: &egui::Context, style_name: &str, is_block: bool) {
            if is_block {
                let prefix = format!("::: {{custom-style=\"{}\"}}\n", style_name);
                let suffix = "\n:::";
                self.apply_formatting_to_selection(ctx, &prefix, suffix);

            } else {
                let prefix = "[";
                let suffix = format!("]{{custom-style=\"{}\"}}", style_name);
                self.apply_formatting_to_selection(ctx, prefix, &suffix);
            }
        }

        pub fn update_filtered_styles(&mut self) {
            self.palette_filtered_styles.clear();
            let search_text = self.palette_search_text.to_lowercase();

            for style in &self.paragraph_styles {
                if style.to_lowercase().contains(&search_text) {
                    self.palette_filtered_styles.push((style.clone(), true));
                }
            }
            for style in &self.character_styles {
                if style.to_lowercase().contains(&search_text) {
                    self.palette_filtered_styles.push((style.clone(), false));
                }
            }
            self.palette_selected_index = self.palette_selected_index.min(self.palette_filtered_styles.len().saturating_sub(1));
        }

        pub fn show_style_palette(&mut self, ctx: &egui::Context) {
            let mut style_to_apply_from_click = None;
            let mut apply_style_from_enter = false;

            self.palette_should_scroll_to_selected = false;

            let area = egui::Area::new("style_palette_area".into())
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO);

            let response = area.show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_max_width(300.0);

                    if self.paragraph_styles.is_empty() && self.character_styles.is_empty() {
                        ui.label("当前未加载任何自定义样式。");
                        ui.label("请先通过“文件 -> 设置导出模板...”");
                        ui.label("加载一个包含自定义样式的 DOCX 文件。");
                        return;
                    }

                    let search_box_id = ui.id().with("palette_search");
                    let search_box = ui.add(
                        egui::TextEdit::singleline(&mut self.palette_search_text)
                            .hint_text("搜索样式...")
                            .id(search_box_id),
                    );
                    ctx.memory_mut(|m| m.request_focus(search_box_id));

                    if search_box.changed() {
                        self.update_filtered_styles();
                    }

                    ui.separator();

                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for (i, (style_name, is_block)) in self.palette_filtered_styles.iter().enumerate() {
                            let label = if style_name.chars().all(|c| c.is_ascii_digit()) {
                                format!("样式ID: {} ({})", style_name, if *is_block { "段落" } else { "字符" })
                            } else {
                                format!("{} ({})", style_name, if *is_block { "段落" } else { "字符" })
                            };
                            
                            let response = ui.selectable_label(self.palette_selected_index == i, &label);
                            
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    egui::vec2(ui.available_width(), response.rect.height()),
                                    egui::Label::new("")
                                );
                            });

                            if response.clicked() {
                                style_to_apply_from_click = Some((style_name.clone(), *is_block));
                                self.palette_selected_index = i;
                            }

                            if self.palette_should_scroll_to_selected && self.palette_selected_index == i {
                                response.scroll_to_me(Some(egui::Align::Center));
                            }
                        }
                    });

                    if !self.palette_filtered_styles.is_empty() {
                        let num_styles = self.palette_filtered_styles.len();
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            self.palette_selected_index = (self.palette_selected_index + 1) % num_styles;
                            self.palette_should_scroll_to_selected = true;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            self.palette_selected_index = (self.palette_selected_index + num_styles - 1) % num_styles;
                            self.palette_should_scroll_to_selected = true;
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            apply_style_from_enter = true;
                        }
                    }
                });
            });

            if let Some((style_name, is_block)) = style_to_apply_from_click {
                self.apply_custom_style(ctx, &style_name, is_block);
                self.style_palette_open = false;
                self.palette_should_scroll_to_selected = false;
            }

            if apply_style_from_enter {
                if let Some((style_name, is_block)) = self.palette_filtered_styles.get(self.palette_selected_index).cloned() {
                    self.apply_custom_style(ctx, &style_name, is_block);
                    self.style_palette_open = false;
                    self.palette_should_scroll_to_selected = false;
                }
            }

            if response.response.clicked_elsewhere() || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.style_palette_open = false;
            }
        }

        pub fn show_info_dialog(&mut self, ctx: &egui::Context) {
            let mut close_button_clicked = false;
            egui::Window::new(self.info_dialog_title.as_str())
                .collapsible(false)
                .resizable(true)
                .open(&mut self.info_dialog_open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(self.info_dialog_message.as_str());
                        ui.add_space(20.0);
                        if ui.button("关闭").clicked() {
                            close_button_clicked = true;
                        }
                    });
                });
            if close_button_clicked {
                self.info_dialog_open = false;
            }
        }

        pub fn show_import_dialog(&mut self, ctx: &egui::Context) {
            let mut import_and_close = false;
            let mut cancel_and_close = false;

            egui::Window::new("导入模板变量")
                .open(&mut self.import_dialog_open)
                .resizable(true)
                .default_width(400.0)
                .default_height(300.0)
                .show(ctx, |ui| {
                    ui.label("请将导出的变量内容粘贴到下方文本框中：");
                    ui.add_space(10.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_sized(
                            [ui.available_width(), 200.0],
                            egui::TextEdit::multiline(&mut self.import_text_area)
                                .desired_width(f32::INFINITY)
                        );
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("导入并替换").clicked() {
                            import_and_close = true;
                        }

                        if ui.button("取消").clicked() {
                            cancel_and_close = true;
                        }
                    });
                });

            if import_and_close {
                self.import_and_apply_variables(&self.import_text_area.clone());
                self.import_text_area.clear();
                self.import_dialog_open = false;
            }
            if cancel_and_close {
                self.import_dialog_open = false;
            }
        }

        pub fn show_menu_bar(&mut self, ctx: &egui::Context) {
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
                            self.paragraph_styles.clear();
                            self.character_styles.clear();
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
                        if ui.button("一键下划线").clicked() {
                            self.apply_underline_to_variables(ctx);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("导入模板变量").clicked() {
                            self.import_dialog_open = true;
                            ui.close();
                        }
                        if ui.button("导出模板变量").clicked() {
                            self.export_template_variables();
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
        }

        pub fn apply_formatting_to_selection(&mut self, ctx: &egui::Context, prefix: &str, suffix: &str) {
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

        pub fn show_panels(&mut self, ctx: &egui::Context) {
            egui::CentralPanel::default().show(ctx, |ui| {
                let stroke_color = ui.style().visuals.widgets.noninteractive.bg_stroke.color;
                
                let (editor_content_height, editor_visible_height) = {
                    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                    let char_width = ui.fonts(|f| f.glyph_width(&font_id, '0'));
                    let line_count = self.markdown_text.lines().count().max(1);
                    let num_digits = line_count.to_string().len();
                    let line_number_width = (num_digits as f32 * char_width) + 15.0;
                    
                    let galley = {
                        let mut job = egui::text::LayoutJob::default();
                        job.append(&self.markdown_text, 0.0, egui::TextFormat::simple(font_id.clone(), ui.style().visuals.text_color()));
                        job.wrap.max_width = ui.available_width() / 2.0 - line_number_width;
                        ui.fonts(|f| f.layout_job(job))
                    };
                    
                    let content_height = galley.size().y;
                    let visible_height = ui.available_height() - 30.0;
                    (content_height, visible_height)
                };
                
                let max_offset_y = (editor_content_height - editor_visible_height).max(0.0);
                let editor_scrollable = max_offset_y > 0.0;

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
                                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
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
                                                        if i == 0 || galley.rows.get(i.saturating_sub(1)).is_some_and(|prev_row| prev_row.row.ends_with_newline) {
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

                                                egui::TextEdit::multiline(&mut self.markdown_text)
                                                    .id(egui::Id::new("main_editor_id"))
                                                    .code_editor()
                                                    .desired_width(ui.available_width() - line_number_width)
                                                    .desired_rows(1)
                                                    .show(ui)
                                                    .response
                                            });
                                        });
                                    });

                                if editor_scrollable {
                                    self.scroll_proportion = editor_scroll_response.state.offset.y / max_offset_y;
                                }

                            });
                        });
                    
                    egui::Frame::new()
                        .inner_margin(egui::Margin { left: 10, right: 10, top: 10, bottom: 10 })
                        .stroke(egui::Stroke::new(1.0, stroke_color))
                        .show(&mut columns[1], |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                ui.label("预览区:");
                                ui.add_space(5.0);

                                let mut preview_scroll_area = egui::ScrollArea::vertical()
                                    .id_salt("preview_scroll_area")
                                    .auto_shrink([false; 2]);

                                if self.scroll_linked && editor_scrollable {
                                    let target_offset_y = self.scroll_proportion * self.preview_max_scroll;
                                    preview_scroll_area = preview_scroll_area.vertical_scroll_offset(target_offset_y);
                                } else if !editor_scrollable {
                                    preview_scroll_area = preview_scroll_area.vertical_scroll_offset(0.0);
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

        pub fn apply_image_width_control(&mut self, ctx: &egui::Context) {
            let editor_id = egui::Id::new("main_editor_id");
            
            if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
                if let Some(char_range) = state.cursor.char_range() {
                    let (primary_idx, secondary_idx) = (char_range.primary.index, char_range.secondary.index);

                    if primary_idx != secondary_idx {
                        let (start_char, end_char) = (primary_idx.min(secondary_idx), primary_idx.max(secondary_idx));

                        let char_to_byte: Vec<usize> = self.markdown_text.char_indices().map(|(i, _)| i).collect();
                        
                        if let Some(&start_byte) = char_to_byte.get(start_char) {
                            let end_byte = char_to_byte.get(end_char).copied().unwrap_or(self.markdown_text.len());
                            let selected_text = &self.markdown_text[start_byte..end_byte];
                            
                            let image_regex = regex::Regex::new(r"!\{\(.*?\)\}\[(.*?)\]\((.*?)\)").unwrap();
                            let mut modified_text = selected_text.to_string();
                            let mut found_images = false;
                            
                            for cap in image_regex.captures_iter(selected_text).collect::<Vec<_>>().into_iter().rev() {
                                if let Some(mat) = cap.get(0) {
                                    let full_match = mat.as_str();
                                    if !full_match.ends_with("{width=6in}") {
                                        let replacement = format!("{}{{width=6in}}", full_match);
                                        modified_text = modified_text.replacen(full_match, &replacement, 1);
                                        found_images = true;
                                    }
                                }
                            }
                            
                            if found_images {
                                self.markdown_text.replace_range(start_byte..end_byte, &modified_text);
                                
                                let new_text_char_len = modified_text.chars().count();
                                let new_cursor_pos_char = start_char + new_text_char_len;
                                
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(new_cursor_pos_char),
                                )));
                                state.store(ctx, editor_id);
                            } else {
                                self.show_no_images_alert(ctx);
                            }
                        }
                    } else {
                        self.show_no_images_alert(ctx);
                    }
                }
            } else {
                self.show_no_images_alert(ctx);
            }
        }
        
        fn show_no_images_alert(&mut self, ctx: &egui::Context) {
            self.show_alert(ctx, "无图片", "选中的内容中没有找到图片。");
        }
        
        fn show_alert(&mut self, ctx: &egui::Context, title: &str, message: &str) {
            egui::Window::new(title)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(message);
                    ui.add_space(10.0);
                    if ui.button("确定").clicked() {
                    }
                });
        }
    }
}



use state::MyApp;
use eframe::{NativeOptions, App};

fn main() {
    let native_options = NativeOptions::default();
    eframe::run_native(
        "文档风格转换器",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)) as Box<dyn App>)),
    ).unwrap();
}
