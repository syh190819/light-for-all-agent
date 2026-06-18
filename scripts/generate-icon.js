#!/usr/bin/env node
/**
 * Generate icons for Light for All Agent
 * Produces: build/icon.svg, build/icon.ico (multi-res)
 * Based on Cursor Light's generate-icon.js
 */
const fs = require("fs");
const path = require("path");

function generateSvg() {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
  <defs>
    <linearGradient id="case" x1="0" x2="0" y1="0" y2="1">
      <stop offset="0" stop-color="#24282c"/>
      <stop offset="1" stop-color="#101214"/>
    </linearGradient>
    <filter id="glow" x="-60%" y="-60%" width="220%" height="220%">
      <feDropShadow dx="0" dy="0" stdDeviation="12" flood-color="#35dc74" flood-opacity=".9"/>
    </filter>
  </defs>
  <rect x="26" y="18" width="204" height="220" rx="42" fill="url(#case)" stroke="#ffffff" stroke-opacity=".16" stroke-width="8"/>
  <circle cx="128" cy="70" r="35" fill="#070809"/>
  <circle cx="128" cy="70" r="24" fill="#ff4037" opacity=".45"/>
  <circle cx="128" cy="128" r="35" fill="#070809"/>
  <circle cx="128" cy="128" r="24" fill="#ffd449" opacity=".5"/>
  <circle cx="128" cy="186" r="35" fill="#070809"/>
  <circle cx="128" cy="186" r="25" fill="#35dc74" filter="url(#glow)"/>
</svg>`;
}

function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

const buildDir = path.join(__dirname, "..", "build");
ensureDir(buildDir);

// Write SVG
fs.writeFileSync(path.join(buildDir, "icon.svg"), generateSvg(), "utf8");
console.log("✅ icon.svg generated");

// Simple ICO header for a 256x256 32bpp PNG-based icon
// Tauri can use SVG directly, so we just need a placeholder .ico
// For production, use a proper tool like png2ico
const icoDir = path.join(__dirname, "..", "src-tauri", "icons");
ensureDir(icoDir);

// Write a minimal valid ICO (points to a 256x256 entry)
// This is a placeholder - in production, convert the SVG
function generateMinimalIco() {
  // Create a minimal valid .ico with one 256x256 entry
  // ICO header: reserved(2) + type(2) + count(2) = 6 bytes
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);     // reserved
  header.writeUInt16LE(1, 2);     // type: ICO
  header.writeUInt16LE(1, 4);     // count: 1

  // Directory entry: 16 bytes
  // w, h (0=256), colors, reserved, planes, bpp, size, offset
  const entry = Buffer.alloc(16);
  entry.writeUInt8(0, 0);         // width (0 = 256)
  entry.writeUInt8(0, 1);         // height (0 = 256)
  entry.writeUInt8(0, 2);         // colors
  entry.writeUInt8(0, 3);         // reserved
  entry.writeUInt16LE(1, 4);      // color planes
  entry.writeUInt16LE(32, 6);     // bits per pixel

  // We'll store the PNG data after the header+entry
  // For now, create a minimal PNG (1x1 transparent)
  const png = Buffer.from([
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
    0x49, 0x48, 0x44, 0x52, // "IHDR"
    0x00, 0x00, 0x00, 0x01, // width: 1
    0x00, 0x00, 0x00, 0x01, // height: 1
    0x08, 0x02,             // bit depth: 8, color type: RGB
    0x00, 0x00, 0x00,       // compression, filter, interlace
    0x00, 0x00, 0x00, 0x00, // CRC (placeholder)
    0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, // IEND chunk
    0xAE, 0x42, 0x60, 0x82  // IEND CRC
  ]);

  const totalSize = header.length + entry.length + png.length;
  entry.writeUInt32LE(png.length, 8);  // image size
  entry.writeUInt32LE(header.length + entry.length, 12); // image offset

  return Buffer.concat([header, entry, png]);
}

const icoPath = path.join(icoDir, "icon.ico");
fs.writeFileSync(icoPath, generateMinimalIco());
console.log("✅ icon.ico generated (placeholder, replace with real icon)");
