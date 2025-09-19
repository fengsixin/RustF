use eframe::egui;
use regex::Regex;
use std::collections::HashSet;

use crate::state::MyApp;

impl MyApp {
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
        let re = Regex::new(r"\{\{([^}]+?)\}\}" ).unwrap();
        
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
    
    pub fn show_assignment_window(&mut self, ctx: &egui::Context) {
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

    pub fn apply_custom_style(&mut self, ctx: &egui::Context, style_name: &str, is_block: bool) {
        if is_block {
            let prefix = format!("::: {{custom-style=\" { } \"}}\n", style_name);
            let suffix = "\n:::";
            self.apply_formatting_to_selection(ctx, &prefix, suffix);

        } else {
            let prefix = "[";
            let suffix = format!("]{{custom-style=\"{}\"}}", style_name);
            self.apply_formatting_to_selection(ctx, prefix, &suffix);
        }
    }

    /// 根据搜索文本，更新过滤后的样式列表
    pub fn update_filtered_styles(&mut self) {
        self.palette_filtered_styles.clear();
        let search_text = self.palette_search_text.to_lowercase();

        // 过滤段落样式
        for style in &self.paragraph_styles {
            if style.to_lowercase().contains(&search_text) {
                // (样式名, is_block = true)
                self.palette_filtered_styles.push((style.clone(), true));
            }
        }
        // 过滤字符样式
        for style in &self.character_styles {
            if style.to_lowercase().contains(&search_text) {
                // (样式名, is_block = false)
                self.palette_filtered_styles.push((style.clone(), false));
            }
        }
        // 确保选中索引不会越界
        self.palette_selected_index = self.palette_selected_index.min(self.palette_filtered_styles.len().saturating_sub(1));
    }

    pub fn show_style_palette(&mut self, ctx: &egui::Context) {
        let mut style_to_apply_from_click = None;
        let mut apply_style_from_enter = false;

        // 重置滚动标志
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
                    ui.set_width(ui.available_width()); // 使用可用宽度
                    for (i, (style_name, is_block)) in self.palette_filtered_styles.iter().enumerate() {
                        // 对于数字样式ID，添加类型说明
                        let label = if style_name.chars().all(|c| c.is_ascii_digit()) {
                            format!("样式ID: {} ({})", style_name, if *is_block { "段落" } else { "字符" })
                        } else {
                            format!("{} ({})", style_name, if *is_block { "段落" } else { "字符" })
                        };
                        
                        let response = ui.selectable_label(self.palette_selected_index == i, &label);
                        
                        // 通过添加一个占据剩余空间的空元素来填充宽度
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                egui::vec2(ui.available_width(), response.rect.height()),
                                egui::Label::new("")
                            );
                        });

                        if response.clicked() {
                            style_to_apply_from_click = Some((style_name.clone(), *is_block));
                            self.palette_selected_index = i; // 更新选中索引
                        }

                        // 只有在需要时才滚动到选中的项目（例如，通过键盘导航）
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

        if apply_style_from_enter 
            && let Some((style_name, is_block)) = self.palette_filtered_styles.get(self.palette_selected_index).cloned() {
            self.apply_custom_style(ctx, &style_name, is_block);
            self.style_palette_open = false;
            self.palette_should_scroll_to_selected = false;
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
}
