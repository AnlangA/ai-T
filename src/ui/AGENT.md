# UI 模块

## 功能概述

本模块负责构建基于 egui 的用户界面，提供现代化的翻译工具交互界面。

## 目录结构

```
ui/
├── mod.rs       # UI模块入口，导出所有UI组件
├── app.rs       # 主应用状态管理
├── sidebar.rs   # 右侧侧边栏组件
├── display.rs   # 中间显示区域组件
└── theme.rs     # 主题和字体管理
```

## 组件说明

### mod.rs
- 导出所有UI组件
- 统一模块接口

### app.rs
- 定义主应用状态结构 (App)
- 管理UI组件间的数据共享
- 管理异步任务
- 实现egui的 `App` trait

### sidebar.rs
- **顶部**: API Key 输入框（使用egui内置保存）
- **中上部**: 目标语言选择（下拉选择器）
- **下部**: 源文本输入框（多行文本输入）
- 翻译按钮

### display.rs
- 结构化显示输入文本和翻译结果
- 流式显示翻译结果
- 支持实时更新

### theme.rs
- 黑白主题切换
- 字体大小调整
- 颜色方案管理

## 技术要点

- 使用 egui 的 `Context` 进行UI渲染
- 通过 tokio channels 与后端异步模块通信
- 使用 `Arc<Mutex<>>` 共享状态
- 流式更新使用 `request_repaint()`

## 依赖

- egui
- eframe
- tokio
- async (内部模块)
