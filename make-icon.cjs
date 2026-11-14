// Generates a 1024x1024 app icon (PNG) with no external dependencies.
const zlib = require("zlib");
const fs = require("fs");

const S = 1024;
const SS = 2; // supersampling

function roundedRectDist(x, y, w, h, r) {
  const dx = Math.abs(x - w / 2) - (w / 2 - r);
  const dy = Math.abs(y - h / 2) - (h / 2 - r);
  const ax = Math.max(dx, 0);
  const ay = Math.max(dy, 0);
  return Math.hypot(ax, ay) + Math.min(Math.max(dx, dy), 0) - r;
}

function ghostAlpha(x, y) {
  // head
  const cx = 512;
  const headCy = 448;
  const headR = 232;
  if (Math.hypot(x - cx, y - headCy) <= headR && y <= headCy) return 1;

  // body rect
  const top = headCy;
  const bottomBase = 760;
  const halfW = headR;
  if (x >= cx - halfW && x <= cx + halfW && y >= top && y <= bottomBase) {
    // wavy bottom
    const t = (x - (cx - halfW)) / (halfW * 2); // 0..1
    const wave = 34 * Math.sin(t * Math.PI * 3 + Math.PI / 2);
    if (y <= bottomBase + wave) return 1;
  }
  return 0;
}

function eyeAlpha(x, y) {
  const eyes = [
    [432, 452, 40, 52],
    [592, 452, 40, 52],
  ];
  for (const [ex, ey, rx, ry] of eyes) {
    const d = ((x - ex) / rx) ** 2 + ((y - ey) / ry) ** 2;
    if (d <= 1) return 1;
  }
  return 0;
}

const buf = Buffer.alloc(S * S * 4);

for (let py = 0; py < S; py++) {
  for (let px = 0; px < S; px++) {
    let r = 0,
      g = 0,
      b = 0,
      a = 0;

    for (let sy = 0; sy < SS; sy++) {
      for (let sx = 0; sx < SS; sx++) {
        const x = px + (sx + 0.5) / SS;
        const y = py + (sy + 0.5) / SS;

        const inside = roundedRectDist(x, y, S, S, 210) <= 0;
        if (!inside) continue;

        // background gradient + radial glow
        const t = y / S;
        let br = 13 + t * 4;
        let bg = 18 + t * 6;
        let bb = 27 + t * 10;

        const gd = Math.hypot(x - 512, y - 460) / 620;
        const glow = Math.max(0, 1 - gd) ** 2 * 0.55;
        br += 30 * glow;
        bg += 76 * glow;
        bb += 150 * glow;

        // ghost body
        const ga = ghostAlpha(x, y);
        if (ga > 0) {
          const sh = Math.max(0, Math.min(1, (x - 280) / 460));
          br = br * (1 - ga) + (200 + 40 * (1 - sh)) * ga;
          bg = bg * (1 - ga) + (226 + 22 * (1 - sh)) * ga;
          bb = bb * (1 - ga) + (255) * ga;
        }

        // eyes
        const ea = eyeAlpha(x, y);
        if (ea > 0) {
          br = br * (1 - ea) + 12 * ea;
          bg = bg * (1 - ea) + 22 * ea;
          bb = bb * (1 - ea) + 48 * ea;
        }

        r += br;
        g += bg;
        b += bb;
        a += 255;
      }
    }

    const n = SS * SS;
    const i = (py * S + px) * 4;
    buf[i] = Math.round(r / n);
    buf[i + 1] = Math.round(g / n);
    buf[i + 2] = Math.round(b / n);
    buf[i + 3] = Math.round(a / n);
  }
}

// --- PNG encode ---
function crc32(buf) {
  let c;
  const table = [];
  for (let n = 0; n < 256; n++) {
    c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  let crc = 0xffffffff;
  for (let i = 0; i < buf.length; i++) crc = table[(crc ^ buf[i]) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length);
  const td = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(td));
  return Buffer.concat([len, td, crc]);
}

const raw = Buffer.alloc(S * (S * 4 + 1));
for (let y = 0; y < S; y++) {
  raw[y * (S * 4 + 1)] = 0;
  buf.copy(raw, y * (S * 4 + 1) + 1, y * S * 4, (y + 1) * S * 4);
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(S, 0);
ihdr.writeUInt32BE(S, 4);
ihdr[8] = 8;
ihdr[9] = 6;

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", zlib.deflateSync(raw, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);

fs.writeFileSync("icon-src.png", png);
console.log("wrote icon-src.png", png.length, "bytes");
