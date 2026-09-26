import {invoke} from '@tauri-apps/api/core';
import type {DownloadPlan} from './download-api';
export type CivitaiModel={id:number;name:string;kind:string;nsfw:boolean;creator:string;downloads:number;rating:number|null;tags:string[];latestVersion:string|null;baseModel:string|null;creditRequired:boolean;commercialUse:string;derivativesAllowed:boolean;differentLicenseAllowed:boolean};
export type CivitaiFile={id:number;name:string;sizeBytes:number;sha256:string|null;format:string|null;pickleScan:string|null;virusScan:string|null;primary:boolean;downloadUrl:string|null};
export type CivitaiVersion={id:number;name:string;baseModel:string|null;publishedAt:string|null;trainedWords:string[];files:CivitaiFile[]};
export type CivitaiDetail={model:CivitaiModel;versions:CivitaiVersion[]};
export type CivitaiSearch={models:CivitaiModel[];nextCursor:string|null};
export const civitaiDirectDownloadEligible=(file:CivitaiFile)=>file.primary&&!!file.sha256&&file.format?.toLowerCase()==='safetensor'&&file.virusScan?.toLowerCase()==='success'&&file.pickleScan?.toLowerCase()==='success';
export const civitaiApi={
 search:(query:string,kind:string,baseModel:string,sort:string,period:string,cursor:string,includeNsfw:boolean)=>invoke<CivitaiSearch>('civitai_model_search',{query,kind,baseModel,sort,period,cursor,includeNsfw}),
 detail:(id:number)=>invoke<CivitaiDetail>('civitai_model_detail',{id}),
 plan:(modelId:number,versionId:number,fileId:number)=>invoke<DownloadPlan>('civitai_download_plan',{modelId,versionId,fileId}),
};
