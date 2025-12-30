# Async 模块

## 功能概述

本模块负责处理异步任务和线程间通信，确保UI流畅性。

## 目录结构

```
async/
├── mod.rs       # Async模块入口
└── channel.rs   # UI与数据处理间的通信通道
```

## 组件说明

### mod.rs
- 导出所有async组件
- 统一模块接口

### channel.rs
- 定义消息类型 (Message enum)
- 实现 tokio channels (mpsc/unbounded)
- UI线程和后台线程间的通信
- 流式数据传输

## 技术要点

- 使用 tokio::sync::mpsc 进行消息传递
- 使用 tokio::sync::broadcast 实现多订阅
- 支持流式数据块传输
- 线程安全设计

## 消息类型

```rust
enum Message {
    TranslationRequest { text: String, target_lang: String },
    TranslationChunk { chunk: String },
    TranslationComplete,
    Error(String),
}
```

## 使用场景

1. UI发送翻译请求 → 后台处理
2. 后台返回翻译数据块 → UI更新显示
3. 错误信息传递 → UI显示错误提示

## 依赖

- tokio
- ui (内部模块)
- api (内部模块)
