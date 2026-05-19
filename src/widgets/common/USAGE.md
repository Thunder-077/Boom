# 组件使用指南

## 目录结构

```
src/widgets/
├── common/
│   ├── base/              # 基础组件（无业务逻辑）
│   │   ├── Button.vue     # 按钮
│   │   ├── Input.vue      # 输入框
│   │   └── Tag.vue        # 标签/芯片
│   ├── composite/         # 复合组件（组合基础组件）
│   │   ├── SearchInput.vue # 搜索输入框
│   │   ├── Pagination.vue  # 分页器
│   │   └── EmptyState.vue  # 空状态
│   └── *.vue              # 其他通用组件
│       ├── TableCard.vue
│       ├── ConfigCard.vue
│       ├── FluentSelect.vue
│       ├── FilterToolbar.vue
│       └── InfoHint.vue
```

## 统一导入

```typescript
// 推荐：从统一入口导入
import { Button, Input, Tag, SearchInput, Pagination, EmptyState } from "@/widgets/common";

// 基础组件
import Button from "@/widgets/common/base/Button.vue";
import Input from "@/widgets/common/base/Input.vue";
import Tag from "@/widgets/common/base/Tag.vue";

// 复合组件
import SearchInput from "@/widgets/common/composite/SearchInput.vue";
import Pagination from "@/widgets/common/composite/Pagination.vue";
import EmptyState from "@/widgets/common/composite/EmptyState.vue";
```

## Button 用法

```vue
<Button variant="primary">保存</Button>
<Button variant="secondary">取消</Button>
<Button variant="danger">删除</Button>
<Button variant="ghost">更多</Button>

<Button size="sm">小按钮</Button>
<Button size="md">中按钮</Button>
<Button size="lg">大按钮</Button>

<Button :loading="true">加载中...</Button>
<Button :disabled="true">禁用</Button>

<Button @click="handleClick">点击</Button>
```

## Input 用法

```vue
<Input v-model="value" label="用户名" placeholder="请输入用户名" />
<Input v-model="value" type="password" />
<Input v-model="value" size="sm | md | lg" />
<Input v-model="value" error="错误信息" />
<Input v-model="value" help-text="提示信息" />

<Input v-model="value">
  <template #prefix>
    <span class="material-symbols-rounded">search</span>
  </template>
  <template #suffix>
    <span class="material-symbols-rounded">close</span>
  </template>
</Input>
```

## Tag 用法

```vue
<Tag>默认标签</Tag>
<Tag variant="primary">主要</Tag>
<Tag variant="success">成功</Tag>
<Tag variant="warning">警告</Tag>
<Tag variant="danger">危险</Tag>
<Tag variant="info">信息</Tag>

<Tag size="sm">小</Tag>
<Tag size="md">中</Tag>
<Tag size="lg">大</Tag>

<Tag clickable :active="isActive" @click="toggle">可点击</Tag>
```

## SearchInput 用法

```vue
<SearchInput v-model="keyword" placeholder="搜索..." @search="handleSearch" />
<SearchInput v-model="keyword" :debounce-ms="500" />
<SearchInput v-model="keyword" disabled />
```

## Pagination 用法

```vue
<Pagination
  v-model:currentPage="currentPage"
  :pageSize="pageSize"
  :total="total"
  @change="handlePageChange"
/>
```

## EmptyState 用法

```vue
<EmptyState
  title="暂无数据"
  description="当前没有符合条件的记录"
  icon="inventory_2"
>
  <Button variant="primary" @click="createNew">新建</Button>
</EmptyState>
```

## 全局对话框

```typescript
import { useAppDialog } from "@/shared/ui/appDialog";

const dialog = useAppDialog();

// Alert
await dialog.alert({
  title: "提示",
  summary: "操作已完成",
  details: ["详细信息..."],
});

// Confirm
const confirmed = await dialog.confirm({
  title: "确认删除",
  summary: "删除后无法恢复",
  details: ["当前记录：xxx"],
  confirmText: "确认删除",
  cancelText: "取消",
  tone: "danger", // default | danger | success | warning
});
```

## Design Tokens

### 间距
- `var(--space-xs)`: 4px
- `var(--space-sm)`: 8px
- `var(--space-md)`: 12px
- `var(--space-lg)`: 16px
- `var(--space-xl)`: 24px
- `var(--space-2xl)`: 32px

### 圆角
- `var(--radius-xs)`: 10px
- `var(--radius-sm)`: 14px
- `var(--radius-md)`: 20px
- `var(--radius-lg)`: 28px
- `var(--radius-pill)`: 999px

### 字体大小
- `var(--font-size-xs)`: 12px
- `var(--font-size-sm)`: 13px
- `var(--font-size-base)`: 14px
- `var(--font-size-lg)`: 15px
- `var(--font-size-xl)`: 18px
- `var(--font-size-2xl)`: 20px
- `var(--font-size-3xl)`: 22px

### 过渡时长
- `var(--transition-fast)`: 0.15s
- `var(--transition-base)`: 0.2s
- `var(--transition-slow)`: 0.3s

## 开发规范

1. **优先使用基础组件**：不要重复实现 Button、Input、Tag 等基础组件
2. **使用 Design Tokens**：间距、圆角、字体大小、过渡时长统一使用 tokens
3. **API 一致性**：
   - 使用 `v-model` 进行双向绑定
   - 事件命名遵循 `@click`、`@change`、`@update:modelValue`
   - Props 使用 camelCase
4. **类型安全**：所有组件必须定义 Props 和 Emits 类型
5. **复用优先**：新功能先检查是否有现成组件可用
