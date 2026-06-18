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

### 1. 启动

双击 `Light-for-all-Agent-0.1.0-x64-portable.exe`，桌面角落出现三色灯。

### 2. 与 Agent 联动

在 WorkBuddy / 其他 AI 工具中，Agent 会自动通过 HTTP 更新状态：

```bash
# 告诉灯：我正在工作
curl -X POST http://127.0.0.1:37421/status \
  -H "Content-Type: application/json" \
  -d '{"mode": "working"}'

# 告诉灯：搞定了
curl -X POST http://127.0.0.1:37421/status \
  -H "Content-Type: application/json" \
  -d '{"mode": "idle"}'
```

### 3. 右键菜单

- **横向 / 竖向** — 切换灯条方向
- **开机自启** — 注册 Windows 开机启动
- **退出** — 关闭应用

## 状态说明

| mode | 灯色 | 含义 | 视觉 |
|------|------|------|------|
| `idle` | 🟢 绿 | 空闲、完成 | 常亮 |
| `working` | 🟡 黄 | 正在执行 | 常亮 |
| `waiting` | 🟠 橙 | 等待用户确认 | 闪烁 |
| `error` | 🔴 红 | 失败、异常 | 常亮 |

## 技术栈

- **前端**：HTML + CSS + JS（无框架，纯静态）
- **后端**：Rust + Tauri v2
- **通信**：HTTP REST API（`127.0.0.1:37421`）
- **包体**：~5MB 便携版 exe

## 开发

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
