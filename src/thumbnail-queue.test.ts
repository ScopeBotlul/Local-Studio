import { describe, expect, it } from 'vitest';
import { createThumbnailQueue } from './thumbnail-queue';

describe('thumbnail scheduling', () => {
  it('bounds active work, drops obsolete queued cards and continues after failure', async () => {
    const enqueue = createThumbnailQueue(2);
    const started: string[] = [];
    let finishA!: () => void; let finishB!: () => void;
    const a = new Promise<void>(resolve => { finishA=resolve; });
    const b = new Promise<void>(resolve => { finishB=resolve; });
    enqueue(async () => { started.push('a'); await a; });
    enqueue(async () => { started.push('b'); await b; throw new Error('damaged image'); });
    const cancel = enqueue(async () => { started.push('obsolete'); });
    const done = new Promise<void>(resolve => enqueue(async () => { started.push('next'); resolve(); }));
    await Promise.resolve(); expect(started).toEqual(['a','b']); cancel(); finishB();
    await done; expect(started).toEqual(['a','b','next']); finishA();
  });
});
