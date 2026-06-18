# Light for All Agent — 流程文档

## 目的

为开发人员、测试人员、以及 AI Agent 使用者提供统一的本地测试、打包、部署和运行流程。

---

## 1. 本地开发与测试

### 环境准备

本地构建需要以下组件（均为一次安装）：

| 组件 | 用途 | 安装方式 |
|------|------|----------|
| Node.js + npm | 前端依赖、调用 Tauri CLI | 官网安装包 |
| Rust 工具链（rustup） | 编译 Rust 后端 | https://rustup.rs |
| MSYS2 + MinGW-w64 | 提供 GNU 链接器（`gcc`/`dlltool`/`ld`） | winget 安装 |

> 本项目在 Windows 上使用 **GNU 工具链**（`x86_64-pc-windows-gnu`），**无需安装 Visual Studio / VS Build Tools**。链接器由 MSYS2 的 MinGW-w64 提供。

### 完整搭建步骤（Windows，从零开始）

以下流程已在 Windows 10/11 + PowerShell 上实测通过。

#### 步骤 1：安装 Node.js

从 https://nodejs.org 下载 LTS 安装包，按默认选项安装即可。验证：

```powershell
node --version
npm --version
```

> 💡 **若在 CodeBuddy IDE 内运行**：IDE 自带 Node.js，路径为
> `C:\Users\<用户名>\.workbuddy\binaries\node\versions\<版本号>\`（如 `22.22.2`）。
> 该路径未默认加入系统 PATH，在 IDE 外的终端需手动拼接或改用系统级安装的 Node.js。
> IDE 内的命令执行环境会自动识别该 Node，`npm install` / `npm run dev` 可直接使用。

#### 步骤 2：安装 Rust 工具链（rustup）

从 https://rustup.rs 下载 `rustup-init.exe` 并运行。安装时默认选择 `x86_64-pc-windows-gnu` 作为 host triple（若已装成 msvc 也可后续切换）。

验证：

```powershell
rustc --version
cargo --version
```

#### 步骤 3：安装项目指定版本的 Rust

项目通过 `src-tauri/rust-toolchain.toml` 锁定 Rust **1.94.0**。在项目 `src-tauri/` 目录下执行任意 `cargo` 命令时，rustup 会自动安装该版本；也可手动预装：

```powershell
rustup toolchain install 1.94.0-x86_64-pc-windows-gnu --profile minimal
```

> ⚠️ **不要使用 Rust 1.96+**，与依赖中的 `smallvec`、`syn` 等 crate 存在已知不兼容问题。

#### 步骤 4：安装 MSYS2 + MinGW-w64（提供链接器）

GNU 工具链编译时需要 `dlltool.exe`、`gcc.exe`、`ld.exe`，由 MSYS2 的 MinGW-w64 包提供。

```powershell
# 1) 安装 MSYS2
winget install MSYS2.MSYS2 --accept-package-agreements --accept-source-agreements

# 2) 用 MSYS2 的 pacman 安装 MinGW-w64 工具链（含 gcc/dlltool/ld/make 等一整套）
& "C:\msys64\usr\bin\pacman.exe" -Syu --noconfirm
& "C:\msys64\usr\bin\pacman.exe" -S --needed --noconfirm mingw-w64-x86_64-toolchain
```

#### 步骤 5：将 MinGW 加入系统 PATH

将 `C:\msys64\mingw64\bin` 永久加入用户 PATH（PowerShell 执行一次即可）：

```powershell
$mingwPath = "C:\msys64\mingw64\bin"
$current = [Environment]::GetEnvironmentVariable("Path", "User")
if ($current -notlike "*$mingwPath*") {
    [Environment]::SetEnvironmentVariable("Path", $current + ";" + $mingwPath, "User")
}
```

**重新打开终端**后验证链接器可用：

```powershell
dlltool --version   # 应输出 GNU Binutils 版本
gcc --version       # 应输出 gcc 版本
```

#### 步骤 6：安装前端依赖并构建

```powershell
cd <仓库根目录>
npm install
cd src-tauri
cargo build
```

看到 `Finished dev profile ...` 即表示后端编译成功。

### Rust 版本说明

- 项目通过 `src-tauri/rust-toolchain.toml` 锁定 Rust 版本
- **当前要求：** 使用 GNU 工具链（`x86_64-pc-windows-gnu`），Rust **1.94.0**
- **Rust 1.96+** 与依赖中的 `smallvec`、`syn` 等 crate 存在已知不兼容问题
- 切换版本命令：
  ```powershell
  rustup toolchain install 1.94.0-x86_64-pc-windows-gnu --profile minimal
  rustup default 1.94.0-x86_64-pc-windows-gnu
  rustc --version
  ```

### 为什么不用 MSVC 工具链？

项目可选用 MSVC（`x86_64-pc-windows-msvc`），但需要额外安装 Visual Studio Build Tools（约 3-5GB），而 GNU 方案仅需 MSYS2（约 500MB）。本项目所有依赖在 GNU 工具链下编译正常，推荐 GNU 路线以保持环境轻量。若确需 MSVC，参考下方[常见问题 6.3.3](#633-linkexe-not-found)。

### 安装依赖

```bash
npm install
```

### 启动开发模式

```bash
npm run dev
```

说明：

- 该命令会启动 Tauri 开发环境
- 启动后，悬浮状态灯窗口会显示在桌面上
- 应用会监听 `http://127.0.0.1:37421`

### 验证状态切换

在应用运行时执行以下命令，确认页面灯色和状态是否正常：

```bash
npm run simulate:idle
npm run simulate:working
npm run simulate:waiting
npm run simulate:error
```

如果你想直接调用 API：

```bash
curl -X POST http://127.0.0.1:37421/status \
  -H "Content-Type: application/json" \
  -d '{"mode":"working","message":"本地测试"}'
```

---

## 2. 构建与打包

### 生成便携版 exe

```bash
npm run dist
```

说明：

- 该命令会调用 `tauri build --no-bundle`
- 构建完成后，会把可执行文件复制到仓库根目录的 `dist/`
- 生成文件名示例：`dist/Light-for-all-Agent-0.1.0-x64-portable.exe`

### 生成安装程序

```bash
npm run dist:installer
```

说明：

- 该命令会生成 NSIS 安装包
- 需要 Windows 打包工具链和 NSIS 环境支持

### 构建产物说明

- `dist/Light-for-all-Agent-0.1.0-x64-portable.exe`：便携版运行文件
- `src-tauri/target/release`：Tauri build 的默认产物目录

---

## 3. 部署与运行

### 直接部署

1. 复制 `dist/Light-for-all-Agent-0.1.0-x64-portable.exe` 到目标机器
2. 双击运行
3. 桌面出现悬浮状态灯

### 安装部署

1. 运行 NSIS 安装包（`npm run dist:installer` 生成）
2. 按安装程序提示完成安装
3. 启动应用

### 运行后的效果

- 应用启动后，会在 Windows 桌面上常驻一个悬浮窗口
- 该窗口支持拖动、吸附、横向/竖向切换、右键菜单和开机自启
- 应用同时监听 `127.0.0.1:37421`

---

## 4. Agent 状态更新标准流程

### 4.1 状态定义

| mode | 灯色 | 含义 |
|------|------|------|
| `idle` | 🟢 绿 | 任务完成、已空闲 |
| `working` | 🟡 黄 | 正在执行任务 |
| `waiting` | 🟠 橙 | 需要用户确认、等待输入 |
| `error` | 🔴 红 | 发生异常或失败 |

### 4.2 推荐使用时机

- 开始执行任务时：`working`
- 需要用户选择/确认时：`waiting`
- 任务结束并返回结果时：`idle`
- 出错时：`error`

### 4.3 API 调用格式

#### 请求

```http
POST /status
Content-Type: application/json

{"mode": "working", "message": "开始执行任务"}
```

#### 示例

```bash
curl -X POST http://127.0.0.1:37421/status \
  -H "Content-Type: application/json" \
  -d '{"mode":"waiting","message":"等待用户确认"}'
```

### 4.4 Agent 集成建议

- 在 Agent 操作链的关键点插入状态上报
- 只在状态变化时发起请求，避免频繁无意义刷新
- `message` 字段可选，建议用于说明当前步骤或错误原因
- 如果 `GET /status` 支持，可用于检查灯端是否正常运行

---

## 5. 角色指南

### 开发人员

- 关注 `README.md` 的开发和打包流程
- 使用 `npm run dev` 开发、`npm run dist` 打包
- 遇到接口逻辑问题，可参考 `src-tauri` 中的 Tauri 处理逻辑

### 测试人员

- 启动应用后使用 `npm run simulate:*` 验证四个状态
- 通过 `curl` 或 Agent 模拟请求确认 HTTP 接口是否可达
- 验证打包产物 `dist/Light-for-all-Agent-*.exe` 是否可正常启动

### Agent 使用者

- 先确认灯端已启动
- 使用 `POST /status` 发送状态
- 仅发送合法四个 mode 之一
- 结束流程后保持 `idle` 状态

---

## 6. 常见问题

### 6.1 无法连接 `127.0.0.1:37421`

- 检查应用是否已启动
- 检查是否有防火墙或安全软件阻止本地请求
- 确认端口未被其他程序占用

### 6.2 发送状态后灯不变

- 检查请求格式是否正确
- 确认 `Content-Type: application/json`
- 确认 `mode` 字段值为 `idle`、`working`、`waiting` 或 `error`

### 6.3 常见本地构建问题

#### 6.3.1 `tauri.conf.json` 格式错误

- 错误表现：`"identifier" is a required property` 或 `Additional properties are not allowed`
- 解决方式：`src-tauri/tauri.conf.json` 需使用 Tauri v2 当前 schema，关键字段为：
  - `identifier`
  - `productName`
  - `version`
  - `build.frontendDist`
  - `app.withGlobalTauri`
  - `app.security.csp`
  - `app.windows`
- 如果你遇到该问题，建议直接使用仓库中已修复的 `src-tauri/tauri.conf.json` 文件。

#### 6.3.2 `dlltool.exe: program not found`

- 说明当前使用的是 GNU 工具链（`x86_64-pc-windows-gnu`），但缺少 MinGW binutils
- 解决方式（推荐，参考[步骤 4](#步骤-4安装-msys2--mingw-w64提供链接器)）：
  ```powershell
  winget install MSYS2.MSYS2
  & "C:\msys64\usr\bin\pacman.exe" -S --needed --noconfirm mingw-w64-x86_64-toolchain
  # 然后将 C:\msys64\mingw64\bin 加入 PATH（见步骤 5）
  ```

#### 6.3.3 `link.exe not found`

- 错误表现：`error: linker link.exe not found`
- 说明当前已切换到 MSVC 工具链，但未安装 Visual C++ 链接器
- 解决方式：
  1. 安装 Visual Studio Build Tools 或 Visual Studio
  2. 勾选"Desktop development with C++"工作负载
  3. 完成安装后，重新启动命令提示符或 PowerShell
  4. 在项目目录执行：
     ```powershell
     rustup override set stable-x86_64-pc-windows-msvc
     npm run dev
     ```
- 额外说明：
  - 如果你已经安装了 Visual Studio 但仍报错，确认 `Developer Command Prompt for VS` 能调用 `link.exe`
  - 也可通过 Visual Studio Installer 对现有安装追加 `MSVC v143 - VS 2022 C++ x64/x86 build tools`
  - **本项目推荐 GNU 路线，通常无需走 MSVC**

#### 6.3.4 `failed to decode icon ... Malformed PNG data: CRC error`

- 错误表现：`cargo build` 在 `tauri::generate_context!()` 处报 `proc macro panicked`，提示 `icon.ico` 的 PNG chunk CRC 错误
- 原因：`src-tauri/icons/icon.ico` 文件损坏或为占位符（`scripts/generate-icon.js` 旧版本生成的 ICO 内嵌 PNG 的 CRC 是占位符 `0x0`）
- 解决方式：重新生成合法的 ICO 文件。Tauri 的 ICO 解码器要求内嵌 **BMP 格式**（非 PNG）：
  ```powershell
  # 用 Python 生成 BMP-based 多分辨率 ICO（16/32/48/64，绿色填充）
  python -c "import struct,zlib,io
def bmp(s):
    rb=s*4; d=struct.pack('<IiiHHIIiiII',40,s,s*2,1,32,0,rb*s,0,0,0,0)
    px=bytes([0x74,0xdc,0x35,0xff])*s
    return d+b''.join(px*s for _ in range(s))
def ico(szs):
    n=len(szs); h=struct.pack('<HHH',0,1,n); img=[]; ent=[]; off=6+n*16
    for s in szs:
        b=bmp(s); img.append(b); wb=s if s<256 else 0
        ent.append(struct.pack('<BBBBHHII',wb,wb,0,0,1,32,len(b),off)); off+=len(b)
    return h+b''.join(ent)+b''.join(img)
open(r'src-tauri/icons/icon.ico','wb').write(ico([16,32,48,64]))
print('icon.ico regenerated')"
  ```
- 注意：`scripts/generate-icon.js` 依赖 Node.js，若当前环境无 Node，可直接用上面的 Python 方案。

#### 6.3.5 Rust 编译报错 `STATUS_STACK_BUFFER_OVERRUN` 或 OS error 1455

- 错误表现：编译器崩溃，退出码 `0xc0000409`
- 原因：Rust 1.96+ 与 MSVC 工具链的 CET/Shadow Stack 保护不兼容
- 解决方式：改用 GNU 工具链，参考 [6.3.2](#632-dlltoolexe-program-not-found)

#### 6.3.6 crate 编译报错（`ambiguous associated type`、`cannot find trait Default` 等）

- 错误表现：`smallvec` / `syn` / `serde_core` 等 crate 报 100+ 编译错误
- 原因：Rust 1.96+ 中 trait 解析规则变更
- 解决方式：降级 Rust 版本至 `1.94.0`，详见上方 **Rust 版本说明**

#### 6.3.7 `页面文件太小，无法完成操作 (os error 1455)`

- 错误表现：`could not execute process ... never executed`
- 原因：系统虚拟内存不足，build script 进程无法启动
- 解决方式：增大 Windows 虚拟内存，或关闭其他占用内存的应用

---

## 7. 参考文档

- `README.md`
- `AGENT.md`
- `docs/design.md`
