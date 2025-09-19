use eframe::{egui, App, Frame};
use crate::state::MyApp;

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.check_for_conversion_result();
        self.check_for_import_result();

        // 检查是否有文件拖入
        if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
            for file in &ctx.input(|i| i.raw.dropped_files.clone()) {
                if let Some(path) = &file.path {
                    // 处理文件或文件夹
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
        
        self.show_panels(ctx);
    }
}

impl MyApp {
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
}
