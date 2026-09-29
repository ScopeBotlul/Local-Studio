import {useEffect,useState} from 'react';
import {invoke} from '@tauri-apps/api/core';
import type {Settings} from './types';
import './core-features.css';
interface LiveHardware{cpuPercent:number;usedMemoryBytes:number;totalMemoryBytes:number;gpus:{name:string;utilization:number|null;usedBytes:number|null;totalBytes:number|null;temperature:number|null}[]}
const gib=(n:number)=>`${(n/1024**3).toFixed(1)} GiB`;
export function CoreSettings({settings:s,de,disabled,onChange}:{settings:Settings;de:boolean;disabled:boolean;onChange:(patch:Partial<Settings>)=>void}){
 return <section className="panel settings-section"><h2>{de?'Hintergrund und Ressourcen':'Background and resources'}</h2>{([
 ['removeCensorTags','Zensur-Tags beim Tag-Import entfernen','Remove censorship tags during tag import'],['autoModelUpdates','Modellupdates automatisch prüfen','Automatically check model updates'],['systemAccent','Windows-Akzentfarbe verwenden','Use Windows accent color'],['liveHardware','Live-Hardwareanzeige','Live hardware monitor'],['minimizeToTray','Beim Schließen im Infobereich weiterlaufen','Keep running in the system tray on close'],['parallelGeneration','Parallele Verarbeitung bei ausreichendem RAM','Parallel processing when RAM is available']
 ] as const).map(([key,d,e])=><label className="setting-row" key={key}><span>{de?d:e}</span><input type="checkbox" checked={!!s[key]} disabled={disabled} onChange={v=>onChange({[key]:v.target.checked})}/></label>)}<p className="hub-hint">{de?'Standardmäßig läuft eine große Berechnung gleichzeitig. Parallel: höchstens zwei Berechnungen, davon eine auf der GPU. Wartende Aufträge bleiben abbrechbar. Beenden im Datei- oder Infobereich-Menü verwendet weiterhin den Speicherdialog.':'One expensive calculation runs at a time by default. Parallel: at most two calculations, with one on the GPU. Waiting jobs remain cancellable. Exit in the File or tray menu still uses the save dialog.'}</p></section>;
}
export default function CoreFeatures({settings,de}:{settings:Settings;de:boolean}){
 useEffect(()=>{if(settings.autoModelUpdates)void invoke('model_updates_check').catch(()=>{});},[settings.autoModelUpdates]);
 const [live,setLive]=useState<LiveHardware|null>(null);
 useEffect(()=>{if(!settings.liveHardware){setLive(null);return;}let alive=true;let timer:ReturnType<typeof setTimeout>;const poll=async()=>{try{const v=await invoke<LiveHardware>('hardware_live');if(alive)setLive(v);}catch{if(alive)setLive(null);}if(alive)timer=setTimeout(poll,2000);};void poll();return()=>{alive=false;clearTimeout(timer);};},[settings.liveHardware]);
 if(!live)return null;
 return <div className="global-resources" aria-label={de?'Aktivität und Hardware':'Activity and hardware'}>{live&&<details><summary>CPU {live.cpuPercent.toFixed(0)} % · RAM {gib(live.usedMemoryBytes)} / {gib(live.totalMemoryBytes)}</summary>{live.gpus.length?live.gpus.map(g=><div key={g.name}>{g.name} · GPU {g.utilization===null?'—':`${g.utilization.toFixed(0)} %`} · VRAM {g.usedBytes===null?'—':gib(g.usedBytes)} / {g.totalBytes===null?'—':gib(g.totalBytes)} · {g.temperature===null?'—':`${g.temperature.toFixed(0)} °C`}</div>):<p>{de?'Keine verlässlichen GPU-Livewerte verfügbar.':'No reliable live GPU readings available.'}</p>}</details>}</div>;
}
