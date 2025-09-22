use crate::state::MyApp;

impl MyApp {
    pub fn load_file(&mut self) {
        let handle = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown"])
            .add_filter("Text", &["txt"])
            .pick_file();
            
        if let Some(path) = handle 
            && let Ok(content) = std::fs::read_to_string(path) {
            self.markdown_text = content;
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
}
