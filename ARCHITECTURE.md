# 软件架构

本文档概述了“文档风格转换器”应用的软件架构。

## 目录结构

`src` 目录被组织为多个模块，每个模块都有其特定的职责。

```
src/
├── app.rs          # 主应用循环与事件处理
├── file_handler.rs # 文件I/O操作 (打开, 保存, 合并)
├── font_utils.rs   # 跨平台字体加载工具
├── main.rs         # 应用入口点
├── pandoc.rs       # 与Pandoc命令行工具交互的逻辑
├── state.rs        # 应用状态结构体 (`MyApp`) 与构造函数
└── ui/             # UI组件
    ├── dialogs.rs  # 所有对话框窗口的逻辑
    ├── menu.rs     # 顶部菜单栏渲染逻辑
    ├── mod.rs      # UI模块声明
    └── panels.rs   # 编辑器与预览面板的渲染逻辑
```

## 模块说明

### `main.rs`

这是应用程序的入口点。它负责初始化 `eframe` 窗口，并通过创建 `MyApp` 的新实例来启动应用。它也声明了整个项目的所有顶层模块。

### `state.rs`

该模块定义了应用的核心状态，这些状态存储在 `MyApp` 结构体中。它还包含了 `new()` 函数，作为 `MyApp` 的构造函数，负责设置应用的初始状态。

### `app.rs`

该模块包含了主应用逻辑。它为 `MyApp` 实现了 `eframe::App` trait。其中的 `update()` 函数作为应用的主循环，在每一帧都会被调用。它负责：
- 处理用户输入和键盘快捷键。
- 调用不同UI组件的渲染函数。
- 管理不同窗口和对话框的可见性。

### `file_handler.rs`

该模块负责所有与用户文件相关的直接文件系统操作，包括：
- `load_file()`: 从磁盘打开并读取一个Markdown文件。
- `save_file()`: 将当前的Markdown内容保存到文件。
- `merge_files()`: 将多个Markdown文件合并成一个。

### `pandoc.rs`

该模块封装了所有与 `pandoc` 命令行工具的交互。这些可能是长时间运行的操作会在独立的线程上执行，以避免阻塞UI。
- `import_from_docx()`: 将一个 `.docx` 文件转换为Markdown。
- `export_as_docx()`: 将当前的Markdown文本转换为一个 `.docx` 文件。
- `set_reference_doc()`: 加载一个 `.docx` 文件作为样式参考，并解析其中的自定义段落和字符样式。
- `check_for_*_result()`: 用于从后台线程检查结果的辅助函数。

### `font_utils.rs`

这个工具模块提供了定位和设置系统原生中日韩（CJK）字体。这确保了中、日、韩字符在不同操作系统（Windows, macOS, Linux）上都能正确显示。

### `ui/` 模块

该目录包含了所有与渲染用户界面相关的代码。

#### `ui/menu.rs`

- `show_menu_bar()`: 渲染应用窗口顶部的菜单栏。

#### `ui/panels.rs`

- `show_panels()`: 渲染应用的中心区域，该区域被分为两列：左侧是文本编辑器，右侧是Markdown预览。它也处理同步滚动的逻辑。
- `apply_formatting_to_selection()`: 一个辅助函数，用于将Markdown格式（如粗体、斜体）应用到编辑器中的选定文本。

#### `ui/dialogs.rs`

该模块包含了渲染所有弹出窗口和对话框的逻辑：
- `show_about_window()`: 渲染“关于”窗口。
- `show_assignment_window()`: 渲染用于为 `{{placeholder}}` 标记赋值的窗口。
- `show_style_palette()`: 渲染用于搜索和应用来自参考DOCX文件的自定义样式的命令面板。