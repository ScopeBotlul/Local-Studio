import { getCurrentWindow } from '@tauri-apps/api/window';

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
  roundedRect(context, 4, 4, 56, 56, 13);
  context.fillStyle = /^#[0-9a-f]{6}$/i.test(accent) ? accent : '#4b9f91';
  context.fill();
  context.fillStyle = accentInk(accent);
  context.save();
  context.translate(32, 32);
  context.transform(1, -0.22, 0, 1, 0, 0);
  for (const [x, height] of [[-13, 27], [-3, 37], [7, 21]] as const) {
    roundedRect(context, x, -height / 2, 6, height, 3);
    context.fill();
  }
  context.restore();
  const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob(value => value ? resolve(value) : reject(new Error('window_icon_png')), 'image/png'));
  await getCurrentWindow().setIcon(new Uint8Array(await blob.arrayBuffer()));
}
