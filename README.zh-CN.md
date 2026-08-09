# jm — 跨平台 JDK / Java 版本管理器

[在线文档](https://shinnosuke0722.github.io/jm/) · [English](README.md)

[![CI](https://github.com/Shinnosuke0722/jm/actions/workflows/ci.yml/badge.svg)](https://github.com/Shinnosuke0722/jm/actions/workflows/ci.yml)
[![最新版本](https://img.shields.io/github/v/release/Shinnosuke0722/jm)](https://github.com/Shinnosuke0722/jm/releases/latest)
[![许可证](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#许可证)

`jm` 是一个用 Rust 编写的原生 JDK / Java 版本管理命令行工具，支持
Linux、macOS 和 Windows。它可以安装多个 OpenJDK 发行版、设置全局默认
Java，并通过 `.java-version` 或 `.sdkmanrc` 中的 `java=` 条目为项目选择 JDK。

## 安装

### Linux 和 macOS

```sh
curl -fsSL https://raw.githubusercontent.com/Shinnosuke0722/jm/main/install.sh | sh
```

安装脚本会从最新 GitHub Release 下载与当前平台匹配的文件，并安装到
`~/.jm/bin`。如果你的安全规范不允许直接执行远程脚本，请先查看
[`install.sh`](install.sh)，再下载到本地执行。

### Windows（PowerShell）

```powershell
irm https://raw.githubusercontent.com/Shinnosuke0722/jm/main/install.ps1 | iex
```

Windows 安装脚本会把 `%USERPROFILE%\.jm\bin` 加入用户 `PATH`。安装完成后请
打开新终端，或执行安装脚本打印的当前终端生效命令。现有 Windows 预编译文件
仅面向 x86-64；PowerShell 配置和 ARM64 说明见 [Windows 指南](docs/windows.md)。

### 从源码安装

需要 Rust 1.97.1 或更高版本：

```sh
cargo install --git https://github.com/Shinnosuke0722/jm.git --locked
```

当发布页同时提供 SHA-256 清单且本机校验工具可用时，Release 安装脚本会尝试
校验所下载的 `jm` 压缩包。

## 快速开始

```sh
# 安装最新匹配的 Temurin JDK 21
jm install 21

# 安装指定发行版
jm install corretto-17

# 设置全局默认版本
jm default 21

# 为当前项目固定一个已经安装的 JDK 21
jm use 21

# 查看当前要求和 Java 版本
jm current
java -version
```

`jm use 21` 会从已安装的 JDK 中选择最新匹配项，然后把完整安装 ID 写入
`.java-version`。如果团队成员需要共享同一项 JDK 要求，可以提交这个文件。

## 核心特点

- **一套跨平台工作流**：在 Linux、macOS 和 Windows 上使用相同的安装、搜索、
  列表、全局默认和项目固定命令。
- **支持多个 JDK 发行版**：通过 [Foojay Disco API](https://foojay.io/) 查询
  Temurin、Corretto、Zulu、Liberica、Microsoft OpenJDK、GraalVM 等发行版。
  当 Disco 不可用时，Temurin 请求可以回退到 Adoptium API。
- **按项目自动选择 Java**：Shell Hook 从当前目录向上查找
  `JM_JAVA_VERSION`、`.java-version` 或 `.sdkmanrc` 中的 Java 条目。
- **运行 `jm` 不需要 JVM**：`jm` 本身是编译后的 Rust 命令行程序。
- **有元数据时执行完整性校验**：默认情况下，如果上游提供 SHA-256，`jm` 会
  校验 JDK 压缩包；如果上游没有提供校验和，则会明确警告后继续。

## Shell 集成

把对应命令加入 Shell 启动文件：

```sh
# Bash（~/.bashrc）
eval "$(jm shell init bash)"

# Zsh（~/.zshrc）
eval "$(jm shell init zsh)"

# Fish（~/.config/fish/config.fish）
jm shell init fish | source
```

```powershell
# PowerShell（$PROFILE）
jm shell init powershell | Invoke-Expression
```

Hook 会为当前 Shell 更新 `JAVA_HOME`，并把所选 JDK 的 `bin` 目录放入
`PATH`。目标 JDK 必须已经安装；项目检测不会在后台静默下载 JDK。

完整的查找顺序、配置示例和缺失版本行为见
[项目级 JDK 切换](docs/project-switching.md)。

## 项目配置

推荐使用 `.java-version`：

```text
temurin-21.0.10+7
```

也可以只写主版本号，例如 `21`，但完整安装 ID 的复现性更好。执行
`jm use <version>` 会自动写入解析后的完整 ID。

如果项目已经使用 SDKMAN，`jm` 可以读取其中的 Java 条目：

```properties
java=21.0.2-tem
kotlin=2.1.0
```

`jm` 只解释 `java=`。它不会安装 `.sdkmanrc` 中的其他候选项，不会执行
SDKMAN Hook，也不代表完整兼容 SDKMAN。迁移细节见
[从 SDKMAN 迁移 Java 项目](docs/sdkman-migration.md)。

## 平台支持

Release 工作流配置了以下文件：

| 操作系统 | 架构 | Release 文件 |
| --- | --- | --- |
| Linux | x86-64 | `jm-linux-x86_64.tar.gz` |
| Linux | ARM64 | `jm-linux-aarch64.tar.gz` |
| macOS | Intel 和 Apple 芯片 | 分架构文件及 Universal 文件 |
| Windows | x86-64 | `jm-windows-x86_64.zip` |

CLI 能识别 x86-64 或 ARM64 上的 Linux、macOS 和 Windows。Windows ARM64
目前没有预编译 Release 文件，从源码构建属于尽力支持范围。

某个 JDK 是否可安装，仍取决于发行版、Java 版本、操作系统、架构以及上游目录
当前提供的文件。

## 常用发行版名称

| 发行版 | 输入示例 |
| --- | --- |
| Eclipse Temurin | `temurin-21` |
| Amazon Corretto | `corretto-17` |
| Azul Zulu | `zulu-21` |
| BellSoft Liberica | `liberica-21` |
| Microsoft OpenJDK | `microsoft-21` |
| GraalVM Community Edition | `graalvm-ce-21` |

可以用 `jm search <发行版>` 或 `jm list --remote --major <主版本>` 查看上游
当前为本机平台提供的内容。

## 命令概览

| 命令 | 用途 |
| --- | --- |
| `jm install <version>` | 下载并安装匹配的 JDK |
| `jm uninstall <version>` | 删除已安装的 JDK |
| `jm list` | 列出已安装的 JDK |
| `jm list --remote` | 列出仍在维护的远程 Java 版本 |
| `jm search <query>` | 搜索远程 JDK 包 |
| `jm use <version>` | 根据已安装项写入项目 `.java-version` |
| `jm default [version]` | 设置或显示全局默认 JDK |
| `jm current` | 显示项目要求或全局默认值及其来源 |
| `jm which [binary]` | 输出全局默认 JDK 中某个命令的路径 |
| `jm env` | 输出 `JAVA_HOME` 和 `PATH` 值 |
| `jm shell init <shell>` | 生成 Shell 集成脚本 |
| `jm shell completions <shell>` | 生成命令补全脚本 |
| `jm config list\|get\|set\|path` | 查看或修改配置 |
| `jm doctor` | 运行本地配置与网络连接诊断 |
| `jm upgrade` | 从最新 GitHub Release 升级 `jm` |

使用 `jm <command> --help` 查看完整参数。

## 配置与排错

执行 `jm config path` 查看 `config.toml` 的位置，执行 `jm config list` 查看
当前配置。常见设置包括默认发行版、代理、Adoptium 回退和压缩包保留策略。

```sh
jm config set global.preferred_distribution zulu
jm config set api.proxy http://proxy.example:8080
jm doctor
```

需要注意：

- `jm install --no-verify <version>` 会明确绕过 JDK 压缩包校验；只有在评估过
  下载来源之后才应使用。
- 项目要求的版本尚未安装时，`jm current` 会给出提示，Shell Hook 暂时继续使用
  全局默认 JDK。
- `packaging/` 中的 Homebrew 和 Scoop 文件只是发布模板；本仓库尚未宣传任何
  官方 Tap 或 Bucket。
- `jm upgrade` 依赖 GitHub Release 中与当前操作系统和架构匹配的文件。

## 文档

- [Windows 与 PowerShell](docs/windows.md)
- [项目级 JDK 切换](docs/project-switching.md)
- [SDKMAN Java 迁移](docs/sdkman-migration.md)
- [贡献指南](CONTRIBUTING.md)
- [安全策略](SECURITY.md)
- [更新日志](CHANGELOG.md)

## 参与贡献

欢迎提交 Bug、文档修正和范围清晰的 Pull Request。修改前请先阅读
[CONTRIBUTING.md](CONTRIBUTING.md)；使用问题见 [SUPPORT.md](SUPPORT.md)。

## 许可证

你可以选择 [Apache License 2.0](LICENSE-APACHE) 或 [MIT License](LICENSE-MIT)
使用本项目。
