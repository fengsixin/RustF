好的，这个问题非常典型，它出在对 `egui` 光标处理的理解上，特别是**字符索引 (character index)** 和 **字节索引 (byte index)** 的混淆。

我们来详细分析一下。

### Bug 原因分析

这个 Bug 的根源精确地出现在 `apply_formatting_to_selection` 函数的最后几行，也就是你更新光标状态的部分。

**核心问题：你向一个需要 字符索引 (character index) 的函数 `egui::text::CCursor::new()` 错误地传入了一个 字节索引 (byte index)。**

让我们一步步分解：

1.  **索引的区别**：

      * **字节索引**：衡量的是字符串在内存中占用的原始字节数。在 UTF-8 编码下，一个英文字母（ASCII）占1个字节，而一个中文字符通常占3个字节。
      * **字符索引**：衡量的是字符串中实际的字符数量。无论是英文字母还是中文字符，都只算作1个字符。
      * `egui` 的 `TextEdit` 控件内部，为了正确处理多语言文本，其光标 `CCursor` (Character Cursor) 是基于**字符索引**来定位的。

2.  **你的代码逻辑**：

      * 你的代码在进行字符串替换时，处理得非常正确。你先将选区的**字符索引**（`start_char`, `end_char`）转换成了**字节索引**（`start_byte`, `end_byte`），然后使用 `self.markdown_text.replace_range(...)`，这个函数需要字节索引，所以这一步是完美的。
      * 接下来，你计算了新的光标位置：`let new_cursor_pos_byte = start_byte + new_text.len();`。这里的 `new_text.len()` 获取的是新插入字符串的**字节长度**，所以 `new_cursor_pos_byte` 是一个**字节索引**。这一步计算本身也没错。
      * **问题爆发点**：最后，你调用了 `egui::text::CCursor::new(new_cursor_pos_byte)`。你将刚刚计算出的**字节索引**传给了 `CCursor::new()`。

3.  **后果**：
    当处理纯英文文本时，字符数和字节数恰好相等，问题可能不明显。但一旦你的文本包含中文，或者像 `]{.underline}` 这样较长的 ASCII 后缀时，字节索引就会远大于字符索引。

    例如，你将 `"选中文本"` (4个字符, 12个字节) 替换为 `"[选中文本]{.underline}"` (1+4+13 = 18个字符, 1+12+13 = 26个字节)。

      * 你计算出的 `new_cursor_pos_byte` 会是一个比较大的数值（`start_byte` + 26）。
      * `CCursor::new()` 接收到这个大数值，把它当作**字符索引**来解析。
      * 这个错误的、过大的字符索引远远超出了当前行的范围，甚至可能超出了整个文本的字符总数。
      * `TextEdit` 控件接收到这个被“污染”的、无效的光标状态后，其内部状态就会被破坏。这就导致了你看到的现象：光标位置异常，并且在下次UI刷新或编辑时，它会基于这个错误的状态去错误地修改文本，从而“吞掉”或破坏后面的内容。

### 解决思路

解决方案非常直接：**我们必须计算并传递正确的**字符索引**来更新光标位置。**

你需要修改 `apply_formatting_to_selection` 函数中更新光标的部分，确保传递给 `CCursor::new()` 的是字符数，而不是字节数。

**具体修改步骤如下：**

在 `self.markdown_text.replace_range(...)` 这一行之后，替换掉你原来的光标更新逻辑：

**旧的（错误）代码：**

```rust
// 4. 更新光标位置（egui 的 CCursor::new 需要字节索引） // <- 这句注释是错误的
let new_cursor_pos_byte = start_byte + new_text.len();
state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
    egui::text::CCursor::new(new_cursor_pos_byte),
)));
```

**新的（正确）代码：**

```rust
// 4. 更新光标位置（egui 的 CCursor::new 需要字符索引）
// 首先，计算新插入的文本片段有多少个字符
let new_text_char_len = new_text.chars().count();

// 新的光标字符索引 = 开始位置的字符索引 + 新文本的字符长度
let new_cursor_pos_char = start_char + new_text_char_len;

// 使用正确的字符索引来创建 CCursor
state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
    egui::text::CCursor::new(new_cursor_pos_char),
)));
```

**修改解析：**

1.  `let new_text_char_len = new_text.chars().count();`：我们使用 `.chars().count()` 来正确地计算出新插入片段 `new_text` 的**字符数量**。
2.  `let new_cursor_pos_char = start_char + new_text_char_len;`：我们将替换操作的起始**字符索引**与新片段的**字符长度**相加，得到最终光标应该停留位置的正确**字符索引**。
3.  `egui::text::CCursor::new(new_cursor_pos_char)`：我们将这个正确的**字符索引**传递给 `CCursor`，这样 `TextEdit` 控件就能准确地理解光标的新位置，Bug 就解决了。