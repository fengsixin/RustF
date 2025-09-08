use crate::state::MyApp;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::Builder;

impl MyApp {
    pub fn check_for_task_result<T, F>(
        receiver_option: &mut Option<crossbeam_channel::Receiver<Result<T, String>>>,
        success_handler: F,
    ) where
        T: 'static,
        F: FnOnce(T),
    {
        if let Some(receiver) = receiver_option 
            && let Ok(result) = receiver.try_recv() {
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
                            // Common default paragraph styles (style IDs can vary)
                            let common_paragraph_styles = [
                                "Normal", "Heading1", "Heading2", "Heading3", "Heading4",
                                "Heading5", "Heading6", "Heading7", "Heading8", "Heading9",
                                "Title", "Subtitle", "ListParagraph", "Caption",
                                "TOC1", "TOC2", "TOC3", "TableNormal"
                            ];
                            
                            // Common default character styles
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
                                // 对于数字ID，我们保留原始ID作为标识符
                                // 但在UI中显示时，我们可以添加样式类型的提示
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
}
