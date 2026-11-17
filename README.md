# Kiroshi

热键呼出的快捷键 / 命令速查框（Windows，Tauri 2 + React）。

按 `Alt+Space` 呼出，输入几个字搜索，`Enter` 把按键或命令复制到剪贴板并隐藏窗口。常驻托盘，失去焦点自动隐藏。

## 使用

| 按键 | 作用 |
|---|---|
| `Alt+Space` | 呼出 / 隐藏（全局） |
| 直接输入 | 搜索按键、命令、作用、应用名、标签；空格分隔的词需同时匹配 |
| `↑` `↓`（或 `Ctrl+J` / `Ctrl+K`） | 选择 |
| `Enter`（或双击） | 复制第一段按键 / 命令，并隐藏 |
| `Tab` / `Shift+Tab` | 在有匹配结果的应用之间切换 |
| `Ctrl+N` | 新增条目 |
| `Ctrl+E` | 编辑选中的条目（删除也在这里） |
| `Esc` | 依次：清空搜索 → 回到“全部” → 隐藏 |

搜索区分按键大小写的优先级：搜 `D` 时 `D` 排在 `d` 前面，反之亦然。

退出在托盘图标的右键菜单里。`kiroshi.exe --hidden` 启动时只进托盘，适合开机自启。

## 数据

都在 `%APPDATA%\com.elmtran.kiroshi\`（托盘菜单“打开数据文件夹”）：

- `shortcuts.json`：全部条目，可以直接手改，下次呼出窗口时生效。文件解析失败时应用只报错，不会覆盖它。
- `settings.json`：`{ "hotkey": "Alt+Space" }`，改完重启生效。热键被别的程序占用时回退到 `Ctrl+Alt+K`。

一条数据：

```json
{
  "id": "seed-101",
  "app": "Vim",
  "kind": "key",
  "keys": "`Ctrl-W` `q`",
  "desc": "关闭当前分屏",
  "tags": ["分屏"]
}
```

`kind` 是 `key`（按键，显示成键帽）或 `command`（命令，显示成终端文字）。`keys` 里用反引号包住真正要按 / 要输入的部分，其余文字当作说明。

首次运行时的初始数据来自 `src-tauri/seed/shortcuts.json`，由 Markdown 速查表生成：

```bash
pnpm seed <速查表.md>
```

`##` 标题作为应用，`###` 标题作为标签，每个两列表格的一行是一条。它只更新种子文件；已经存在的 `shortcuts.json` 不会被改动。

## 开发

```bash
pnpm install
```

```bash
pnpm tauri dev
```

```bash
pnpm test
```

```bash
pnpm tauri build
```

- `pnpm dev` 可以只在浏览器里跑界面（用种子数据，改动不保存）。
- 设置环境变量 `KIROSHI_KEEP_OPEN=1` 后窗口失焦不隐藏，方便调试。
- 前端 `src/`：`search.ts`（搜索与排序）、`App.tsx`、`Editor.tsx`、`api.ts`（Tauri 命令封装）。
- 后端 `src-tauri/src/`：`store.rs`（JSON 读写）、`window.rs`（窗口、托盘、热键）、`commands.rs`。

## 后续

- 快速输入：选中后直接把文本打进之前的窗口，而不只是复制。
- AI 集成：用自然语言找快捷键，从文档自动整理条目。
