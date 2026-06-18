# Light for All Agent — Agent 操作指南

## 项目概述

Light for All Agent 是一个悬浮桌面的三色状态指示灯，AI Agent（我）通过 HTTP API 实时更新灯色，让你不必切回窗口也能知道当前工作状态。

## 架构

```
AI Agent (我) → HTTP POST /status → Light for All Agent (Tauri 桌面悬浮窗)
```

- Agent 端：每步操作前后主动推状态
- 灯端：常驻 Tauri 小窗口，接收 HTTP 更新灯色

## 状态映射

| mode | 灯色 | 含义 | Agent 何时发送 |
|------|------|------|---------------|
| `idle` | 🟢 绿 | 空闲/完成 | 任务结束、等待用户输入 |
| `working` | 🟡 黄 | 工作中 | 执行命令、搜索代码、读写文件 |
| `waiting` | 🟠 橙闪烁 | 等待用户 | 需要确认权限、选择方案、回答 |
| `error` | 🔴 红 | 失败 | 命令报错、异常终止 |

## Agent 调用方式

Agent 在执行关键操作时，向以下端点发送 HTTP POST 请求：

```
POST http://127.0.0.1:37421/status
Content-Type: application/json

{"mode": "working", "message": "执行 npm install"}
```

### 发送时机

| 节点 | 发送的 mode |
|------|------------|
| 收到用户指令后开始执行 | `working` |
| 开始执行 bash/PowerShell 命令 | `working` |
| 开始读写文件 | `working` |
| 需要用户确认/选择 | `waiting` |
| 用户确认后继续 | `working` |
| 命令执行失败 | `error` |
| 任务全部完成 | `idle` |
| 等待用户下达新指令 | `idle` |

### 查询当前状态

```
GET http://127.0.0.1:37421/status
```

返回当前 mode、message、orientation、autoStart 状态。

## 使用与流程

### 1. 启动灯端

- 本地开发：`npm install` + `npm run dev`
- 生产环境：运行 `dist/Light-for-all-Agent-0.1.0-x64-portable.exe`（需 `WebView2Loader.dll` 同目录）
- 启动后，灯端会监听 `http://127.0.0.1:37421`
- 应用常驻系统托盘（右下角），**不在任务栏显示**；托盘左键点击可隐藏/显示悬浮窗
- 若 `GET /status` 无响应，可能是应用未启动或窗口被隐藏（但 API 仍可用）

### 2. Agent 发送状态

- 通过 `POST /status` 更新状态
- 仅支持四个 mode：`idle`、`working`、`waiting`、`error`
- `message` 为可选字段，用于补充当前步骤说明

示例：

```bash
curl -X POST http://127.0.0.1:37421/status \
  -H "Content-Type: application/json" \
  -d '{"mode":"waiting","message":"等待用户选择"}'
```

### 3. 详细流程文档

- 开发、测试、打包、部署、环境搭建：`docs/process.md`
- 设计与架构：`docs/design.md`

## 技术栈与工具链

| 层 | 技术 |
|----|------|
| 前端 | 纯 HTML + CSS + JS（无框架） |
| 后端 | Rust 1.94.0 + Tauri v2（含系统托盘） |
| 工具链 | `x86_64-pc-windows-gnu`（通过 `src-tauri/rust-toolchain.toml` 锁定） |
| 链接器 | MSYS2 MinGW-w64（提供 `gcc`/`dlltool`/`ld`） |
| 通信 | HTTP REST API（`127.0.0.1:37421`） |
| 产物 | 约 18MB 便携版 exe（静态链接，附 WebView2Loader.dll） |

> ⚠️ Rust 版本必须锁在 1.94.0，1.96+ 与依赖存在不兼容问题。Windows 构建走 GNU 工具链，需 MSYS2 提供 MinGW 链接器，**无需安装 Visual Studio**。完整搭建步骤见 `docs/process.md`。

### 注意事项

- 端口固定为 `37421`
- 应用关闭后 API 不可用
- 尽量只在状态变化时发送请求，避免频繁刷新
