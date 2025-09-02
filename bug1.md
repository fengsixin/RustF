好的，我们来分析一下你代码中行号与内容错位的 Bug。

### Bug 原因分析

这个 Bug 的根本原因在于，**行号的绘制与文本编辑器的内容渲染是完全分离的两个过程**，并且你对文本高度的计算方式过于理想化。

1.  **错误的假设：行高是固定的。**
    你的代码通过 `ctx.fonts(|f| f.row_height(&monospace_font))` 获取了一个标准的、固定的行高 `row_height`。然后，你通过 `(i - 1) as f32 * row_height` 来计算每一行行号的 Y 轴坐标。这个计算方式**严格依赖于一个假设：编辑器中的每一行文本都精确地占据 `row_height` 的高度**。

2.  **现实情况：`TextEdit` 的行高是动态的。**
    `egui::TextEdit` 是一个非常复杂的控件，它自身的布局逻辑远比“固定行高”要复杂。最关键的一点是**文本折行 (Word Wrapping)**。当你在编辑器中输入一个很长的、没有换行的段落时，`TextEdit` 会自动将其折成多行来显示。这时，一个**逻辑行**（以 `\n` 分隔）会占用多个**物理行**的高度。

3.  **错位产生。**
    当文本发生折行时，你的行号绘制逻辑依然只为这个逻辑行分配了一个 `row_height` 的高度，然后就去绘制下一行的行号了。但与此同时，`TextEdit` 控件为了显示折行的文本，实际使用了两倍、三倍甚至更多的垂直空间。
    这就导致了：

      * **初期对齐**：在没有长文本折行的情况下，行号和内容看起来是对齐的。
      * **遇到长文本后开始错位**：一旦出现折行，编辑器内容区域的高度被撑开，而行号区域的高度没有，后续所有行号的位置就都发生了偏移，并且这个偏移会随着折行文本的增多而不断累积，导致越来越“离谱”。

总结一下，你手动维护了一套行号布局系统，而这套系统与 `TextEdit` 内部的实际布局系统（尤其是对折行的处理）完全脱节，从而导致了视觉上的错位。

### 解决思路和步骤

核心思路是**放弃手动绘制行号**，而是利用 `egui` 更高级的文本布局功能，将**行号作为文本内容的一部分**，与代码一起交给 `egui` 的布局引擎处理。这样可以保证行号和其对应的内容行永远在同一个布局单元里，无论是否折行，都能完美对齐。

我们将使用 `TextEdit` 控件的一个强大功能：`layouter` 方法。它允许你提供一个自定义的函数来接管文本的布局过程。

**具体解决步骤如下：**

1.  **移除所有手动绘制行号的代码。**

      * 删除 `ui.horizontal` 布局中用于显示行号的整个 `egui::Frame::new().show(...)` 代码块。
      * 删除所有与计算行号宽度、高度、位置相关的逻辑，例如 `row_height`, `char_width`, `line_count`, `line_number_width` 以及 `ui.painter().text(...)` 的循环。
      * 由于不再需要 `ui.horizontal` 来分割行号和编辑器，也可以将其移除，直接在 `ScrollArea` 中放置 `TextEdit` 即可。

2.  **创建一个自定义的 `layouter` 函数。**
    你需要定义一个函数（或者闭包），它的签名大致如下：
    `fn line_number_layouter(ui: &egui::Ui, text: &str, wrap_width: f32) -> std::sync::Arc<egui::Galley>`
    这个函数会在每一帧被 `TextEdit` 调用，你需要在这个函数里完成文本的布局工作。

3.  **在 `layouter` 函数内部实现新的布局逻辑。**
    这是最关键的一步。在这个函数里，你需要：

      * **创建一个 `egui::text::LayoutJob`**。`LayoutJob` 是一个强大的结构，它允许你将一段文本描述为多个片段（`TextFragment`），每个片段都可以有不同的颜色、字体、下划线等样式。
      * **按行分割输入文本**。将传入的 `text: &str` 按 `\n` 分割成独立的行。
      * **遍历每一行文本**，对于每一行：
          * **创建行号 `TextFragment`**：
              * 计算当前行号。
              * 创建一个 `egui::text::TextFragment`。它的文本内容是格式化后的行号（例如 `format!("{:<4} ", line_number)`，使用空格进行右对齐和分隔）。
              * 设置这个 `TextFragment` 的颜色为灰色（`egui::Color32::GRAY`）。
              * 设置字体为等宽字体（`egui::FontId::monospace(...)`）。
          * **创建内容 `TextFragment`**：
              * 创建另一个 `TextFragment`，其文本内容是当前行的实际内容。
              * 使用默认颜色和等宽字体。
          * **将两个 `TextFragment` 添加到 `LayoutJob` 中**。先添加行号的，再添加内容的。
          * **添加换行符**。在每行内容（除了最后一行）的 `TextFragment` 结尾加上 `\n`，以确保 `LayoutJob` 能正确处理换行。
      * **设置 `LayoutJob` 的 `wrap.max_width`**。将函数参数中的 `wrap_width` 传递给 `LayoutJob`，这样 `egui` 就能知道在何处进行自动折行。
      * **使用 `ui.fonts()` 来最终生成 `Galley`**。调用 `ui.fonts(|f| f.layout_job(job))` 将你精心构造的 `LayoutJob` 转换成一个 `egui` 可以直接渲染的 `Galley` 对象。
      * **返回 `Galley`**。将生成的 `Arc<egui::Galley>` 作为函数的返回值。

4.  **将自定义 `layouter` 应用到 `TextEdit` 控件。**
    在你的 `update` 函数中，修改 `TextEdit` 的创建代码，调用 `.layouter()` 方法并传入你刚刚创建的布局函数。

    ```rust
    // 伪代码示意
    egui::TextEdit::multiline(&mut self.markdown_text)
        .id(egui::Id::new("main_editor_id"))
        .code_editor()
        .desired_width(f32::INFINITY)
        // 在这里应用你的自定义 layouter
        .layouter(&mut |ui, text, wrap_width| {
            my_custom_layouter(ui, text, wrap_width) // 调用你实现的布局函数
        })
        .show(ui);
    ```

通过以上步骤，你将行号和文本内容的布局完全交给了 `egui` 的核心引擎。当 `egui` 因为折行而增加某行的垂直空间时，与它在同一个 `LayoutJob` 中的行号也会被自然地推到正确的位置，从而从根本上解决了错位问题。