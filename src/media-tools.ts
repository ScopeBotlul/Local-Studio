import {invoke} from '@tauri-apps/api/core';
import type {Timeline} from './creative-state';
export interface Waveform {duration:number;peaks:number[]}
export interface MediaInfo {assetId:string;proxies:{id:string;width:number;height:number;bytes:number}[];waveform:Waveform|null}
export interface MediaTask {id:string;assetId:string;name:string;kind:'proxy'|'wave';status:string;progress:number;error:string|null}
export interface Frame {dataUrl:string;time:number;proxyAssets:string[]}
export const mediaApi={
 frame:(id:string,timeline:Timeline,time:number,proxies:boolean)=>invoke<Frame>('video_frame',{id,timeline,time,proxies}),
 info:(id:string)=>invoke<MediaInfo[]>('media_info',{id}),
 prepare:(id:string,assetId:string,kind:'proxy'|'wave',resolution='auto',width=960)=>invoke<MediaTask>('media_prepare',{id,assetId,kind,resolution,width}),
 status:()=>invoke<MediaTask|null>('media_status'),cancel:(id:string)=>invoke<void>('media_cancel',{id})
};
