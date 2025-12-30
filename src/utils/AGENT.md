# Utils 模块

## 功能概述

本模块提供通用工具函数，包括日志记录、配置管理等。

## 目录结构

```
utils/
├── mod.rs       # Utils模块入口
├── logger.rs    # 日志记录工具
└── config.rs    # 配置管理
```

## 组件说明

### mod.rs
- 导出所有工具组件
- 统一模块接口

### logger.rs
- 文本格式日志记录
- 时间戳记录 (使用 chrono)
- 完整的翻译上下文记录
- 线程安全的文件写入
- 日志文件管理

### config.rs
- API Key 持久化存储
- 目标语言配置
- 主题配置 (light/dark)
- 字体大小配置
- 配置文件读写

## 日志格式

```
[2025-12-30 12:00:00] Source Language: Auto-detected (English)
Target Language: Chinese
Source Text: Hello, world!
Translation Result: 你好，世界！
```

## 技术要点

- 使用 std::fs 进行文件操作
- 使用 Mutex 实现线程安全
- 使用 serde 进行配置序列化
- 配置文件使用 JSON 或 TOML 格式

## 依赖

- chrono
- serde/serde_json
- std (标准库)
