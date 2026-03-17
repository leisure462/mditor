# Mditor

[![CI](https://github.com/leisure462/mditor/actions/workflows/mditor-ci.yml/badge.svg)](https://github.com/leisure462/mditor/actions/workflows/mditor-ci.yml)
[![Release](https://github.com/leisure462/mditor/actions/workflows/mditor-release.yml/badge.svg)](https://github.com/leisure462/mditor/actions/workflows/mditor-release.yml)

`Mditor` 是一个基于 Zed 改造的桌面 Markdown 编辑器，目标不是继续做“全能编程 IDE”，而是回到更专注的写作、预览和本地知识整理体验。

这个仓库已经将产品品牌、应用目录、协议名和默认启动二进制切换为 `mditor`，并持续移除与 Markdown 写作无关的功能入口和依赖链。

## 项目定位

- 专注 Markdown 编辑、预览和本地文档管理。
- 保留原生编辑器内核带来的性能、分栏和工作区能力。
- 保留系统内置 Agent 能力。
- 移除登录、自动更新、插件市场、外部 Agent 适配、协作、调试器、终端等不再需要的产品路径。
- 与机器上原本安装的 Zed 隔离配置、缓存、日志和用户数据，不互相污染。

## 当前特性

- Markdown 编辑与实时预览。
- 项目侧边栏与多标签页。
- 独立的 `mditor` 应用目录、URL Scheme 与 CLI 名称。
- 内置 `macOS Classic Light` / `macOS Classic Dark` 主题可选。
- 更接近单层标题栏的窗口交互逻辑。
- 默认精简后的菜单、设置项与快捷键集合。

## 与原始 Zed 的主要差异

- 产品名、应用目录和协议从 `Zed` / `zed://` 改为 `Mditor` / `mditor://`。
- 不再走登录、自动更新与账号相关 UI。
- 不再暴露扩展安装、插件市场、外部 Agent 适配入口。
- 不再保留调试器、任务、终端、协作等主要前端入口。
- 清理了原始文档站、扩展桥接源码、相关测试残留与无关发布文档。

## 下载

发布构建会通过 GitHub Actions 生成并上传到 Releases：

- Releases 页面: <https://github.com/leisure462/mditor/releases>
- 首个版本计划标签: `v0.1.0`

当前发布流程默认生成：

- Windows x86_64: `mditor-<version>-windows-x86_64.zip`
- Linux x86_64: `mditor-<version>-linux-x86_64.tar.gz`

## 本地构建

### 运行开发版

```bash
cargo run -p zed --bin mditor
```

### 构建发布版

```bash
cargo build --release -p zed --bin mditor
```

构建完成后的可执行文件位于：

- Windows: `target/release/mditor.exe`
- Linux: `target/release/mditor`

## 本地数据目录

`Mditor` 默认与原始 Zed 使用不同的数据目录。

- Windows:
  - 配置: `%APPDATA%\\Mditor`
  - 数据: `%LOCALAPPDATA%\\Mditor`
- Linux:
  - 配置: `~/.config/Mditor`
  - 数据: `~/.local/share/Mditor`
- macOS:
  - 配置与数据会落到 `Mditor` 相关应用目录，而不是 `Zed`

## GitHub Actions

仓库内置两条适合当前项目状态的工作流：

- `mditor-ci.yml`
  - 在 Windows 和 Linux 上执行 `cargo check -p zed --bin mditor`
- `mditor-release.yml`
  - 当推送 `v*` 标签时，自动构建发布版
  - 自动打包二进制
  - 自动创建 GitHub Release 并上传构建产物

## 开发说明

这个项目来自对 Zed 的深度裁剪，因此仓库内部仍保留一部分底层基础设施 crate。当前方向是继续把产品层收敛为更干净的 Markdown 编辑器，而不是维护完整 IDE 功能。

如果你准备继续做产品化工作，推荐优先处理这些方向：

- 继续清理残留的 Zed 文案与品牌痕迹
- 为 Markdown 写作场景定制更多默认设置
- 打磨首屏、空白页、文件打开体验
- 继续缩减不再使用的 workspace crate 与资源

## 许可证与致谢

本项目基于 Zed 代码库演进而来，并遵循仓库中的原有许可证体系。根目录保留了主要许可证文件：

- `LICENSE-GPL`
- `LICENSE-APACHE`
- `LICENSE-AGPL`

如果你在分发或二次开发时需要做许可证审查，请同时检查各 crate 与资源目录下附带的许可证文件。
