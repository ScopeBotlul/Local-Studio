import {useState} from 'react';
import {FolderOpen,FolderPlus,ArrowRight} from 'lucide-react';
import type {useProject} from './useProject';
import {displayPath} from './helpers';
import './home-projects.css';

export default function HomeProjects({controller:c,de,disabled,workspaceRecovery,onOpened}:{controller:ReturnType<typeof useProject>;de:boolean;disabled:boolean;workspaceRecovery:boolean;onOpened:()=>void}){
 const [selected,setSelected]=useState<string|null>(null);
 const [creating,setCreating]=useState(false),[name,setName]=useState('');
 const current=c.project;
 const rows=[...(current?[{key:`active:${current.id}`,name:current.name,path:current.path,available:true,active:true,openedAt:0}]:[]),...c.recent.filter(p=>p.path!==current?.path).map(p=>({...p,key:p.path,active:false}))];
 const chosen=rows.find(row=>row.key===selected)??rows.find(row=>row.available);
 const locked=disabled||c.busy||!c.ready;
 const cannotCreate=locked||workspaceRecovery||!!current?.recovery||!!current?.locked;
 async function openSelected(){if(!chosen||!chosen.available||locked)return;if(chosen.active){onOpened();return;}if(chosen.path&&await c.openPath(chosen.path))onOpened();}
 return <section className="panel home-projects" aria-labelledby="home-projects-title">
  <div className="section-heading"><div><h2 id="home-projects-title">{de?'Projekte':'Projects'}</h2><p className="hub-hint">{de?'Aktuelles Projekt und zuletzt geöffnete Projektdateien.':'Current project and recently opened project files.'}</p></div><FolderOpen size={23}/></div>
  <div className="home-project-actions">
   <button type="button" className="button secondary" disabled={cannotCreate} onClick={()=>{setCreating(true);setName('');}}><FolderPlus size={16}/>{de?'Neues Projekt':'New project'}</button>
   <button type="button" className="button secondary" disabled={locked} onClick={()=>void c.open().then(ok=>{if(ok)onOpened();})}><FolderOpen size={16}/>{de?'Durchsuchen …':'Browse …'}</button>
  </div>
  {creating&&<form className="home-project-create" onSubmit={e=>{e.preventDefault();if(!cannotCreate&&name.trim())void c.create(name.trim()).then(ok=>{if(ok){setCreating(false);onOpened();}});}}>
   <label className="project-name-field">{de?'Projektname':'Project name'}<input autoFocus maxLength={100} value={name} disabled={cannotCreate} onChange={e=>setName(e.target.value)}/></label>
   <p className="hub-hint">{de?'Die aktuellen Studio-Eingaben werden übernommen. Mit Datei → Speichern legst du später die Projektdatei ab.':'Current Studio inputs are included. Use File → Save to save the project file later.'}</p>
   <div className="home-project-actions"><button type="button" className="button secondary" disabled={c.busy} onClick={()=>setCreating(false)}>{de?'Abbrechen':'Cancel'}</button><button className="button primary" disabled={cannotCreate||!name.trim()}>{de?'Projekt erstellen':'Create project'}</button></div>
  </form>}
  <div className="home-project-list" role="radiogroup" aria-label={de?'Projekt auswählen':'Select project'}>
   {rows.map(row=><label key={row.key} className={`home-project-row ${chosen?.key===row.key?'selected':''} ${!row.available?'unavailable':''}`}>
    <input type="radio" name="home-project" checked={chosen?.key===row.key} disabled={locked||!row.available} onChange={()=>setSelected(row.key)}/>
    <span className="home-project-copy"><strong>{row.name}{row.active&&<small>{de?'Geöffnet':'Open'}</small>}</strong><span title={row.path?displayPath(row.path):undefined}>{row.path?displayPath(row.path):(de?'Noch nicht als Projektdatei gespeichert':'Not yet saved as a project file')}</span>{!row.available&&<span>{de?'Datei nicht verfügbar – bitte über Durchsuchen erneut wählen.':'File unavailable — locate it using Browse.'}</span>}</span>
   </label>)}
   {!rows.length&&<p className="home-project-empty">{!c.ready?(de?'Projekte werden geladen …':'Loading projects …'):(de?'Noch keine Projekte. Erstelle ein neues Projekt oder öffne eine .localstudio-Datei.':'No projects yet. Create a project or open a .localstudio file.')}</p>}
  </div>
  <div className="home-project-footer"><button type="button" className="button primary" disabled={locked||!chosen?.available} onClick={()=>void openSelected()}>{de?'Ausgewähltes Projekt öffnen':'Open selected project'}<ArrowRight size={16}/></button></div>
 </section>;
}
