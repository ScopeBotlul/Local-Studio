import {describe,it,expect} from 'vitest';
import {downloadEta} from './DownloadOverview';
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
