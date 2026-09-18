import { useEffect, useRef, useState } from 'react';

export default function GalleryImage({ url, zoom, actualSize, resetKey, alt, onError }: { url: string; zoom: number; actualSize: boolean; resetKey: number; alt: string; onError: () => void }) {
  const viewport = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 1, height: 1 });
  const [natural, setNatural] = useState({ width: 0, height: 0 });
  const drag = useRef<{ id: number; x: number; y: number; left: number; top: number } | null>(null);
  useEffect(() => {
    const element = viewport.current; if (!element) return;
    const observer = new ResizeObserver(() => setSize({ width: element.clientWidth, height: element.clientHeight })); observer.observe(element);
    return () => observer.disconnect();
  }, []);
  useEffect(() => { viewport.current?.scrollTo(0, 0); }, [url, actualSize, resetKey]);
  const uiScale = Number(getComputedStyle(document.documentElement).getPropertyValue('--ui-scale')) || 1;
  const scale = actualSize ? 1 / uiScale : Math.min(size.width / Math.max(1, natural.width), size.height / Math.max(1, natural.height));
  const width = natural.width * scale * zoom; const height = natural.height * scale * zoom;
  return <div ref={viewport} className="gallery-image-scroll" style={{ height: '55vh', cursor: zoom > 1 || actualSize ? 'grab' : 'default' }} onPointerDown={e => {
    if (e.button !== 0) return; const el = e.currentTarget;
    el.closest<HTMLElement>('.gallery-viewer')?.focus({ preventScroll: true });
    drag.current = { id: e.pointerId, x: e.clientX, y: e.clientY, left: el.scrollLeft, top: el.scrollTop }; el.setPointerCapture(e.pointerId); e.preventDefault();
  }} onPointerMove={e => { const start = drag.current; if (!start || start.id !== e.pointerId) return; e.currentTarget.scrollTo(start.left - (e.clientX - start.x) / uiScale, start.top - (e.clientY - start.y) / uiScale); }} onPointerUp={() => { drag.current = null; }} onLostPointerCapture={() => { drag.current = null; }} onPointerCancel={() => { drag.current = null; }}>
    <div style={{ width: Math.max(size.width, width), height: Math.max(size.height, height), display: 'grid', placeItems: 'center' }}><img src={url} alt={alt} draggable={false} style={{ width, height, maxWidth: 'none', objectFit: 'contain', visibility: natural.width ? 'visible' : 'hidden' }} onLoad={e => setNatural({ width: e.currentTarget.naturalWidth, height: e.currentTarget.naturalHeight })} onError={onError} /></div>
  </div>;
}
