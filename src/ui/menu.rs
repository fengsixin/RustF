use eframe::egui;
use crate::state::MyApp;

impl MyApp {
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
}
