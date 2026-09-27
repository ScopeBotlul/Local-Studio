import {describe,expect,it,vi} from 'vitest';
import {saveWithGalleryFallback} from './image-api';

describe('gallery save fallback',()=>{
 it('retries once in the gallery root when a remembered folder is gone',async()=>{
  const save=vi.fn(async(_id:string,folder:string)=>{if(folder==='Deleted folder')throw 'gallery_missing';return 'gallery/image.png';});
  await expect(saveWithGalleryFallback('job-1','Deleted folder',save)).resolves.toEqual({path:'gallery/image.png',folder:''});
  expect(save.mock.calls).toEqual([['job-1','Deleted folder'],['job-1','']]);
 });

 it('does not hide storage failures or retry an already-rooted save',async()=>{
  const storage=vi.fn(async()=>{throw 'image_storage';});
  await expect(saveWithGalleryFallback('job-1','Folder',storage)).rejects.toBe('image_storage');
  expect(storage).toHaveBeenCalledTimes(1);
  const missingRoot=vi.fn(async()=>{throw 'gallery_missing';});
  await expect(saveWithGalleryFallback('job-2','',missingRoot)).rejects.toBe('gallery_missing');
  expect(missingRoot).toHaveBeenCalledTimes(1);
 });
});
