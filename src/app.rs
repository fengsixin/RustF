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
                    // 确保是文件而不是文件夹
                    if path.is_file() {
                        // 通过文件扩展名判断是否为图片
                        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                            let is_image = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp"].contains(&extension.to_lowercase().as_str());
                            if is_image {
                                // 拖入的是图片，调用专门的方法来处理
                                self.insert_image_markdown(ctx, path);
                            }
                        }
                    }
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
        
        self.show_panels(ctx);
    }
}
