<p align="center">
  <img src="public/boom.svg" alt="BOOM Logo" width="120" height="120">
</p>

<h1 align="center">BOOM</h1>

<p align="center">
  <strong>教务管理系统</strong>
</p>

<p align="center">
  一款现代化的桌面应用程序，用于学校考试管理与监考排班优化
</p>

<p align="center">
  <a href="#功能特性">功能特性</a> •
  <a href="#技术栈">技术栈</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#项目结构">项目结构</a> •
  <a href="#开发指南">开发指南</a> •
  <a href="#许可证">许可证</a>
</p>

---

## 📖 简介

**BOOM** 是一款基于 **Tauri + Vue 3** 构建的现代化桌面应用程序，专为学校考试管理与监考排班优化而设计。系统采用 **OR-Tools CP-SAT 约束求解器** 实现智能监考排班算法，可自动生成公平合理的监考方案。

**English Documentation**: [README.md](README.md)

---

## ✨ 功能特性

### 🎯 核心功能

- **📊 考试仪表盘** - 考试安排与统计数据的可视化概览
- **📋 监考管理** - 监考任务分配与全面管理
- **🎲 监考抽签** - 基于约束条件的智能监考抽签系统
- **👨‍🏫 教师管理** - 教师信息维护与查询
- **📝 成绩管理** - 学生成绩录入、统计与分析
- **🏫 班级配置** - 班级信息管理与配置
- **⚙️ 系统设置** - 应用参数与偏好设置

###  技术亮点

- **智能算法**：采用 Google OR-Tools CP-SAT 解决约束满足问题
- **跨平台**：基于 Tauri 构建，支持 Windows/macOS/Linux
- **高性能**：Rust 后端确保卓越的运行性能
- **现代 UI**：流畅设计（Fluent Design）风格用户界面
- **数据导入导出**：支持 Excel 文件导入与导出
- **本地存储**：采用 SQLite 数据库实现安全可靠的数据存储

---

## 🛠️ 技术栈

### 前端

| 技术 | 版本 | 说明 |
|------------|---------|-------------|
| [Vue](https://vuejs.org/) | ^3.5.13 | 渐进式 JavaScript 框架 |
| [TypeScript](https://www.typescriptlang.org/) | ~5.6.2 | JavaScript 的超集，提供类型支持 |
| [Vite](https://vitejs.dev/) | ^6.0.3 | 下一代前端构建工具 |
| [Vue Router](https://router.vuejs.org/) | ^5.0.4 | Vue.js 官方路由 |
| [Tauri API](https://tauri.app/) | ^2 | 桌面应用框架 API |

### 后端

| 技术 | 版本 | 说明 |
|------------|---------|-------------|
| [Rust](https://www.rust-lang.org/) | Edition 2021 | 系统级编程语言 |
| [Tauri](https://tauri.app/) | ^2 | 跨平台桌面应用框架 |
| [SQLite](https://www.sqlite.org/) | via rusqlite | 轻量级关系型数据库 |
| [OR-Tools](https://developers.google.com/optimization) | v9.12 | Google 优化工具（CP-SAT） |
| [Serde](https://serde.rs/) | ^1 | 序列化/反序列化框架 |

### 开发工具

- **包管理器**：pnpm
- **测试**：Vitest + Vue Test Utils
- **类型检查**：vue-tsc

---

## 🚀 快速开始

### 前置要求

- **Node.js** >= 18.x
- **pnpm** >= 8.x
- **Rust** 工具链（stable 版本）
- **Windows**：Visual Studio 2022（含 C++ 开发工作负载，用于 OR-Tools）

### 安装

```bash
# 克隆仓库
git clone https://github.com/your-username/Academic-Administration-System.git
cd Academic-Administration-System

# 安装依赖
pnpm install
```

### 开发模式

```bash
# 启动 Tauri 开发服务器
pnpm tauri dev
```

应用将以开发模式启动，支持热重载。

### 生产构建

```bash
# 构建生产版本
pnpm tauri build
```

构建产物将位于 `src-tauri/target/release/bundle/` 目录。

---

## 📁 项目结构

```
Academic-Administration-System/
├── public/                     # 静态资源
│   ├── fonts/                  # 字体文件
│   └── boom.svg               # 应用图标
├── src/                        # 前端源码
│   ├── app/                    # 应用核心
│   │   ├── router/             # 路由配置
│   │   └── App.vue            # 根组件
│   ├── entities/               # 数据模型
│   │   ├── class-config/      # 班级配置模型
│   │   ├── exam-plan/         # 考试计划模型
│   │   ├── score/             # 成绩模型
│   │   └── teacher/           # 教师模型
│   ├── features/              # 功能模块
│   │   ├── dashboard/         # 考试仪表盘
│   │   ├── invigilation/      # 监考管理
│   │   ├── monitor-draw/      # 监考抽签
│   │   ├── teachers/          # 教师管理
│   │   ├── scores/            # 成绩管理
│   │   ├── classes/           # 班级配置
│   │   └── settings/          # 系统设置
│   ├── pages/                 # 页面组件
│   ├── shared/                # 共享模块
│   │   ├── styles/            # 全局样式
│   │   ├── theme/             # 主题配置
│   │   ├── types/             # 类型定义
│   │   └── utils/             # 工具函数
│   ├── widgets/               # 通用组件
│   │   ├── common/            # 基础组件
│   │   ── layout/            # 布局组件
│   └── main.ts               # 入口文件
── src-tauri/                 # Tauri/Rust 后端
│   ├── src/                   # Rust 源码
│   │   ├── main.rs           # 应用入口
│   │   ├── lib.rs            # API 定义
│   │   ├── schema.rs         # 数据库结构
│   │   ├── invigilation.rs   # 监考逻辑
│   │   ├── exam_allocation.rs # 考试分配
│   │   ├── teacher.rs        # 教师管理
│   │   ├── score.rs          # 成绩管理
│   │   └── ...               # 其他模块
│   ├── vendor/               # 第三方依赖
│   │   ── or-tools/         # OR-Tools 二进制文件
│   └── third_party/cp_sat/   # CP-SAT 绑定
├── sample/                    # 示例数据
├── package.json              # Node.js 配置
├── Cargo.toml               # Rust 配置
└── tauri.conf.json          # Tauri 配置
```

---

## 💻 开发指南

### 可用脚本

```bash
# Tauri 开发模式
pnpm tauri dev

# 仅启动前端开发服务器
pnpm dev

# 类型检查
pnpm typecheck

# 构建前端
pnpm build

# 预览构建结果
pnpm preview

# 运行测试
pnpm test

# 监听模式运行测试
pnpm test:watch
```

### 核心架构

#### 前端架构

采用 **功能模块驱动（Feature-Based）** 的模块化架构：

- **Entities**：定义数据模型与类型
- **Features**：按业务功能组织的独立模块（UI + Store + Service）
- **Widgets**：可复用的通用组件
- **Shared**：工具函数、样式、主题等共享资源

#### 后端架构

采用 **模块化 Rust** 设计：

- **Schema**：数据库表结构定义（SQLite）
- **Services**：业务逻辑实现（教师、考试、成绩等）
- **API Layer**：通过 Tauri Commands 暴露给前端的接口
- **Solver**：OR-Tools CP-SAT 约束求解器集成

### 数据流

```
[Vue 组件] → [Tauri Invoke] → [Rust Commands] → [SQLite 数据库]
       ↑                                                        ↓
       ──────────────── [响应] ←────────────────────────────────┘
```

---

## 📊 界面截图

*(即将上线)*

---

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 提交 Pull Request

---

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

---

## 🙏 致谢

- [Tauri 团队](https://tauri.app/) - 优秀的跨平台桌面框架
- [Vue.js 团队](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Google OR-Tools](https://developers.google.com/optimization) - 强大的优化工具套件
- [Rust 社区](https://www.rust-lang.org/) - 安全高效的系统级编程语言

---

<p align="center">
  Made with ❤️ by BOOM Team
</p>
