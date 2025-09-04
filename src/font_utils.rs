use eframe::egui;

pub fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    if let Some(chinese_font_data) = load_system_chinese_font() {
        fonts.font_data.insert("chinese".to_owned(), chinese_font_data.into());
        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
        fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".to_owned());
    }
    
    ctx.set_fonts(fonts);
}

fn load_system_chinese_font() -> Option<egui::FontData> {
    // 根据不同操作系统，从系统路径加载合适的中文字体
    #[cfg(target_os = "windows")]
    {
        let font_paths = [
            r"C:\Windows\Fonts\msyh.ttc",      // Microsoft YaHei
            r"C:\Windows\Fonts\simsun.ttc",     // SimSun
            r"C:\Windows\Fonts\msyhbd.ttc",     // Microsoft YaHei Bold
            r"C:\Windows\Fonts\simhei.ttf",     // SimHei
            r"C:\Windows\Fonts\simkai.ttf",     // KaiTi
        ];
        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                // 使用 from_owned，因为我们是从文件动态读取的
                return Some(egui::FontData::from_owned(font_data));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let font_paths = [
            "/System/Library/Fonts/PingFang.ttc", // PingFang SC
            "/System/Library/Fonts/STHeiti Medium.ttc", // Heiti SC
            "/Library/Fonts/Arial Unicode.ttf", // Fallback
        ];
        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                return Some(egui::FontData::from_owned(font_data));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux 的字体路径和名称可能因发行版而异，这里尝试一些常见路径
        let font_paths = [
            // Noto CJK
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto/NotoSansCJK-Regular.ttc",
            // WenQuanYi
            "/usr/share/fonts/wenquan-micro-hei/wqy-microhei.ttc",
            "/usr/share/fonts/wqy-microhei/wqy-microhei.ttc",
            // Droid Sans Fallback
            "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        ];
        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                return Some(egui::FontData::from_owned(font_data));
            }
        }
    }

    // 如果在所有特定平台的路径下都找不到字体，则返回 None
    None
}
