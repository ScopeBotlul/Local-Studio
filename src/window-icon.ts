import { invoke } from '@tauri-apps/api/core';

export function accentInk(accent: string): '#102421' | '#ffffff' {
  const match = /^#([0-9a-f]{6})$/i.exec(accent);
  if (!match) return '#102421';
  const value = Number.parseInt(match[1], 16);
  const r = value >> 16;
  const g = (value >> 8) & 255;
  const b = value & 255;
  return (r * 299 + g * 587 + b * 114) / 1000 < 142 ? '#ffffff' : '#102421';
}

function roundedRect(context: CanvasRenderingContext2D, x: number, y: number, width: number, height: number, radius: number) {
  context.beginPath();
  context.moveTo(x + radius, y);
  context.lineTo(x + width - radius, y);
  context.quadraticCurveTo(x + width, y, x + width, y + radius);
  context.lineTo(x + width, y + height - radius);
  context.quadraticCurveTo(x + width, y + height, x + width - radius, y + height);
  context.lineTo(x + radius, y + height);
  context.quadraticCurveTo(x, y + height, x, y + height - radius);
  context.lineTo(x, y + radius);
  context.quadraticCurveTo(x, y, x + radius, y);
  context.closePath();
}

export async function applyAccentWindowIcon(accent: string): Promise<void> {
  const canvas = document.createElement('canvas');
  canvas.width = 64;
  canvas.height = 64;
  const context = canvas.getContext('2d');
  if (!context) throw new Error('window_icon_canvas');
  // Keep the native icon in step with the three-bar mark rendered by Logo in
  // App.tsx and styles.css (43px square, 12px corner radius, 4px bars).
  const scale = 64 / 43;
  roundedRect(context, 0, 0, 64, 64, 12 * scale);
  context.fillStyle = /^#[0-9a-f]{6}$/i.test(accent) ? accent : '#4b9f91';
  context.fill();
  context.fillStyle = accentInk(accent);
  context.save();
  context.translate(32, 32);
  context.transform(1, -Math.tan(25 * Math.PI / 180), 0, 1, 0, 0);
  for (const [x, height] of [[-9.5, 19], [-2.5, 25], [4.5, 14]] as const) {
    roundedRect(context, x * scale, -height * scale / 2, 4 * scale, height * scale, 2 * scale);
    context.fill();
  }
  context.restore();
  const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob(value => value ? resolve(value) : reject(new Error('window_icon_png')), 'image/png'));
  const png = new Uint8Array(await blob.arrayBuffer());
  await invoke('window_accent_icon', { png: Array.from(png) });
}
