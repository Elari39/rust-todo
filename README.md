# rust-todo

> 基于 **Tauri 2 + Vue 3 + Rust + SQLite** 的 Windows 桌面待办管理应用（v0.3.1）

轻量、本地优先、带系统托盘与置顶磁贴的 Todo 工具。界面用 Vue 还原现代仪表盘风格，业务逻辑、存储与系统集成全部由 Rust 完成。

---

## ❓ 项目问答

**Q：这是一个什么样的项目？**

A：一个 Windows 桌面端待办（Todo）管理软件。前端是 Vue 3 绘制的自定义仪表盘界面（侧栏 + 今日清单 + 详情抽屉），后端是 Rust：SQLite 持久化、系统托盘、开机自启、到期通知都由 Rust 侧完成，通过 Tauri 的 IPC（`invoke`）与前端通信。

**Q：为什么选 Tauri 2，而不是 Electron 或纯 Rust GUI（egui/iced）？**

A：这个应用的界面是圆角卡片、侧栏、日历、甘特图这类 Web 风格的自定义 UI——用 Web 技术（Vue + CSS）还原成本最低，日历/甘特有现成生态；纯 Rust GUI（egui/iced）很难做出这种视觉效果。相比 Electron，Tauri 复用系统 WebView2，安装包和内存占用小得多，同时业务层可以完全用 Rust 写。

## ❓ 功能问答

**Q：目前支持哪些功能？**

A：

- 今日 / 全部任务 / 日历 / 甘特图四个视图
- 快速任务与详细任务，右侧详情面板完成、编辑、删除
- 项目分组：新建/编辑/删除项目（颜色 + 进度条），删除项目不会删除任务
- 置顶磁贴：独立的 always-on-top 小窗，随手勾任务、快速输入
- 任务排序：「手动 / 按截止时间」切换，手动模式下拖拽卡片排序
- 搜索：标题 + 备注全文匹配，`Ctrl+K` 聚焦
- 系统托盘：左键唤回主窗口，右键菜单（显示主窗口 / 开关磁贴 / 退出）
- 关闭最小化到托盘（可改为直接退出；系统关机时自动放行）
- 开机自启（注册表 Run 键）
- 导出备份：任务 + 项目 + 设置打包 JSON 到「文档 / TodoBackup」
- 到期系统通知，提前量 0–720 分钟可配置

**Q：今日页和全部任务页有什么区别？**

A：今日页只显示「今天开始/截止」以及已逾期的未完成任务，顶部可以直接快速输入；全部任务页显示所有任务，带搜索、状态筛选和排序切换。

**Q：项目分组怎么用？删除项目会删任务吗？**

A：在「项目跟踪」页新建项目（名称 + 预设色板），任务详情里可以把任务归入项目。删除项目时，项目下的任务只会变成「未分组」，不会被删除。

**Q：任务怎么排序？**

A：「全部任务」页可以在「按截止时间」（默认）和「手动排序」之间切换。手动模式下直接拖动卡片调整顺序，松手即保存（搜索时暂停拖拽，避免乱序）。

**Q：搜索支持什么？**

A：匹配标题和备注，显示匹配条数；`Ctrl+K` 快速聚焦搜索框。

**Q：置顶磁贴是什么？怎么开关？**

A：一个 320×480 的无边框小窗，始终置顶、不占任务栏，可拖动、可缩放。在「设置 → 置顶磁贴」开关；关闭主窗口后磁贴仍可独立使用。

**Q：到期提醒怎么工作？**

A：截止前 N 分钟（设置页可配，默认 15）发一次 Windows 系统通知；已通知的任务不会重复提醒，错过提醒的任务在 24 小时内会补发一次。

**Q：关闭窗口后程序去哪了？**

A：默认最小化到系统托盘（托盘左键唤回，右键菜单退出）。设置里可以改成「直接退出」。系统正在关机时会自动放行关闭请求，不会阻塞 Windows 关机。

## ❓ 开发与构建问答

**Q：怎么在本地跑起来？**

A：需要 Node 18+、Rust 1.77+ 和 WebView2 Runtime（Win11 一般自带）。

```bash
npm install
npm run tauri dev
```

**Q：怎么打包成 Windows 安装包？**

A：

```bash
npm run tauri build
```

产物为 NSIS 安装包（`src-tauri/target/release/bundle/nsis/`）。

**Q：项目结构是什么样的？**

A：

```
├── src/                    # Vue 3 前端
│   ├── components/         # 视图与组件（今日/全部/日历/甘特/项目/设置/磁贴）
│   ├── composables/        # useTasks / useProjects / useSettings / useReminders
│   ├── utils/datetime.ts   # 时间格式化与到期判定
│   └── api.ts              # invoke 封装
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── db.rs           # SQLite（tasks / projects 表与迁移、示例数据）
│       ├── commands.rs     # #[tauri::command] IPC 层
│       ├── settings.rs     # settings.json 持久化
│       ├── platform.rs     # Windows 关机检测（SM_SHUTTINGDOWN）
│       └── lib.rs          # 托盘、窗口事件、插件注册
└── src-tauri/capabilities/ # Tauri 2 权限声明（main + tile 窗口）
```

**Q：数据存在哪里？如何备份？**

A：数据库与设置在 `%APPDATA%\com.elaina.todo`（`todo.db` + `settings.json`），设置页内可查看完整路径。备份文件写入「文档 / TodoBackup」，删除数据目录即可完全重置。

**Q：升级版本会丢数据吗？**

A：不会。数据库结构自动迁移（v0.2 补 `projects` 表、v0.3 补 `sort_order` 列），`settings.json` 字段缺失时使用默认值。

**Q：窗口是怎么区分主窗口和磁贴的？**

A：两个窗口加载同一份前端，按 Tauri 窗口 label（`main` / `tile`）在 `App.vue` 里切换渲染。磁贴窗口由 async 命令按需创建——注意在 Windows 上必须在 async command 里创建 webview，同步 command 里创建会得到白屏窗口。
