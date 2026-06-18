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

if (fs.existsSync(src)) {
  fs.copyFileSync(src, dest);
  console.log(`✅ Release exe copied to: ${dest}`);
} else {
  console.error(`❌ Source exe not found: ${src}`);
  process.exit(1);
}
