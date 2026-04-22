<p align="center">
  <img src="public/boom.svg" alt="BOOM Logo" width="120" height="120">
</p>

<h1 align="center">BOOM</h1>

<p align="center">
  <strong>Academic Administration System</strong>
</p>

<p align="center">
  A modern desktop application for school exam management and invigilation scheduling optimization
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#tech-stack">Tech Stack</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#project-structure">Project Structure</a> •
  <a href="#development-guide">Development Guide</a> •
  <a href="#license">License</a>
</p>

---

## 📖 Description

**BOOM** is a modern desktop application built with **Tauri + Vue 3**, designed for school exam management and invigilation scheduling optimization. The system uses the **OR-Tools CP-SAT constraint solver** to implement intelligent invigilation scheduling algorithms that automatically generate fair and reasonable monitoring plans.

**中文文档**: [README_CN.md](README_CN.md)

---

## ✨ Features

### 🎯 Core Features

- **📊 Exam Dashboard** - Visual overview of exam schedules and statistics
- **📋 Invigilation Management** - Comprehensive invigilation task assignment and management
- **🎲 Monitor Draw** - Constraint-based intelligent invigilation lottery system
- **👨‍🏫 Teacher Management** - Teacher information maintenance and queries
- **📝 Score Management** - Student score entry, statistics, and analysis
- **🏫 Class Configuration** - Class information management and configuration
- **⚙️ Settings** - Application parameters and preferences

### 🔧 Technical Highlights

- **Smart Algorithm**: Uses Google OR-Tools CP-SAT for constraint satisfaction problem solving
- **Cross-platform**: Built with Tauri, supports Windows/macOS/Linux
- **High Performance**: Rust backend ensures excellent runtime performance
- **Modern UI**: Fluent Design style user interface
- **Data Import/Export**: Support for Excel file import and export
- **Local Storage**: Uses SQLite database for secure and reliable data storage

---

## 🛠️ Tech Stack

### Frontend

| Technology | Version | Description |
|------------|---------|-------------|
| [Vue](https://vuejs.org/) | ^3.5.13 | Progressive JavaScript Framework |
| [TypeScript](https://www.typescriptlang.org/) | ~5.6.2 | Typed superset of JavaScript |
| [Vite](https://vitejs.dev/) | ^6.0.3 | Next Generation Frontend Tooling |
| [Vue Router](https://router.vuejs.org/) | ^5.0.4 | Official router for Vue.js |
| [Tauri API](https://tauri.app/) | ^2 | Desktop Application Framework API |

### Backend

| Technology | Version | Description |
|------------|---------|-------------|
| [Rust](https://www.rust-lang.org/) | Edition 2021 | Systems programming language |
| [Tauri](https://tauri.app/) | ^2 | Cross-platform Desktop Application Framework |
| [SQLite](https://www.sqlite.org/) | via rusqlite | Lightweight Relational Database |
| [OR-Tools](https://developers.google.com/optimization) | v9.12 | Google Optimization Tools (CP-SAT) |
| [Serde](https://serde.rs/) | ^1 | Serialization/Deserialization Framework |

### Development Tools

- **Package Manager**: pnpm
- **Testing**: Vitest + Vue Test Utils
- **Linting**: vue-tsc (TypeScript type checking)

---

## 🚀 Quick Start

### Prerequisites

- **Node.js** >= 18.x
- **pnpm** >= 8.x
- **Rust** toolchain (stable channel)
- **Windows**: Visual Studio 2022 with C++ development workload (for OR-Tools)

### Installation

```bash
# Clone the repository
git clone https://github.com/your-username/Academic-Administration-System.git
cd Academic-Administration-System

# Install dependencies
pnpm install
```

### Development Mode

```bash
# Start development server with Tauri
pnpm tauri dev
```

The application will start in dev mode with hot reload support.

### Production Build

```bash
# Build for production
pnpm tauri build
```

Build artifacts will be in the `src-tauri/target/release/bundle/` directory.

---

## 📁 Project Structure

```
Academic-Administration-System/
├── public/                     # Static assets
│   ├── fonts/                  # Font files
│   └── boom.svg               # App icon
├── src/                        # Frontend source
│   ├── app/                    # App core
│   │   ├── router/             # Routing configuration
│   │   └── App.vue            # Root component
│   ├── entities/               # Data models
│   │   ├── class-config/      # Class config model
│   │   ├── exam-plan/         # Exam plan model
│   │   ├── score/             # Score model
│   │   └── teacher/           # Teacher model
│   ├── features/              # Feature modules
│   │   ├── dashboard/         # Exam dashboard
│   │   ├── invigilation/      # Invigilation management
│   │   ├── monitor-draw/      # Monitor draw
│   │   ├── teachers/          # Teacher management
│   │   ├── scores/            # Score management
│   │   ├── classes/           # Class configuration
│   │   └── settings/          # System settings
│   ├── pages/                 # Page components
│   ├── shared/                # Shared modules
│   │   ├── styles/            # Global styles
│   │   ├── theme/             # Theme configuration
│   │   ├── types/             # Type definitions
│   │   └── utils/             # Utility functions
│   ├── widgets/               # Common widgets
│   │   ├── common/            # Basic components
│   │   └── layout/            # Layout components
│   └── main.ts               # Entry point
├── src-tauri/                 # Tauri/Rust backend
│   ├── src/                   # Rust source code
│   │   ├── main.rs           # Application entry
│   │   ├── lib.rs            # API definitions
│   │   ├── schema.rs         # Database schema
│   │   ├── invigilation.rs   # Invigilation logic
│   │   ├── exam_allocation.rs # Exam allocation
│   │   ├── teacher.rs        # Teacher management
│   │   ├── score.rs          # Score management
│   │   └── ...               # Other modules
│   ├── vendor/               # Third-party dependencies
│   │   └── or-tools/         # OR-Tools binaries
│   └── third_party/cp_sat/   # CP-SAT bindings
├── sample/                    # Sample data
├── package.json              # Node.js config
├── Cargo.toml               # Rust config
└── tauri.conf.json          # Tauri config
```

---

## 💻 Development Guide

### Available Scripts

```bash
# Development mode with Tauri
pnpm tauri dev

# Frontend dev server only
pnpm dev

# Type checking
pnpm typecheck

# Build frontend
pnpm build

# Preview build
pnpm preview

# Run tests
pnpm test

# Run tests in watch mode
pnpm test:watch
```

### Core Architecture

#### Frontend Architecture

Adopts a **Feature-Based** modular architecture:

- **Entities**: Define data models and types
- **Features**: Independent modules organized by business function (UI + Store + Service)
- **Widgets**: Reusable common components
- **Shared**: Utilities, styles, themes, and other shared resources

#### Backend Architecture

Uses **Modular Rust** design:

- **Schema**: Database table structure definitions (SQLite)
- **Services**: Business logic implementation (teachers, exams, scores, etc.)
- **API Layer**: Interfaces exposed to frontend via Tauri Commands
- **Solver**: OR-Tools CP-SAT constraint solver integration

### Data Flow

```
[Vue Components] → [Tauri Invoke] → [Rust Commands] → [SQLite Database]
       ↑                                                              ↓
       └──────────────── [Response] ←─────────────────────────────────┘
```

---

## 📊 Screenshots

*(Coming soon)*

---

## 🤝 Contributing

Issues and Pull Requests are welcome!

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Tauri Team](https://tauri.app/) - Excellent cross-platform desktop framework
- [Vue.js Team](https://vuejs.org/) - Progressive JavaScript framework
- [Google OR-Tools](https://developers.google.com/optimization) - Powerful optimization toolkit
- [Rust Community](https://www.rust-lang.org/) - Safe and efficient systems programming language

---

<p align="center">
  Made with ❤️ by BOOM Team
</p>
