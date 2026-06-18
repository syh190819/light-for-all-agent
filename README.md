# Light for All Agent 🚦

通用 AI Agent 桌面状态指示灯。悬浮三色灯，实时显示 Agent 工作状态。

## 它是做什么的？

当你让 AI Agent 执行长任务时，不需要频繁切回窗口看它跑完了没。这个悬浮小灯会告诉你：

- 🟢 **绿灯** — 任务完成 / 空闲
- 🟡 **黄灯** — 正在工作
- 🟠 **橙灯闪烁** — 在等你确认
- 🔴 **红灯** — 出错了

## 功能特性

- 置顶悬浮小窗口，不占任务栏
- 支持拖动，靠近屏幕边缘自动吸附
- 横向 / 竖向布局切换（右键菜单）
- 开机自启（右键菜单开关）
- HTTP API 驱动，无 Hooks 依赖
- Tauri 原生壳，内存占用约 8-15MB
- 便携版 exe，无需安装

## 如何使用

### 核心流程

- 启动灯端应用（可直接运行已打包的便携 exe 或使用开发模式）
- 灯端会在本地监听 `http://127.0.0.1:37421`
- Agent 通过 `POST /status` 更新当前状态
- 支持状态：`idle`、`working`、`waiting`、`error`

### 运行简介

- 直接运行 `dist/Light-for-all-Agent-0.1.0-x64-portable.exe`，即可在桌面看到悬浮状态灯
- 右键菜单可切换横向/竖向、开机自启和退出
- 应用会实时展示状态变化，并通过 HTTP 接口与 Agent 联动

## 技术栈

- **前端**：HTML + CSS + JS（无框架，纯静态）
- **后端**：Rust + Tauri v2（GNU 工具链，锁定 Rust 1.94）
- **通信**：HTTP REST API（`127.0.0.1:37421`）
- **包体**：~5MB 便携版 exe

## 主要文档

- 设计与架构：`docs/design.md`
- 开发/测试/打包/部署流程：`docs/process.md`
- Agent 使用与集成：`AGENT.md`

## 开发

- `npm install`
- `npm run dev`
- `npm run dist`

更多详细流程请参见 `docs/process.md`。

```bash
# 安装依赖
npm install

# 开发模式
npm run dev

# 构建便携版 exe
npm run dist

# 模拟灯色测试
npm run simulate:working
npm run simulate:idle
npm run simulate:waiting
npm run simulate:error
```

## 许可证

MIT — Forked from [Cursor Light](https://github.com/Cave-znl/cursor-light)
