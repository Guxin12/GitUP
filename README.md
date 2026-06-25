# GitUp – 交互式 Git 工作流助手

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**GitUp** 是一款用 Rust 编写的命令行工具，通过交互式菜单简化日常 Git 操作。无论你是内核开发者、开源贡献者，还是团队协作成员，GitUp 都能帮助你高效管理多仓库、多分支、多远程的工作流。

📌 **告别冗长命令，轻松掌控版本控制。**

---

## ✨ 特性一览

- 🔧 **一次性配置** – 将仓库路径、用户信息、远程仓库、默认分支等保存至本地 `.ini` 文件，迁移机器只需复制文件。
- 📦 **智能克隆** – 支持 GitHub 或任意 Git 仓库，可选择完整克隆或浅克隆，克隆后自动更新配置并切换目录。
- ✍️ **提交与推送** – 一键暂存所有变更，编写标题和多行正文（`\\` 换行），推送时可选择普通推送或 `--force-with-lease`。
- 🚀 **直接上传** – 无需重新输入提交信息，直接推送已有本地提交，适合频繁推送场景。
- 🔄 **从远程更新** – 拉取更新前可查看新提交列表，支持 stash/discard 未提交变更，合并或变基任选，更新后还可选择推送。
- 🔗 **合并任意远程提交** – 临时添加远程（GitHub 或外部 URL），支持拉取全部或最新 N 个提交，并可通过单个或范围 cherry-pick 合并，冲突处理交互友好。
- 📋 **交互式日志查看** – 分页浏览提交历史（默认每页 20 条，可动态调整），查看任意提交的完整信息（哈希、作者、日期、标题、正文）。
- 🛡️ **安全撤销** – 通过 `git revert` 生成新的回退提交，避免破坏历史，并可选择立即推送。
- ⚠️ **硬重置与回退** – 支持 soft/mixed/hard 三种重置模式，强制推送前双重确认，防止误操作。
- 🌿 **分支自动化** – 启动时自动切换到配置的分支，若不存在则自动创建。

---

## 🚀 安装指南

### 前置条件
- [Rust](https://rustup.rs/) (1.70+)
- Git

### 从源码编译
```bash
git clone https://github.com/Guxin12/GitUP.git
cd gitup
cargo build --release
```

编译后二进制文件位于 target/release/gitup，可将其复制到 $PATH 目录（如 /usr/local/bin）以便全局调用。

---

📖 首次使用

1. 运行 ./gitup，如果配置文件 git保存内容.ini 不存在或不完整，程序将引导你完成初始化：
   · 手动配置：输入仓库目录（相对路径）、Git 用户名/邮箱、远程名、GitHub 仓库（username/repo）、默认分支。
   · 克隆仓库：直接输入 GitHub 仓库地址，可选指定分支和克隆深度，克隆后自动写入配置并切换目录。
2. 配置完成后，自动进入目标仓库并显示主菜单。

---

🧭 主菜单

```
1) 提交并推送
2) 创建 / 切换分支
3) 克隆仓库
4) 从远程仓库合并提交
5) 更新源码
6) 修改配置
7) 撤销指定提交（安全）
8) 删除本地提交（回退）
9) 查看提交日志（交互式）
10) 直接上传（无需提交信息）
11) 退出
```

选择对应数字，跟随提示操作即可。所有确认问题均支持 y/Y/yes/YES 和 n/N/no/NO，回车则使用默认值。

---

⚙️ 配置文件

配置文件 git保存内容.ini 位于可执行文件同目录，示例内容：

```ini
DIR="my_kernel_repo"
GIT_NAME="Your Name"
GIT_EMAIL="you@example.com"
GIT_REMOTE_NAME="origin"
GIT_REMOTE_REPO="username/repository"
GIT_BRANCH="main"
```

DIR：仓库目录（相对于可执行文件）。
GIT_NAME / GIT_EMAIL：本地 Git 用户配置。
GIT_REMOTE_NAME：远程仓库别名。
GIT_REMOTE_REPO：GitHub 仓库 用户名/仓库名。
GIT_BRANCH：默认分支。

可通过菜单选项 6 交互式修改，也可手动编辑。

---

📜 许可证

GitUp 使用 GNU Affero General Public License v3.0 (AGPLv3) 许可。无论是分发还是作为网络服务提供，任何使用或修改本软件的人都必须开源其源码。详见 LICENSE 文件。

---

🤝 贡献指南

欢迎提交 Issue、Pull Request 或功能建议。

---

🌟 致谢

GitUp 使用 ❤️ 和 Rust 构建，旨在让 Git 操作变得更直观、更高效。

---

开始使用 GitUp，享受流畅的版本控制体验！ 🚀
