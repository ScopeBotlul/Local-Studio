import { describe,it,expect } from 'vitest';
import { clampView,initialView,wipePosition } from './gallery-compare-view';
describe('linked image comparison viewport',()=>{
 it('keeps both image positions bounded when zooming back out',()=>{
   expect(clampView({zoom:3,x:9,y:-9})).toEqual({zoom:3,x:1,y:-1});
   expect(clampView({zoom:1,x:1,y:-1})).toEqual(initialView);
   expect(clampView({zoom:NaN,x:Infinity,y:NaN})).toEqual(initialView);
   expect(clampView({zoom:90,x:0,y:0}).zoom).toBe(8);
 });
 it('maps the divider to its actual viewport and supports both full-image endpoints',()=>{
   expect(wipePosition(600,100,1000)).toBe(50);expect(wipePosition(-100,100,1000)).toBe(0);expect(wipePosition(5000,100,1000)).toBe(100);expect(wipePosition(1,0,0)).toBe(50);
 });
});
