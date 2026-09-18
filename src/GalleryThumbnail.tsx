import { useEffect, useRef, useState } from 'react';
import { Image, Music, Video } from 'lucide-react';
import { galleryApi, galleryError, type GalleryEntry, type GalleryThumbnailData } from './gallery-api';
import { videoFrame } from './video-thumbnail';
import { enqueueThumbnail } from './thumbnail-queue';

export default function GalleryThumbnail({ entry, rootId, epoch, de }: { entry: GalleryEntry; rootId: string; epoch: number; de: boolean }) {
  const host = useRef<HTMLSpanElement>(null);
  const [visible,setVisible] = useState(false);
  const [data,setData] = useState<GalleryThumbnailData|null>(null);
  const [error,setError] = useState('');
  useEffect(() => {
    if (!host.current || !['image','video'].includes(entry.kind)) return;
    const observer = new IntersectionObserver(entries => {
      if (entries.some(e => e.isIntersecting)) { setVisible(true); observer.disconnect(); }
    });
    observer.observe(host.current); return () => observer.disconnect();
  },[entry.kind]);
  useEffect(() => {
    setData(null); setError('');
    if (!visible || !['image','video'].includes(entry.kind)) return;
    let live = true; const abort = new AbortController();
    const cancel = enqueueThumbnail(async () => {
      if (!live) return;
      try {
        const query={rootId,path:entry.path,version:entry.thumbnailVersion};
        let next: GalleryThumbnailData;
        try { next = await galleryApi.thumbnail(query); }
        catch (e) { if (String(e)!=='gallery_video_frame') throw e; next=await galleryApi.storeVideoThumbnail(query,await videoFrame(rootId,entry.path,entry.thumbnailVersion,abort.signal)); }
        if (live) setData(next);
      } catch (e) { if (live) setError(galleryError(e,de)); }
    });
    return () => { live = false; abort.abort(); cancel(); };
  },[rootId,entry.path,entry.thumbnailVersion,entry.kind,epoch,visible,de]);
  const Icon = entry.kind==='image' ? Image : entry.kind==='video' ? Video : Music;
  return <span className="gallery-thumbnail" ref={host} title={error || (data && !data.cacheStored ? (de?'Vorschau verfügbar, Cache nicht beschreibbar.':'Preview available; cache is not writable.') : undefined)}>
    {data && !error ? <img src={data.dataUrl} alt="" width={data.width} height={data.height} decoding="async" onError={()=>setError(de?'Miniatur konnte nicht angezeigt werden.':'Thumbnail could not be displayed.')}/> : <><Icon size={32}/><span>{['image','video'].includes(entry.kind) ? (error ? (de?'Keine Miniatur':'No thumbnail') : (de?'Vorschau lädt …':'Loading preview …')) : entry.kind==='video' ? 'Video' : (de?'Audio / Musik':'Audio / Music')}</span></>}
  </span>;
}
