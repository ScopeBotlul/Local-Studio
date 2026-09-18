import { useEffect, useRef, useState } from 'react';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { inDesktop } from './api';

export function useFileDrop(disabled: boolean, addProject: (paths: string[]) => Promise<boolean>, onError: (error: unknown) => void) {
  const current = useRef({ disabled, addProject, onError }); current.current = { disabled, addProject, onError };
  const [target, setTarget] = useState<string | null>(null);
  useEffect(() => {
    if (!inDesktop()) return;
    let disposed = false; let sequence = 0; let off: (() => void) | undefined;
    void getCurrentWebview().onDragDropEvent(async event => {
      const serial = ++sequence;
      if (event.payload.type === 'leave') { setTarget(null); return; }
      try {
        const factor = await getCurrentWindow().scaleFactor();
        if (disposed || (serial !== sequence && event.payload.type !== 'drop')) return;
        const point = event.payload.position.toLogical(factor);
        const element = document.elementFromPoint(point.x, point.y)?.closest<HTMLElement>('[data-file-drop]');
        const blocked = current.current.disabled || document.querySelector('dialog[open], [aria-modal="true"]') || element?.closest('[inert]');
        const destination = blocked ? null : element?.dataset.fileDrop ?? null;
        setTarget(event.payload.type === 'drop' ? null : destination);
        if (event.payload.type !== 'drop' || !destination) return;
        const paths = event.payload.paths;
        if (destination === 'project') await current.current.addProject(paths);
        if (destination === 'gallery') window.dispatchEvent(new CustomEvent('studio-file-drop', { detail: { paths } }));
      } catch (error) { if (!disposed) { setTarget(null); current.current.onError(error); } }
    }).then(unlisten => { if (disposed) unlisten(); else off = unlisten; }).catch(e => current.current.onError(e));
    return () => { disposed = true; off?.(); };
  }, []);
  return target;
}
