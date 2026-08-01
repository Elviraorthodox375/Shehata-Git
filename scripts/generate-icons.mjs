/**
 * Shehata Git icon generator.
 *
 * Renders the brand mark (same geometry as public/logo-mark.svg) into the
 * PNG/ICO files Tauri needs — without any native dependencies.
 *
 * Usage: node scripts/generate-icons.mjs
 */
import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const OUT_DIR = join(dirname(fileURLToPath(import.meta.url)), "../apps/desktop/src-tauri/icons");

// Brand palette
const BG = [11, 13, 16]; // #0B0D10
const ACCENT = [85, 221, 185]; // #55DDB9
const ACCENT_STRONG = [46, 207, 159]; // #2ECF9F

// ---------------------------------------------------------------- PNG encode

function crc32(buf) {
  let table = crc32.table;
  if (!table) {
    table = crc32.table = new Uint32Array(256);
    for (let n = 0; n < 256; n++) {
      let c = n;
      for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
      table[n] = c >>> 0;
    }
  }
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) crc = table[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const out = Buffer.alloc(8 + data.length + 4);
  out.writeUInt32BE(data.length, 0);
  out.write(type, 4, "ascii");
  data.copy(out, 8);
  out.writeUInt32BE(crc32(out.subarray(4, 8 + data.length)), 8 + data.length);
  return out;
}

function encodePng(size, rgba) {
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // color type RGBA
  const stride = size * 4;
  const raw = Buffer.alloc((stride + 1) * size);
  for (let y = 0; y < size; y++) {
    raw[y * (stride + 1)] = 0; // filter: none
    rgba.copy(raw, y * (stride + 1) + 1, y * stride, (y + 1) * stride);
  }
  const idat = deflateSync(raw, { level: 9 });
  return Buffer.concat([
    signature,
    chunk("IHDR", ihdr),
    chunk("IDAT", idat),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

// ------------------------------------------------------------- raster canvas

class Canvas {
  constructor(size) {
    this.size = size;
    this.scale = size / 64; // logo grid is 64x64
    this.data = Buffer.alloc(size * size * 4);
  }

  blend(x, y, [r, g, b], alpha) {
    const { size, data } = this;
    if (x < 0 || y < 0 || x >= size || y >= size) return;
    const i = (y * size + x) * 4;
    const a = alpha / 255;
    data[i] = Math.round(r * a + data[i] * (1 - a));
    data[i + 1] = Math.round(g * a + data[i + 1] * (1 - a));
    data[i + 2] = Math.round(b * a + data[i + 2] * (1 - a));
    data[i + 3] = Math.min(255, data[i + 3] + alpha);
  }

  disc(cx, cy, radius, color) {
    const { scale } = this;
    const x0 = Math.floor((cx - radius) * scale);
    const x1 = Math.ceil((cx + radius) * scale);
    const y0 = Math.floor((cy - radius) * scale);
    const y1 = Math.ceil((cy + radius) * scale);
    const r = radius * scale;
    const cxS = cx * scale;
    const cyS = cy * scale;
    for (let y = y0; y <= y1; y++) {
      for (let x = x0; x <= x1; x++) {
        const d = Math.hypot(x + 0.5 - cxS, y + 0.5 - cyS);
        const coverage = Math.max(0, Math.min(1, r + 0.5 - d)); // 1px AA
        if (coverage > 0) this.blend(x, y, color, Math.round(coverage * 255));
      }
    }
  }

  ring(cx, cy, radius, strokeWidth, color, innerColor) {
    this.disc(cx, cy, radius, color);
    this.disc(cx, cy, Math.max(0, radius - strokeWidth), innerColor);
  }

  strokePath(points, width, color) {
    const r = width / 2;
    for (let i = 0; i < points.length - 1; i++) {
      const [x0, y0] = points[i];
      const [x1, y1] = points[i + 1];
      const steps = Math.max(1, Math.ceil(Math.hypot(x1 - x0, y1 - y0) * 2));
      for (let s = 0; s <= steps; s++) {
        const t = s / steps;
        this.disc(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, r, color);
      }
    }
  }
}

// Cubic bezier sampling: B(t) for control points p0..p3
function cubic(p0, p1, p2, p3, segments = 120) {
  const pts = [];
  for (let i = 0; i <= segments; i++) {
    const t = i / segments;
    const mt = 1 - t;
    pts.push([
      mt ** 3 * p0[0] + 3 * mt ** 2 * t * p1[0] + 3 * mt * t ** 2 * p2[0] + t ** 3 * p3[0],
      mt ** 3 * p0[1] + 3 * mt ** 2 * t * p1[1] + 3 * mt * t ** 2 * p2[1] + t ** 3 * p3[1],
    ]);
  }
  return pts;
}

function renderMark(size, { withBackground }) {
  const c = new Canvas(size);
  const s = 64;
  if (withBackground) {
    // Rounded-rect background stamped as filled rows (radius 14/64)
    const radius = 14;
    for (let y = 0; y < s; y += 0.5) {
      for (let x = 0; x < s; x += 0.5) {
        const inX = Math.min(x, s - x);
        const inY = Math.min(y, s - y);
        let inside = inX >= 0 && inY >= 0;
        if (inX < radius && inY < radius) {
          inside = Math.hypot(radius - inX, radius - inY) <= radius;
        }
        if (inside) c.disc(x, y, 0.75, BG);
      }
    }
  }
  // Converging identity lines (stroke 3)
  c.strokePath([[11, 12], [23, 21.5]], 3, ACCENT);
  c.strokePath([[11, 26], [20, 27.5]], 3, ACCENT);
  // S trunk (stroke 6): M44 18.5 C36.5 11.5 21.5 13 20.5 23 C19.5 33 45 31.5 45 42 C45 52.5 28 55.5 20 46
  const sPath = [
    ...cubic([44, 18.5], [36.5, 11.5], [21.5, 13], [20.5, 23]),
    ...cubic([20.5, 23], [19.5, 33], [45, 31.5], [45, 42]),
    ...cubic([45, 42], [45, 52.5], [28, 55.5], [20, 46]),
  ];
  c.strokePath(sPath, 6, ACCENT);
  // Nodes
  c.disc(11, 12, 3.6, ACCENT);
  c.disc(11, 26, 3.6, ACCENT);
  c.ring(20.5, 23, 4.2, 2.6, ACCENT, BG);
  c.ring(45, 42, 4.2, 2.6, ACCENT_STRONG, BG);
  return c.data;
}

function encodeIco(pngs) {
  // ICONDIR + ICONDIRENTRY per image, PNG-compressed payloads (Vista+).
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(pngs.length, 4);
  let offset = 6 + 16 * pngs.length;
  const entries = [];
  for (const { size, data } of pngs) {
    const entry = Buffer.alloc(16);
    entry[0] = size >= 256 ? 0 : size; // 0 means 256
    entry[1] = size >= 256 ? 0 : size;
    entry[2] = 0;
    entry[3] = 0;
    entry.writeUInt16LE(1, 4); // planes
    entry.writeUInt16LE(32, 6); // bpp
    entry.writeUInt32LE(data.length, 8);
    entry.writeUInt32LE(offset, 12);
    entries.push(entry);
    offset += data.length;
  }
  return Buffer.concat([header, ...entries, ...pngs.map((p) => p.data)]);
}

// ---------------------------------------------------------------------- main

mkdirSync(OUT_DIR, { recursive: true });

const targets = [
  { file: "32x32.png", size: 32 },
  { file: "128x128.png", size: 128 },
  { file: "128x128@2x.png", size: 256 },
  { file: "icon.png", size: 512 },
];

for (const { file, size } of targets) {
  const rgba = renderMark(size, { withBackground: true });
  writeFileSync(join(OUT_DIR, file), encodePng(size, rgba));
  console.log(`wrote ${file} (${size}x${size})`);
}

const ico = encodeIco(
  [16, 32, 48, 256].map((size) => ({
    size,
    data: encodePng(size, renderMark(size, { withBackground: true })),
  })),
);
writeFileSync(join(OUT_DIR, "icon.ico"), ico);
console.log("wrote icon.ico (16, 32, 48, 256)");
