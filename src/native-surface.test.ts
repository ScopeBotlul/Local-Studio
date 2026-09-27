import {describe,expect,it} from 'vitest';
import {clipNativeSurface} from './native-surface';

const content={left:0,top:36,right:1440,bottom:940},viewport={width:1440,height:940};

describe('native child surface clipping',()=>{
 it('clips a scrolled website below the title bar',()=>{
  expect(clipNativeSurface({left:200,top:-120,right:1200,bottom:500},content,viewport)).toEqual({x:200,y:36,width:1000,height:464});
 });
 it('hides a surface with no usable content-area intersection',()=>{
  expect(clipNativeSurface({left:200,top:-500,right:1200,bottom:20},content,viewport)).toBeNull();
 });
 it('clips all four edges to the app content and viewport',()=>{
  expect(clipNativeSurface({left:-50,top:20,right:1500,bottom:1000},content,viewport)).toEqual({x:0,y:36,width:1440,height:904});
 });
});
