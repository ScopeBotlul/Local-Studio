import {describe,it,expect} from 'vitest';
import {combinedDownloadProgress,downloadEta} from './DownloadOverview';
describe('Download ETA',()=>{
 it('does not invent a duration without a usable measurement',()=>{
  for(const speed of [0,-1,NaN,Infinity])expect(downloadEta(100,speed,true)).toContain('ermittelt');
  expect(downloadEta(0,100,true)).toContain('ermittelt');
 });
 it('rounds up remaining time into readable units',()=>{
  expect(downloadEta(15,1,true)).toBe('Noch ca. < 1 Min.');
  expect(downloadEta(61,1,true)).toBe('Noch ca. 2 Min.');
  expect(downloadEta(3661,1,true)).toBe('Noch ca. 1 Std. 2 Min.');
  expect(downloadEta(3600,1,false)).toBe('About 1 h left');
 });
});

describe('combined download progress',()=>{
 it('is hidden without active downloads',()=>{
  expect(combinedDownloadProgress([])).toBeNull();
 });
 it('combines downloads by transferred bytes',()=>{
  expect(combinedDownloadProgress([{total:100,received:50},{total:300,received:75}])).toEqual({count:2,determinate:true,percent:31.25});
 });
 it('clamps invalid transferred byte values',()=>{
  const progress=combinedDownloadProgress([{total:100,received:-10},{total:100,received:250},{total:100,received:NaN}]);
  expect(progress).toMatchObject({count:3,determinate:true});
  expect(progress?.percent).toBeCloseTo(100/3);
 });
 it('uses an indeterminate state if any total size is unknown',()=>{
  expect(combinedDownloadProgress([{total:100,received:50},{total:0,received:10}])).toEqual({count:2,determinate:false,percent:0});
 });
});
