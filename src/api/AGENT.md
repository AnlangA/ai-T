# API 模块

## 功能概述

本模块负责与 zai-rs API 进行交互，实现AI翻译功能。

## 目录结构

```
api/
├── mod.rs       # API模块入口
├── client.rs    # zai-rs API客户端封装
└── translator.rs # 翻译逻辑处理
```

## 组件说明

### mod.rs
- 导出所有API组件
- 统一模块接口

### client.rs
- 初始化 zai-rs 客户端
- API Key 认证管理
- 流式输出处理
- 错误处理和重试机制

### translator.rs
- 构建翻译 prompt
- 调用 GLM-4.7 coding-plan 模式
- 处理流式响应数据
- 自动识别源语言

## 技术要点

- 使用 zai-rs 库调用 GLM-4.7 API
- 流式输出处理 (streaming)
- 异步API调用 (tokio)
- 错误恢复和重试逻辑

## API调用流程

1. 接收源文本和目标语言
2. 构建翻译 prompt
3. 调用 zai-rs API
4. 处理流式响应
5. 返回翻译结果

## 依赖

- zai-rs
- tokio
- serde/serde_json
- utils (内部模块 - 日志)
