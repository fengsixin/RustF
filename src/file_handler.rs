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
}
