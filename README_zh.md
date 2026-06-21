# rdice

[English](README.md) | 中文

`rdice` 是一个 Rust 骰子项目，公开部分采用 workspace 管理，包含可复用核心库、命令行工具和终端 UI 工具。

## 子项目

| 项目 | 类型 | Cargo 包名 | 产物 | 发布目标 |
| --- | --- | --- | --- | --- |
| `rdice-core` | 库 | `rdice-core` | Rust crate `rdice_core` | crates.io |
| `rdice-cli` | CLI | `rdice-cli` | `rdice` 可执行文件 | crates.io |
| `rdice-tui` | TUI | `rdice-tui` | `rdice-tui` 可执行文件 | crates.io |

私有应用项目，例如 `rdice-app` 和 `rdice-web`，建议放在独立的私有仓库中，并依赖 crates.io 上发布的 `rdice-core`。

## 目录结构

```text
.
├── Cargo.toml
├── LICENSE
├── README.md
├── README_zh.md
├── crates/
│   ├── rdice-core/
│   ├── rdice-cli/
│   └── rdice-tui/
└── docs/
    ├── ARCHITECTURE.md
    ├── RELEASING.md
    ├── plan/
    └── spec/
```

## 安装

安装命令行掷骰工具：

```sh
cargo install rdice-cli
rdice roll 3d6
```

安装终端 UI：

```sh
cargo install rdice-tui
rdice-tui
```

在其他 Rust 项目中使用核心库：

```toml
[dependencies]
rdice-core = "0.1"
```

## 构建与测试

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## CLI 用法

```sh
rdice roll [-f|--folded] [-x|--expanded] [-E|--ev] [-R|--range] <dice-expr...>
rdice [-E|--ev] [-R|--range] <dice-expr...>
rdice list
rdice config path
rdice config edit
rdice help [--no-color]
```

示例：

```sh
rdice roll 5d6
rdice roll -x 4d6
rdice roll -f -E -R 3d13 2coin 5 -3
rdice -E -R 3d6 5 -3
```

CLI 默认使用 ANSI 颜色输出。脚本或纯文本日志场景可传入 `--no-color`，或设置
`NO_COLOR` 禁用颜色；TUI 也会遵循 `NO_COLOR`。

CLI 优先读取 `RDICE_CONFIG_PATH` 指定的配置文件。未设置时默认使用：

```text
~/.config/rdice/config.toml
```

配置示例：

```toml
[[dice]]
name = "coin"
faces = ["heads", "tails"]

[[dice]]
name = "fate"
faces = [-1, 0, 1]
```

## TUI

`rdice-tui` 是交互式虚拟骰盘工作台，支持持久化骰盘、重复掷骰、锁定 slot，以及紧凑的键盘操作。

TUI 状态路径可通过 `RDICE_TUI_STATE_PATH` 覆盖，默认使用：

```text
~/.local/state/rdice-tui/state.toml
```

启动：

```sh
rdice-tui
```

## 文档

- [架构说明](docs/ARCHITECTURE.md)
- [发布流程](docs/RELEASING.md)
- [TUI 设计文档](docs/spec/2026-04-27-tui-design.md)
- [TUI 实现计划](docs/plan/2026-04-27-tui-implementation-plan.md)

## 许可证

本项目使用 [MIT License](LICENSE)。
