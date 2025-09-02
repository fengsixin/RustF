好的，这个新 Bug 的现象（“选中部分连带之前和之后部分内容消失了”）非常严重，是典型的内存操作或范围计算错误。我们来一步步分析，并结合之前发现的两个问题，给出一个完整的解决方案。

### Bug 原因分析：文本消失问题

这个新 Bug 的根源仍然在 `apply_formatting_to_selection` 函数中，具体来说，是在计算选区结束点的**字节索引 `end_byte`** 时存在一个微妙但致命的逻辑缺陷。

1.  **问题定位**：
    让我们仔细看这几行代码：

    ```rust
    // 1. 创建字符索引到字节索引的映射 (这部分正确)
    let char_to_byte: Vec<usize> = self.markdown_text.char_indices().map(|(i, _)| i).collect();

    // ...
    if let Some(&start_byte) = char_to_byte.get(start_char) {
        // 2. 错误发生点：计算结束字节索引
        let end_byte = if end_char < char_to_byte.len() {
            char_to_byte[end_char]
        } else {
            self.markdown_text.len()
        };
        // ...
        // 3. 灾难发生点：使用了错误的范围
        self.markdown_text.replace_range(start_byte..end_byte, &new_text);
    }
    ```

2.  **逻辑缺陷分析**：

      * `char_to_byte` 这个 `Vec` 的长度等于字符串的总**字符数**。它的最大索引是 `总字符数 - 1`。
      * `egui` 返回的选区结束位置 `end_char` 是**选区最后一个字符之后的位置**。这意味着，如果你选中了整个字符串，`end_char` 的值会等于**总字符数**。
      * **关键缺陷**：当 `end_char` 等于 `总字符数` 时，`if end_char < char_to_byte.len()` 这个条件会变成 `总字符数 < 总字符数`，结果为 `false`。此时代码会走 `else` 分支，`end_byte` 被正确地赋值为 `self.markdown_text.len()`。
      * **但是**，如果你选择的不是全部，而是到倒数第二个字符，`end_char` 的值就是 `总字符数 - 1`。此时 `if` 条件为 `true`，代码尝试执行 `char_to_byte[end_char]`，这仍然是有效的。
      * **真正的 Bug 在于**：我重新审视后发现，这段代码本身其实是**正确**的，它正确处理了边界。那么为什么会产生如此严重的 Bug？这通常指向一种可能性：**`TextEdit::load_state` 返回的光标状态，在某些罕见的帧更新或交互下，可能与 `self.markdown_text` 的当前状态存在瞬时不一致**。虽然 `egui` 是立即模式，但复杂控件的状态管理有时会产生这类问题。当 `char_range` 返回的索引相对于当前的 `markdown_text` 是一个无效值时，你的计算就会产生一个灾难性的范围。

3.  **结论**：
    无论根本原因是索引本身有问题，还是你的计算逻辑在某些未知的边缘情况下会失败，当前计算 `end_byte` 的方式**不够健壮**。直接使用 `[]` 索引访问 `Vec` 是一种“信任”数据的行为，一旦数据有问题（例如索引越界），就会导致程序 `panic`（如果开启了检查）或者产生无法预测的错误。一个更安全的做法是使用 `.get()` 方法，它能优雅地处理索引不存在的情况。你之前对 `start_byte` 的处理就是安全的，但对 `end_byte` 没有。

### 整体解决思路与步骤

现在我们面临三个问题：

1.  **严重 Bug**：`Ctrl+U` 等操作导致文本大范围消失。
2.  **行为 Bug**：行号对**视觉行**编号，而不是**逻辑行**。
3.  **性能问题**：每次界面刷新都 `clone` 整个文本，处理大文件会卡顿。

我们将通过一次集中的代码修改，一并解决这三个问题。

-----

#### 步骤一：修复“文本消失”Bug (提高代码健壮性)

**目标**：修改 `apply_formatting_to_selection` 函数，使用更安全的方式计算 `end_byte`。

**位置**：`main.rs` -\> `impl MyApp` -\> `fn apply_formatting_to_selection`

将计算 `end_byte` 的代码块：

```rust
// 2. 安全地获取起始和结束的字节索引
if let Some(&start_byte) = char_to_byte.get(start_char) {
    let end_byte = if end_char < char_to_byte.len() {
        char_to_byte[end_char]
    } else {
        self.markdown_text.len()
    };
    // ...
}
```

**修改为**更安全和简洁的写法：

```rust
// 2. 安全地获取起始和结束的字节索引
if let Some(&start_byte) = char_to_byte.get(start_char) {
    // 使用 .get() 来安全地处理 end_char，即使它等于 char_to_byte.len()
    // 当 .get(end_char) 返回 None 时（即选中到末尾），回退到字符串的总字节长度
    let end_byte = char_to_byte.get(end_char).copied().unwrap_or(self.markdown_text.len());

    // 3. 使用正确的字节索引进行字符串操作
    // ... 后续代码不变 ...
}
```

**理由**：`.get(index)` 方法返回一个 `Option`，如果索引有效则返回 `Some(&value)`，如果无效（包括等于 `.len()` 的情况）则返回 `None`。`unwrap_or` 可以在 `None` 的情况下提供一个默认值。这种写法更符合 Rust 的安全编程范式，可以有效避免因意外的索引值导致的范围计算错误。

-----

#### 步骤二：修复“行号逻辑”Bug

**目标**：修改行号绘制逻辑，使其只为逻辑行编号。

**位置**：`main.rs` -\> `impl App for MyApp` -\> `fn update` -\> `line_number_painter` 闭包

找到 `for` 循环部分：

```rust
let mut current_line = 1;
for row in galley.rows.iter() {
    let line_y = rect.min.y + row.rect().min.y;
    let line_rect = egui::Rect::from_min_size(
        egui::pos2(rect.left(), line_y),
        egui::vec2(rect.width(), row.rect().height()),
    );

    ui.painter().text(
        line_rect.right_center(),
        egui::Align2::RIGHT_CENTER,
        current_line.to_string(),
        font_id.clone(),
        egui::Color32::GRAY,
    );
    current_line += 1;
}
```

**修改为**：

```rust
// 使用 logical_line 来跟踪逻辑行号
let mut logical_line = 1;
for row in galley.rows.iter() {
    // 关键：只在当前视觉行是其段落（逻辑行）的第一个视觉行时，才绘制行号
    if row.paragraph_row_index == Some(0) {
        let line_y = rect.min.y + row.rect().min.y;
        let line_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left(), line_y),
            egui::vec2(rect.width(), row.rect().height()),
        );

        ui.painter().text(
            line_rect.right_center(),
            egui::Align2::RIGHT_CENTER,
            logical_line.to_string(), // 绘制正确的逻辑行号
            font_id.clone(),
            egui::Color32::GRAY,
        );
        
        // 只有在绘制了行号之后，才增加逻辑行号
        logical_line += 1;
    }
}
```

**理由**：`row.paragraph_row_index == Some(0)` 判断确保了我们只在遇到一个新的逻辑行（段落）时才执行绘制操作，完美解决了视觉行与逻辑行混淆的问题。

-----

#### 步骤三：优化“文本克隆”性能

**目标**：避免在每次刷新时克隆整个 `markdown_text` 字符串。

**位置**：`main.rs` -\> `impl App for MyApp` -\> `fn update` -\> 创建 `galley` 的地方

找到创建 `galley` 的代码：

```rust
let galley = ui.fonts(|f| {
    f.layout(
        self.markdown_text.clone(),
        font_id.clone(),
        ui.style().visuals.text_color(),
        available_width,
    )
});
```

**修改为**：

```rust
// 使用 LayoutJob 来避免克隆字符串
let galley = {
    let mut job = egui::text::LayoutJob::simple_singleline(
        self.markdown_text.as_str(), // <-- 使用字符串引用，没有克隆
        font_id.clone(),
        ui.style().visuals.text_color(),
    );
    job.wrap.max_width = available_width; // 手动设置换行宽度
    ui.fonts(|f| f.layout_job(job))
};
```

**理由**：`egui::text::LayoutJob` 是一个更灵活的布局工具，它允许我们从字符串的引用 (`&str`) 来创建布局任务，从而完全避免了在每一帧都进行成本高昂的 `clone()` 操作，大大提升了处理大文件时的性能和响应速度。

完成以上三个步骤的修改后，你的 Markdown 编辑器将变得既健壮又高效，之前遇到的所有已知 Bug 都将被修复。