import {describe,it,expect} from 'vitest';
import {inModelCategory,modelCategory,modelSupport,type ModelProfile} from './ModelClassification';
import type {LocalModel} from './LocalModels';
import {displayPath} from './helpers';
import {validImageDimensions} from './image-api';
const model=(profile:Partial<ModelProfile>,extra:Partial<LocalModel>={}):LocalModel=>({id:'fixture',name:'fixture',path:'D:\\models\\file.safetensors',sourceRoot:'D:\\models',format:'safetensors',kind:'candidate',discovery:'model',discoveryReason:'',family:null,totalBytes:10,status:'checked',completeness:'container',files:[],checkedAt:'',profile:{purpose:'unknown',role:'unknown',family:null,packaging:'unknown',evidence:'unknown',support:'unknown',requirements:[],...profile},...extra});
describe('model library categories',()=>{
 it('keeps extensions, components and unknown files out of main models',()=>{for(const role of ['component','extension','unknown'] as const){const m=model({role,purpose:'image'});expect(inModelCategory(m,'models')).toBe(false);expect(modelCategory(m)).toBe(role);}});
 it('groups by purpose and keeps excluded files separate',()=>{const m=model({role:'model',purpose:'video'});expect(inModelCategory(m,'models')).toBe(true);expect(modelCategory(m)).toBe('video');expect(modelCategory({...m,discovery:'excluded'})).toBe('excluded');});
 it('does not describe unavailable models as ready',()=>{expect(modelSupport(model({support:'preflight'},{status:'missing'}),true)).toBe('Dateien nicht verfügbar');});
});
describe('custom image dimensions and path display',()=>{
 it('accepts useful portrait and landscape sizes with matching area limit',()=>{for(const [w,h]of [[640,960],[768,1152],[1344,768],[2048,1024]])expect(validImageDimensions(w,h)).toBe(true);});
 it('rejects excessive, fractional, empty or misaligned dimensions',()=>{for(const [w,h]of [[2048,2048],[4096,512],[0,512],[641,960],[512.5,512],[NaN,512]])expect(validImageDimensions(w,h)).toBe(false);});
 it('removes only Windows extended-path display prefixes',()=>{expect(displayPath('\\\\?\\D:\\models\\a')).toBe('D:\\models\\a');expect(displayPath('\\\\?\\UNC\\server\\share')).toBe('\\\\server\\share');expect(displayPath('D:\\models\\a')).toBe('D:\\models\\a');});
});
