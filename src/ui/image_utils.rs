use eframe::egui;
use crate::state::MyApp;

impl MyApp {
    /// 处理图片宽度控制功能
    /// 在选中的文本中为所有图片添加宽度控制代码
    pub fn apply_image_width_control(&mut self, ctx: &egui::Context) {
        let editor_id = egui::Id::new("main_editor_id");
        
        // 检查是否有选中的文本
        if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id)
            && let Some(char_range) = state.cursor.char_range() {
            let (primary_idx, secondary_idx) = (char_range.primary.index, char_range.secondary.index);

            if primary_idx != secondary_idx {
                let (start_char, end_char) = (primary_idx.min(secondary_idx), primary_idx.max(secondary_idx));

                // 获取选中的文本
                let char_to_byte: Vec<usize> = self.markdown_text.char_indices().map(|(i, _)| i).collect();
                
                if let Some(&start_byte) = char_to_byte.get(start_char) {
                    let end_byte = char_to_byte.get(end_char).copied().unwrap_or(self.markdown_text.len());
                    let selected_text = &self.markdown_text[start_byte..end_byte];
                    
                    // 查找选中文本中的所有图片标记
                    let image_regex = regex::Regex::new(r"\!\[(.*?)\]\((.*?)\)").unwrap();
                    let mut modified_text = selected_text.to_string();
                    let mut found_images = false;
                    
                    // 反向遍历匹配项以避免索引偏移问题
                    for cap in image_regex.captures_iter(selected_text).collect::<Vec<_>>().into_iter().rev() {
                        if let Some(mat) = cap.get(0) {
                            let full_match = mat.as_str();
                            // 检查是否已经有宽度控制
                            if !full_match.ends_with("{width=6in}") {
                                // 添加宽度控制代码
                                let replacement = format!("{}{{width=6in}}", full_match);
                                modified_text = modified_text.replacen(full_match, &replacement, 1);
                                found_images = true;
                            }
                        }
                    }
                    
                    // 如果找到了图片，则更新选中的文本
                    if found_images {
                        self.markdown_text.replace_range(start_byte..end_byte, &modified_text);
                        
                        // 更新光标位置到修改后文本的末尾
                        let new_text_char_len = modified_text.chars().count();
                        let new_cursor_pos_char = start_char + new_text_char_len;
                        
                        state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(new_cursor_pos_char),
                        )));
                        state.store(ctx, editor_id);
                    } else {
                        // 如果没有找到图片，显示提示信息
                        self.show_no_images_alert(ctx);
                    }
                }
            } else {
                // 没有选中文本，显示提示信息
                self.show_no_images_alert(ctx);
            }
        } else {
            // 无法获取编辑器状态，显示提示信息
            self.show_no_images_alert(ctx);
        }
    }
    
    /// 显示无图片提示信息
    fn show_no_images_alert(&mut self, ctx: &egui::Context) {
        // 创建一个临时的弹窗来显示提示信息
        // 这里我们使用一个简单的标志来控制弹窗显示
        // 在实际应用中，你可能需要使用更复杂的弹窗管理机制
        self.show_alert(ctx, "无图片", "选中的内容中没有找到图片。");
    }
    
    /// 显示提示信息弹窗
    fn show_alert(&mut self, ctx: &egui::Context, title: &str, message: &str) {
        // 创建一个简单的提示弹窗
        egui::Window::new(title)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(message);
                ui.add_space(10.0);
                if ui.button("确定").clicked() {
                    // 关闭弹窗的逻辑可以在这里添加
                    // 由于这是一个简单的实现，我们暂时不处理弹窗关闭
                }
            });
    }
}