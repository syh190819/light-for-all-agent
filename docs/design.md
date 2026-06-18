# Light for All Agent — 设计文档

## 概述

Light for All Agent 是一个通用 AI Agent 桌面状态指示灯。Agent 通过 HTTP API 主动推送状态，桌面小窗口以三色灯形式实时展示，让用户不切换窗口即可知悉 Agent 工作状态。

## 架构

```
用户 ←→ WorkBuddy ←[指令/结果]→ AI Agent (我)
                                    │
                         HTTP POST  │  :37421/status
                                    ▼
                     Light for All Agent
                     (Tauri 桌面悬浮窗)
```

- **Agent**：在每步关键操作前后主动调用 HTTP API
- **灯端**：Tauri 原生窗口，内置 HTTP 服务器接收状态更新
- **通信**：纯旁路 HTTP，不依赖 WorkBuddy 的任何 Hooks 或插件系统

## 状态定义

| mode | 灯色 | 视觉表现 | 描述 |
|------|------|---------|------|
| `idle` | 🟢 绿 | 常亮 + 绿色发光 | 任务完成，等待用户输入 |
| `working` | 🟡 黄 | 常亮 + 黄色发光 | 正在执行命令/读写文件/搜索 |
| `waiting` | 🟠 橙 | 1s 呼吸闪烁 | 需要用户确认权限/选择方案 |
| `error` | 🔴 红 | 常亮 + 红色发光 | 命令失败或任务异常终止 |

## HTTP API 设计

```
POST /status
Content-Type: application/json

请求: {"mode": "working", "message": "内容说明"}
响应: {"ok": true, "mode": "working"}

GET /status
响应: {"ok": true, "mode": "idle", "message": "...", "orientation": "horizontal", "autoStart": true}
```

端口：`37421`（固定高位端口）

## 窗口规格

| 项 | 横向 | 竖向 |
|----|------|------|
| 尺寸 | 自适应（~192×72px） | 自适应（~48×200px） |
| 三灯布局 | 水平排列 | 垂直排列 |
| 边框 | 无边框，圆角 12px | 同左 |
| 背景 | 深色半透明玻璃态 | 同左 |

## 功能清单

- [x] 三色灯状态显示（idle/working/waiting/error）
- [x] HTTP API 接收状态更新
- [x] 窗口置顶
- [x] 拖动 + 屏幕边缘自动吸附
- [x] 横向/竖向切换（右键菜单）
- [x] 开机自启（右键菜单 → 注册表 HKCU Run）
- [x] 便携版 exe，无需安装
- [ ] Agent Skill 自动集成到 WorkBuddy

## 状态机

```
                   ┌──────────┐
       ┌──────────│   idle   │◄────────────┐
       │          │   🟢绿   │             │
       │          └────┬─────┘             │
       │               │ 收到指令          │
       │               ▼                   │
       │          ┌──────────┐             │
       ├─────────►│ working  │─────────────┤
       │          │   🟡黄   │  任务完成    │
       │          └────┬─────┘             │
       │               │ 需要用户确认      │
       │               ▼                   │
       │          ┌──────────┐             │
       │          │ waiting  │─────────────┤
       │          │   🟠橙闪 │  用户确认    │
       │          └──────────┘             │
       │                                   │
       │          ┌──────────┐             │
       └─────────►│  error   │─────────────┘
                  │   🔴红   │  手动/自动恢复
                  └──────────┘
```

## 实现参考

基于 [Cursor Light](https://github.com/Cave-znl/cursor-light)（MIT 协议）fork 改造：

- 移除 Cursor Hooks 监听 → 替换为 HTTP API 直控
- 保留 Tauri 窗口全部功能（置顶/吸附/横竖/右键菜单）
- 三色升级为四态（新增 waiting 橙闪状态）
- 新增 Windows 开机自启注册功能
- 端口从 18765 改为 37421

## 技术栈

- **前端**：纯 HTML + CSS + JS（无框架）
- **后端**：Rust + Tauri v2
- **HTTP**：TcpListener（无第三方依赖）
- **注册表**：winreg crate（仅 Windows）
- **包体**：~5MB 便携版 exe
