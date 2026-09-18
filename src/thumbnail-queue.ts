// Do not enqueue an entire directory in native IPC. Unmounted cards can cancel
// pending work; at most two requests from visible cards are active at a time.
export function createThumbnailQueue(limit = 2) {
  let running = 0;
  const pending: Array<{ run: () => Promise<void>; cancelled: boolean }> = [];
  const pump = () => {
    while (running < limit && pending.length) {
      const job = pending.shift()!;
      if (job.cancelled) continue;
      running++;
      void Promise.resolve().then(job.run).catch(() => {}).finally(() => { running--; pump(); });
    }
  };
  return (run: () => Promise<void>) => {
    const job = { run, cancelled: false };
    pending.push(job); pump();
    return () => { job.cancelled = true; const index = pending.indexOf(job); if (index >= 0) pending.splice(index, 1); };
  };
}
export const enqueueThumbnail = createThumbnailQueue();
