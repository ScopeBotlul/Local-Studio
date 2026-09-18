import {invoke} from '@tauri-apps/api/core';
import type {LineageQuery} from './gallery-api';
export interface Corrections {brightness:number;contrast:number;saturation:number;temperature:number}
export const neutralCorrections:Corrections={brightness:0,contrast:0,saturation:0,temperature:0};
export type EditOperation={type:'rotate';clockwise:boolean}|{type:'flip';horizontal:boolean}|{type:'crop';x:number;y:number;width:number;height:number}|{type:'resize';width:number;height:number}|({type:'adjust'}&Corrections);
export interface ProjectEditQuery {id:string;assetId:string;sha256:string}
export interface EditHistory {operations:EditOperation[];cursor:number;floor:number}
export const emptyHistory:EditHistory={operations:[],cursor:0,floor:0};
export function appendEdit(h:EditHistory,op:EditOperation,limit:number):EditHistory{const operations=[...h.operations.slice(0,h.cursor),op];return {operations,cursor:operations.length,floor:Math.max(h.floor,operations.length-Math.max(1,limit))};}
export function undoEdit(h:EditHistory):EditHistory{return {...h,cursor:Math.max(h.floor,h.cursor-1)};}
export function redoEdit(h:EditHistory):EditHistory{return {...h,cursor:Math.min(h.operations.length,h.cursor+1)};}
export interface EditPreview {dataUrl:string;width:number;height:number;profile:boolean}
export const editorApi={preview:(query:LineageQuery,operations:EditOperation[])=>invoke<EditPreview>('editor_preview',{request:{query,operations}}),export:(query:LineageQuery,operations:EditOperation[],format:'png'|'jpeg',quality:number)=>invoke<string>('editor_export',{request:{query,operations},format,quality}),projectPreview:(query:ProjectEditQuery,operations:EditOperation[])=>invoke<EditPreview>('project_editor_preview',{query,operations}),projectExport:(query:ProjectEditQuery,operations:EditOperation[],format:'png'|'jpeg',quality:number)=>invoke<string>('project_editor_export',{query,operations,format,quality})};
// Synchronous state lets the consolidated app-close flow wait for an in-flight export.
export const editorActivity:{pending:Promise<unknown>|null;draft:boolean;persisted:boolean;cancelBatch:(()=>void)|null;flushCreative:(()=>Promise<void>)|null}={pending:null,draft:false,persisted:true,cancelBatch:null,flushCreative:null};
