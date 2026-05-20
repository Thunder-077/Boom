---
alwaysApply: true
scene: git_message
---

# Git 提交记录规范 (Git Commit Message Specification)

所有提交信息必须遵循 Conventional Commits（约定式提交）规范，以确保自动 Release 流程能够正确解析 Git 历史并自动生成发布日志（Release Notes）。

## 1. 提交信息基本格式

提交信息的 Header（首行）必须符合以下正则表达式：
```regex
^\w+(\(.+\))?!?:
```

具体格式如下：
```text
<type>(<scope>)?: <description>
# 或者包含重大变更（Breaking Change）时：
<type>(<scope>)?!: <description>
```

### 格式要求：
- **一句话描述**：`<description>` 用一句话描述提交的内容，建议使用中文，不超过 30 个字符。
- **无标点**：末尾不需要加句号等标点符号。
- **小写类型**：`<type>` 必须全小写。

---

## 2. 类别（Type）与 Release Logs 映射

GitHub Release 脚本（[.github/workflows/release.yml](file:///d:/self/Boom/.github/workflows/release.yml)）会过滤并归类提交记录，对应的分类映射如下：

| 提交类别 (`<type>`) | 说明 | 对应 Release Notes 板块 |
| :--- | :--- | :--- |
| **`feat`** | 引入新功能、新组件等 | **`### ✨ 新增`** |
| **`fix`** | 修复 Bug、异常等 | **`### 🐛 修复`** |
| **`refactor`** / **`perf`** | 代码重构（不影响功能的调整）或性能优化 | **`### ⚡ 优化`** |
| **其他符合规范的类别**<br>(如 `chore`, `docs`, `style`, `test`, `ci`, `build` 等) | 包含项目脚手架变更、文档撰写、代码格式微调、测试用例补全或 CI 流程配置等 | **`### 🔧 其他`** |

> [!WARNING]
> 不满足 `^\w+(\(.+\))?!?:` 正则规范（例如首行不带冒号前缀，或前缀不规范）的提交，以及 Merge 提交，**将不会**被 Release 脚本提取至 Release Notes 中。

---

## 3. 重大变更（Breaking Changes）

如果提交引入了破坏性变更（不向下兼容的修改），请在 `type` 或 `scope` 括号后追加 **`!`**（英文感叹号），并在 description 中简要说明破坏点。例如：
- `feat(api)!: 重构用户权限接口`
- `fix!: 移除旧的废弃配置字段`

---

## 4. 提交示例

- **新增功能**：
  `feat(settings): 检查更新支持全新三态UI渲染`
- **修复 Bug**：
  `fix(updater): 修复 useAppUpdater 未暴露 errorMessage 导致的编译问题`
- **重构/优化**：
  `refactor(invigilation): 优化监考卡片渲染层级与内存占用`
  `perf(classes): 提升课表排程算法运行效率`
- **其他修改**：
  `docs(rules): 补充 Git Commit 提交信息规范文档`
  `test(settings): 为 UpdatePanel.vue 组件补全状态渲染测试`
