const fs = require("fs");
const path = require("path");

const releaseName = "Light-for-all-Agent";
const buildDir = "src-tauri/target/release";
const distDir = "dist";

const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));
const version = pkg.version;

if (!fs.existsSync(distDir)) {
  fs.mkdirSync(distDir, { recursive: true });
}

const exeName = "light-for-all-agent.exe";
const src = path.join(buildDir, exeName);
const dest = path.join(distDir, `${releaseName}-${version}-x64-portable.exe`);

// 运行时依赖 DLL（WebView2 加载器），exe 同目录必须存在才能启动
const runtimeDeps = ["WebView2Loader.dll"];

if (fs.existsSync(src)) {
  fs.copyFileSync(src, dest);
  console.log(`✅ Release exe copied to: ${dest}`);
  for (const dep of runtimeDeps) {
    const depSrc = path.join(buildDir, dep);
    const depDest = path.join(distDir, dep);
    if (fs.existsSync(depSrc)) {
      fs.copyFileSync(depSrc, depDest);
      console.log(`✅ Runtime DLL copied: ${dep}`);
    } else {
      console.warn(`⚠️  Runtime DLL not found, skipped: ${depSrc}`);
    }
  }
} else {
  console.error(`❌ Source exe not found: ${src}`);
  process.exit(1);
}
