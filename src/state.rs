use std::collections::HashMap;
use crate::font_utils;
use regex::Regex;

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

    // --- 新增字段 ---
    /// 控制命令面板是否显示
    pub style_palette_open: bool,
    /// 存储命令面板中的搜索文本
    pub palette_search_text: String,
    /// 存储当前键盘选中的样式在过滤后列表中的索引
    pub palette_selected_index: usize,
    /// 存储过滤后的样式列表，元组包含 (样式名, 是否为段落样式)
    pub palette_filtered_styles: Vec<(String, bool)>,
    /// 标志，指示是否需要滚动到选中的项目
    pub palette_should_scroll_to_selected: bool,
    // 正则表达式
    pub underline_regex: Regex,

    // 通用信息提示框
    pub info_dialog_open: bool,
    pub info_dialog_title: String,
    pub info_dialog_message: String,
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
        }
    }

    /// 处理拖入的文件或文件夹
    pub fn process_dropped_path(&mut self, ctx: &egui::Context, path: &std::path::Path) {
        if path.is_file() {
            // 处理单个文件
            if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                let is_image = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp"].contains(&extension.to_lowercase().as_str());
                if is_image {
                    // 拖入的是图片，调用专门的方法来处理
                    self.insert_image_markdown(ctx, path);
                }
            }
        } else if path.is_dir() {
            // 处理文件夹
            self.process_dropped_directory(ctx, path);
        }
    }

    /// 递归处理拖入的文件夹
    fn process_dropped_directory(&mut self, ctx: &egui::Context, dir_path: &std::path::Path) {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    // 处理文件
                    if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                        let is_image = ["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp"].contains(&extension.to_lowercase().as_str());
                        if is_image {
                            // 拖入的是图片，调用专门的方法来处理
                            self.insert_image_markdown(ctx, &path);
                        }
                    }
                } else if path.is_dir() {
                    // 递归处理子文件夹
                    self.process_dropped_directory(ctx, &path);
                }
            }
        }
    }

    /// 将 Markdown 图片代码插入到编辑器中
    /// 它会找到当前光标位置并进行插入
    pub fn insert_image_markdown(&mut self, ctx: &egui::Context, file_path: &std::path::Path) {
        // `main_editor_id` 必须与 `panels.rs` 中 TextEdit 的 id_source 相同
        let editor_id = egui::Id::new("main_editor_id");
        
        // 尝试加载编辑器的状态以获取光标位置
        if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
            let filename = file_path.file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_else(|| "image".into());

            // 生成 Markdown 格式的图片引用，并在末尾添加换行
            let markdown_image = format!("![{}]({})\n", filename, file_path.to_string_lossy());
            
            // 获取光标位置，如果没有光标则插入到文本末尾
            let current_pos = state.cursor.char_range().map(|r| r.primary.index).unwrap_or(self.markdown_text.chars().count());
            
            // 将字符串切片并插入新文本
            let text = self.markdown_text.clone();
            let chars = text.chars().collect::<Vec<_>>();
            let (prefix, suffix) = chars.split_at(current_pos);
            self.markdown_text = prefix.iter().collect::<String>() + &markdown_image + &suffix.iter().collect::<String>();
            
            // 将光标移动到新插入文本之后（包括换行符），以便连续拖入多张图片时能正确插入
            let new_cursor_pos = current_pos + markdown_image.chars().count();
            let new_cursor = egui::text::CCursor::new(new_cursor_pos);
            let new_range = egui::text::CCursorRange::one(new_cursor);
            state.cursor.set_char_range(Some(new_range));
            egui::TextEdit::store_state(ctx, editor_id, state);
        } else {
            // 如果无法获取编辑器状态（例如编辑器没有焦点），则直接在文档末尾添加
            let filename = file_path.file_name()
                .map(|s| s.to_string_lossy())
                .unwrap_or_else(|| "image".into());
            self.markdown_text.push_str(&format!("\n\n![{}]({})\n", filename, file_path.to_string_lossy()));
        }
    }
}