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

### 注意事项

- 端口固定 `37421`，避免冲突
- 应用关闭后 API 不可用（灯灭）
- 不要过度发送 — 只在关键状态切换时发送，避免无意义刷新
- `message` 字段可选，建议在 `working` 和 `error` 状态下附带简要说明
