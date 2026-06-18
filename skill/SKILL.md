---
title: "Light for All Agent"
summary: "AI Agent 桌面状态指示灯 skill — 指导 Agent 通过 HTTP API 更新灯色状态"
since: "2026-06-18"
category: "tools"
---

# Light for All Agent — Agent Skill

当用户已启动 Light for All Agent（即本项目的桌面状态灯程序），你（AI Agent）需要在关键操作节点主动推送灯色状态，让用户实时感知你的工作进度。

## 前提

Light for All Agent 已启动运行（监听 `127.0.0.1:37421`）。

> 应用常驻系统托盘（右下角），不在任务栏显示。窗口可能被用户隐藏，但 HTTP API 始终可用。若 `POST /status` 成功即说明灯端在运行，无需关心窗口是否可见。

## 状态枚举

| mode | 灯色 | 含义 | 发送时机 |
|------|------|------|---------|
| `idle` | 🟢 绿 | 空闲/完成 | 任务结束，等待用户输入 |
| `working` | 🟡 黄 | 工作中 | 执行命令、搜索、读写文件 |
| `waiting` | 🟠 橙闪 | 等待用户 | 需要用户确认/选择/回答 |
| `error` | 🔴 红 | 失败 | 命令报错、任务异常终止 |

## HTTP API

```
POST http://127.0.0.1:37421/status
Content-Type: application/json

{"mode": "working", "message": "可选说明"}
```

## 调用时机（必须遵守）

在以下节点必须调用：

1. **收到用户指令后开始执行** → `working`
2. **准备执行 bash 命令前** → `working`
3. **需要用户确认/选择时** → `waiting`
4. **用户确认后继续** → `working`
5. **命令执行失败** → `error`（附带错误信息）
6. **全部任务完成** → `idle`

## CURL 示例

```bash
# Bash 中调用
curl -s -X POST http://127.0.0.1:37421/status \
  -H "Content-Type: application/json" \
  -d '{"mode":"working","message":"正在安装依赖"}'
```

```powershell
# PowerShell 中调用
Invoke-RestMethod -Uri http://127.0.0.1:37421/status \
  -Method Post \
  -ContentType "application/json" \
  -Body '{"mode":"working","message":"正在安装依赖"}'
```

## 本地实现（无 curl 时）

若环境无 curl/PowerShell，用以下 Rust 原生方式或 Python：

```python
import urllib.request
import json

def set_light(mode: str, message: str = ""):
    data = json.dumps({"mode": mode, "message": message}).encode()
    req = urllib.request.Request(
        "http://127.0.0.1:37421/status",
        data=data,
        headers={"Content-Type": "application/json"}
    )
    urllib.request.urlopen(req, timeout=2)
```

## ⚠️ 铁律（优先级最高）

**灯只是给用户看的信号，绝对不能影响实际工作。**

- 灯接口连通 → 同步推送灯色（锦上添花）
- 灯接口不通 → **静默忽略，零影响继续干活**
- 灯接口超时 → 超时后丢弃请求，不重试不阻塞
- **任何时候**，如果灯相关的操作会让任务变慢、出错或分心——直接跳过，不需要通知用户

## 注意事项

- 端口固定 `37421`，不要改
- 仅在状态切换时发送，不要循环刷
- 发送失败（连接拒绝）说明灯未启动，静默忽略，不影响主任务
- `message` 字段可选，推荐在 `working` 和 `error` 状态下附带简要说明
