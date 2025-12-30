# AI Translate Tool - 开发计划文档

## 项目概述

构建一个基于egui的AI文本翻译工具，使用zai-rs提供的GLM-4.7 API进行多语言文本翻译。

---

## 1. 项目架构设计

### 1.1 技术栈
- **UI框架**: egui
- **异步运行时**: tokio
- **AI模型**: zai-rs GLM-4.7 (coding-plan 模式)
- **输出模式**: 流式输出 (streaming)

### 1.2 目录结构
```
ai-translate/
├── src/
│   ├── main.rs              # 应用程序入口
│   ├── ui/
│   │   ├── mod.rs           # UI模块入口
│   │   ├── app.rs           # 主应用状态和结构
│   │   ├── sidebar.rs       # 右侧侧边栏组件
│   │   ├── display.rs       # 中间显示区域组件
│   │   └── theme.rs         # 主题和字体管理
│   ├── api/
│   │   ├── mod.rs           # API模块入口
│   │   ├── client.rs        # zai-rs API客户端封装
│   │   └── translator.rs    # 翻译逻辑处理
│   ├── async/
│   │   ├── mod.rs           # 异步模块入口
│   │   └── channel.rs       # UI与数据处理间的通信通道
│   └── utils/
│       ├── mod.rs           # 工具模块入口
│       ├── logger.rs        # 日志记录工具
│       └── config.rs        # 配置管理
├── AGENT.md                 # 项目需求文档
├── DEVELOPMENT_PLAN.md      # 本开发计划文档
├── Cargo.toml               # Rust项目配置
└── .gitignore               # Git忽略配置
```

---

## 2. 开发阶段划分

### 阶段1: 项目初始化和基础配置
**目标**: 建立项目基础架构，配置依赖和环境

**任务列表**:
1. 配置 Cargo.toml 依赖项
   - 添加 egui、eframe (egui的框架支持)
   - 添加 tokio (异步运行时)
   - 添加 zai-rs (AI API客户端)
   - 添加 serde、serde_json (数据序列化)
   - 添加 chrono (时间戳)
   - 添加 log/env_logger (日志)

2. 创建目录结构
   - 创建 ui/、api/、async/、utils/ 子目录
   - 为每个子目录创建 mod.rs 和 AGENT.md

3. 配置 .gitignore
   - 添加 Rust 常见忽略项
   - 添加日志文件忽略

---

### 阶段2: 工具模块开发
**目标**: 实现日志、配置管理等基础工具

**任务列表**:
1. 实现日志工具 (utils/logger.rs)
   - 文本格式日志记录
   - 时间戳和翻译上下文
   - 线程安全的日志写入

2. 实现配置管理 (utils/config.rs)
   - API Key 持久化存储
   - 目标语言配置
   - 主题和字体配置

3. 实现异步通信 (async/channel.rs)
   - Tokio channels 用于 UI 和数据处理通信
   - 流式数据传输支持

---

### 阶段3: API集成开发
**目标**: 集成 zai-rs API，实现翻译逻辑

**任务列表**:
1. 实现 API 客户端 (api/client.rs)
   - zai-rs 客户端初始化
   - API Key 认证
   - 流式输出处理

2. 实现翻译逻辑 (api/translator.rs)
   - 构建翻译 prompt
   - 调用 GLM-4.7 coding-plan 模式
   - 处理流式响应
   - 错误处理和重试机制

---

### 阶段4: UI核心组件开发
**目标**: 实现主界面布局和核心UI组件

**任务列表**:
1. 实现应用状态 (ui/app.rs)
   - 应用主状态结构
   - UI组件间的数据共享
   - 异步任务管理

2. 实现侧边栏 (ui/sidebar.rs)
   - API Key 输入框（egui内置保存）
   - 目标语言选择（多语言下拉框）
   - 源文本输入框

3. 实现显示区域 (ui/display.rs)
   - 输入文本显示
   - 翻译结果实时显示（流式）
   - 结构化布局

4. 实现主题管理 (ui/theme.rs)
   - 黑白主题切换
   - 字体大小调整

---

### 阶段5: UI集成和功能完善
**目标**: 集成所有组件，实现完整功能

**任务列表**:
1. 集成 UI 模块 (ui/mod.rs)
   - 导出所有UI组件
   - 统一接口

2. 实现主程序入口 (main.rs)
   - 初始化egui应用
   - 设置窗口参数
   - 连接所有组件

3. 实现流式输出集成
   - UI实时更新翻译结果
   - 异步任务管理
   - 防止UI阻塞

---

### 阶段6: 测试和质量保证
**目标**: 确保代码质量和功能正确性

**任务列表**:
1. 单元测试
   - API 测试
   - 工具函数测试

2. 代码格式化
   - 运行 `cargo fmt`

3. 代码检查
   - 运行 `cargo clippy`

4. CI/CD 配置
   - GitHub Actions 工作流
   - 自动化测试和检查

---

## 3. 关键技术点

### 3.1 UI与数据处理分离
- 使用 tokio channels (mpsc/unbounded) 进行通信
- UI线程负责显示
- 后台线程处理API调用和数据处理

### 3.2 流式输出处理
- 使用 egui 的 `CtxRef::request_repaint()` 实现实时更新
- 分块接收AI响应并更新UI
- 避免阻塞主线程

### 3.3 状态管理
- 使用 Arc<Mutex<>> 或 Rc<RefCell<>> 共享状态
- 使用 egui 的内存管理特性

### 3.4 日志记录
- 每次翻译记录:
  - 时间戳
  - 源语言（自动识别）
  - 目标语言
  - 源文本
  - 翻译结果

---

## 4. 依赖清单

```toml
[dependencies]
egui = "0.28"
eframe = "0.28"
tokio = { version = "1", features = ["full"] }
zai-rs = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
log = "0.4"
env_logger = "0.11"
```

---

## 5. 开发优先级

**高优先级**:
1. 项目初始化和依赖配置
2. API集成和翻译逻辑
3. 核心UI组件

**中优先级**:
4. 流式输出实现
5. 日志功能
6. 主题和字体调整

**低优先级**:
7. 代码优化和重构
8. 测试覆盖完善
9. CI/CD配置

---

## 6. 里程碑

- **里程碑1**: 项目基础架构完成，依赖配置完成
- **里程碑2**: API集成完成，可调用翻译功能
- **里程碑3**: UI核心组件完成，基本界面可用
- **里程碑4**: 流式输出实现，实时翻译功能完成
- **里程碑5**: 所有功能完成，测试通过

---

## 7. 风险和注意事项

1. **API限流**: 注意zai-rs API的调用频率限制
2. **UI响应性**: 确保异步处理不阻塞UI
3. **错误处理**: 完善的错误提示和恢复机制
4. **跨平台**: 确保在不同操作系统上字体显示正常

---

## 8. 后续扩展

- 支持批量翻译
- 历史记录查看
- 导出翻译结果
- 自定义翻译模型参数
- 多窗口支持
