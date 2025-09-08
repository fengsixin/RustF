use eframe::egui;
use crate::state::MyApp;

impl MyApp {
    pub fn apply_formatting_to_selection(&mut self, ctx: &egui::Context, prefix: &str, suffix: &str) {
        let editor_id = egui::Id::new("main_editor_id");
        if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id)
            && let Some(char_range) = state.cursor.char_range() {
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

    pub fn show_panels(&mut self, ctx: &egui::Context) {
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
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
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
