use std::collections::HashMap;
use crate::font_utils;

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
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        font_utils::setup_chinese_fonts(&cc.egui_ctx);

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
        }
    }
}