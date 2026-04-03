# My Mind

一个轻量级的智能语音转文字桌面应用，专为 macOS 设计。按下快捷键，说话，你的语音会被自动转录、经 LLM 润色后粘贴到当前活跃的应用中。

## 工作原理

1. 按下 **Option+Space** 激活录音
2. 对着麦克风说话
3. 松开按键（按住说话模式）或再次按下（切换模式）
4. 音频发送至 ASR 服务进行语音识别
5. 原始转录文本经 LLM 润色（去除口语化表达、修正错误、添加标点）
6. 最终文本自动粘贴到之前聚焦的应用中

## 功能特性

- **全局快捷键** - 随时随地按 Option+Space 触发录音，无需切换窗口
- **双录音模式** - 支持"按住说话"和"按键切换"两种模式
- **在线语音识别** - 兼容 Whisper API 的语音识别，支持 OpenAI、SiliconFlow 等服务商
- **LLM 智能后处理** - 由 OpenAI 或 Anthropic 兼容 API 驱动的转录文本优化
  - 去除口头禅、结巴重复和自我纠正
  - 上下文驱动的智能纠错（同音字、字母数字混淆等）
  - 自动添加标点和合理分段
  - 自适应输出：闲聊保持简洁，长文本自动结构化（步骤列表、要点加粗等）
- **自定义 Prompt** - 在设置界面自定义 LLM 后处理 Prompt，或使用内置默认 Prompt
- **自动粘贴** - 处理完成后自动粘贴到之前活跃的应用（微信、浏览器、编辑器等）
- **错误反馈** - 粘贴失败时浮层显示错误提示，文本保留在剪贴板供手动粘贴
- **焦点管理** - 录音前捕获当前活跃应用，处理完成后自动恢复焦点
- **历史记录** - 所有转录结果本地保存（SQLite），可在独立窗口中浏览、复制、删除历史记录
- **提示词诊断** - 当发现转录结果有问题时，可直接在历史记录中与 LLM 对话诊断问题，并获得提示词改进建议
- **设置界面** - 从系统托盘菜单打开，可视化配置 ASR、LLM、Prompt、快捷键等参数
- **macOS 原生体验** - HudWindow 毛玻璃浮层、系统托盘集成、辅助功能权限自动检测、CGEvent 输入模拟

## 技术栈

- **后端**：Rust + [Tauri v2](https://tauri.app/)
- **前端**：[SolidJS](https://www.solidjs.com/) + [Tailwind CSS](https://tailwindcss.com/) + TypeScript
- **音频**：CPAL（采集）+ Rubato（重采样）+ Hound（WAV 编码）
- **语音识别**：Whisper API 兼容（REST，multipart 上传）
- **语言模型**：OpenAI / Anthropic API 兼容
- **数据存储**：SQLite via rusqlite（内置编译，无系统依赖）

## 快速开始

### 前置要求

- macOS 10.15+
- Rust 1.85+（通过 [rustup](https://rustup.rs/) 安装）
- Node.js 18+ 和 [pnpm](https://pnpm.io/)
- Tauri CLI：`cargo install tauri-cli`

### 配置

创建配置文件 `~/.config/my-mind/config.toml`：

```toml
[asr.online]
api_key = "your-asr-api-key"
api_base_url = "https://api.siliconflow.cn/v1"  # 或任何 Whisper 兼容的接口
model = "TeleAI/TeleSpeechASR"                  # 可选

[llm]
provider = "openai"
api_key = "your-llm-api-key"
api_base_url = "https://api.moonshot.cn/v1"      # 或任何 OpenAI 兼容的接口
model = "kimi-k2-turbo-preview"                  # 可选，默认：gpt-4o-mini
temperature = 0.3
enabled = true
prompt = ""  # 留空使用内置默认 Prompt，或填写自定义系统提示词

[shortcuts]
record = "Alt+Space"
mode = "push_to_talk"  # 或 "toggle"

[output]
auto_paste = true
```

也可以通过系统托盘菜单中的 **Settings** 进行可视化配置。

### 构建与运行

```bash
# 开发模式
pnpm install
cargo tauri dev

# 生产构建
cargo tauri build
```

构建产物位于 `target/release/bundle/macos/My Mind.app`。

### 权限

首次启动时，macOS 会请求以下权限：
- **麦克风** 权限（用于录音）
- **辅助功能** 权限（用于模拟 Cmd+V 粘贴）

## 项目结构

```
my-mind/
├── crates/
│   ├── my-mind-core/        # 核心库：音频、ASR、LLM、流水线、历史存储
│   └── my-mind-tauri/       # Tauri 命令、状态、事件
├── packages/
│   └── web/                 # SolidJS 前端（录音浮层 + 设置界面 + 历史记录）
├── src-tauri/               # Tauri 应用入口、初始化、托盘、快捷键
└── Cargo.toml               # Rust workspace 根配置
```

## 数据存储

- **配置文件**：`~/.config/my-mind/config.toml`
- **历史记录**：`~/.config/my-mind/history.db`（SQLite）

## 未来规划

- [ ] 离线语音识别（本地 Whisper 模型）
- [ ] 快捷键热更新（修改后无需重启）
- [ ] 流式语音识别，实时显示转录文本
- [ ] 多语言自动检测
- [ ] Windows 和 Linux 平台支持
- [x] 历史记录搜索与导出
- [x] 历史记录提示词诊断

## 许可证

[MIT](LICENSE)
