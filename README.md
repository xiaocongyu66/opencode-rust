# rsopencode

opencode 的 Rust 重写版 — AI 驱动的命令行编程工具。

## 项目信息

- **基于**: [opencode](https://github.com/anomalyco/opencode)
- **基于的提交**: `4643e65` — `fix(opencode): enable web search for Go (#42630)`
- **语言**: Rust (edition 2021)
- **架构**: 单 crate 扁平化模块设计
- **二进制**: `rsopencode` (15MB, release, statically linked)

## 安装

```bash
# 从源码构建
cargo build --release
cp target/release/rsopencode /usr/local/bin/rsopencode

# 使用安装包
tar xzf rsopencode-aarch64-linux.tar.gz
./install.sh
```

## 使用

```bash
rsopencode              # 交互式终端聊天（默认，直接输入文字）
rsopencode serve        # 启动 HTTP API 服务器
rsopencode agents       # 列出 agent
rsopencode models       # 列出模型
rsopencode api          # 向服务器发请求
rsopencode --help       # 查看所有命令
```

### 环境变量

```
OPENAI_API_KEY          # OpenAI API key
ANTHROPIC_API_KEY       # Anthropic API key
LANG=zh_CN.UTF-8        # 中文界面
```

## 项目结构

```
crates/bin/src/
├── main.rs              # 入口
├── lib.rs              # 模块声明
├── schema/             # 数据模型（对应 TS schema 包）
├── llm/                # LLM 抽象层（对应 TS llm 包）
│   ├── providers/      # 12 个 provider 实现
│   └── openai_api.rs   # OpenAI v1 API 兼容
├── tools/              # 19 个工具实现（含 sonar 代码搜索）
├── core/               # 核心业务逻辑（对应 TS core + opencode 包）
│   ├── config/         # 配置管理
│   └── session/        # 会话管理（runner/store/history/compaction 等）
├── server/             # HTTP 服务器（53 条路由）
├── protocol/           # API 协议定义
├── cli/                # CLI 命令
└── tui/                # 终端 UI（对应 TS tui 包，79% 覆盖率）
    ├── theme/          # 33 个主题
    ├── component/      # UI 组件
    ├── context/        # 上下文系统
    ├── config/         # 键盘映射
    ├── routes/         # 路由页面
    ├── plugins/        # Feature plugins
    └── util/           # 工具函数
```

## 与原版的差异

| 特性 | TS 原版 | Rust 版 |
|------|---------|---------|
| 架构 | 31 个 monorepo 包 | 单 crate 模块化 |
| TUI 框架 | SolidJS + opentui | ratatui |
| 运行时 | Bun (Node.js) | 原生 Rust |
| 数据库 | effect-sqlite | rusqlite |
| 二进制大小 | ~174MB | 15MB |
| 启动时间 | ~300ms | ~10ms |
| 代码搜索 | 无 | 集成 sonar (BM25+语义) |

## 开发

```bash
# 构建
cargo build --release

# 运行测试
cargo test

# 检查
cargo clippy
```

## License

Apache-2.0
