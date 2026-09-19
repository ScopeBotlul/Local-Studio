import type {LocalModel} from './LocalModels';
export interface ModelProfile {
  purpose:'image'|'video'|'language'|'audio'|'unknown';
  role:'model'|'extension'|'component'|'unknown';
  family:string|null; baseFamily?:string|null; packaging:string; evidence:string; support:string;
  requirements:{name:string;embedded:boolean}[];
}
export const modelCategories=['models','image','video','language','audio','extension','component','unknown','excluded'] as const;
export type ModelCategory=typeof modelCategories[number];
const categoryLabels:Record<ModelCategory,[string,string]>={
  models:['Alle Modelle','All models'],image:['Bild','Image'],video:['Video','Video'],language:['Sprache','Language'],audio:['Audio & Musik','Audio & music'],
  extension:['Erweiterungen','Extensions'],component:['Technische Komponenten','Technical components'],unknown:['Nicht eindeutig erkannt','Not clearly identified'],excluded:['Ausgeblendete Treffer','Hidden findings'],
};
export const categoryLabel=(category:ModelCategory,de:boolean)=>categoryLabels[category][de?0:1];
export function modelCategory(model:LocalModel):ModelCategory {
  if(model.discovery==='excluded')return 'excluded';
  const profile=model.profile;
  if(!profile || profile.role==='unknown')return 'unknown';
  if(profile.role==='extension'||profile.role==='component')return profile.role;
  return profile.purpose;
}
export function inModelCategory(model:LocalModel,category:ModelCategory) {
  const actual=modelCategory(model);
  return category==='models'?['image','video','language','audio'].includes(actual):actual===category;
}
export function modelSupport(model:LocalModel,de:boolean):string {
  const labels:Record<string,[string,string]>={
    missing:['Komponenten fehlen','Components missing'],preflight:['SDXL unterstützt · Vorprüfung im Studio','SDXL supported · preflight in Studio'],
    unsupported:['Noch nicht zur Ausführung unterstützt','Execution not supported yet'],dependency:['Benötigt ein passendes Basismodell','Requires a compatible base model'],
    unknown:['Unterstützung noch unbekannt','Support unknown'],recheck:['Dateistruktur erneut prüfen','Recheck file structure'],
  };
  if(model.status==='missing'||model.status==='unavailable')return de?'Dateien nicht verfügbar':'Files unavailable';
  if(model.status==='invalid')return de?'Dateistruktur fehlerhaft':'Invalid file structure';
  if(model.status==='incomplete')return de?'Gewichtsdateien fehlen':'Weight files missing';
  if(model.status==='changed')return de?'Dateien verändert · erneut prüfen':'Files changed · recheck';
  if(model.profile?.purpose==='language'&&model.format==='gguf'&&model.profile.role==='model')return de?'Im Assistenten separat importieren und prüfen':'Import and check separately in Assistant';
  return labels[model.profile?.support??'unknown']?.[de?0:1]??(de?'Unterstützung noch unbekannt':'Support unknown');
}
export function ModelClassification({model,de}:{model:LocalModel;de:boolean}) {
  const profile=model.profile;
  const packing:Record<string,[string,string]>={checkpoint:['Vollständiger Checkpoint','Complete checkpoint'],backbone:['Modell mit zusätzlichen Abhängigkeiten','Model with additional dependencies'],sharded:['Auf mehrere Dateien verteilt','Split across multiple files']};
  const evidence:Record<string,[string,string]>={structure:['Dateistruktur erkannt','File structure identified'],metadata:['Zuordnung laut Metadaten','Classification from metadata'],location:['Vorläufige Zuordnung','Tentative classification'],unknown:['Nicht eindeutig erkannt','Not clearly identified']};
  return <div className="model-classification">
    <div className="hub-tags"><span>{categoryLabel(modelCategory(model),de)}</span>{profile?.family&&<span>{profile.family}</span>}{profile&&packing[profile.packaging]&&<span>{packing[profile.packaging][de?0:1]}</span>}</div>
    <p className={'model-support '+(profile?.support==='preflight'?'supported':'')}>{modelSupport(model,de)}</p>
    {profile?.support==='missing'&&!!profile.requirements.length&&<p className="hub-hint">{de?'Nicht im Checkpoint enthalten: ':'Not included in this checkpoint: '}{profile.requirements.filter(r=>!r.embedded).map(r=>r.name).join(', ')}. {de?'Der aktuelle Bildadapter benötigt einen vollständigen SDXL-Checkpoint. Lose Komponenten werden noch nicht zu einer ausführbaren Pipeline verbunden.':'The current image adapter requires a complete SDXL checkpoint. Separate components cannot yet be assembled into an executable pipeline.'}</p>}
    {profile?.role==='extension'&&<p className="hub-hint">{profile.baseFamily?(de?'Basisfamilie laut Metadaten: ':'Base family from metadata: ')+profile.baseFamily+'. ':(de?'Kompatible Basisfamilie noch unbekannt. ':'Compatible base family unknown. ')}{de?'Die aktive Bildgenerierung unterstützt Erweiterungen noch nicht.':'The current image generator does not support extensions yet.'}</p>}
    {profile?.role==='component'&&<p className="hub-hint">{de?'Technischer Bestandteil einer Pipeline, kein eigenständiger Generator.':'Technical part of a pipeline, not a standalone generator.'}</p>}
    <small>{evidence[profile?.evidence??'unknown']?.[de?0:1]}</small>
    {!!profile?.requirements.length&&<details className="model-requirements"><summary>{de?'Bestandteile und Zuordnung':'Components and assignment'}</summary><ul>{profile.requirements.map(r=><li key={r.name}><strong>{r.name}</strong><span>{r.embedded?(profile.support==='preflight'?(de?'Im Checkpoint enthalten · automatisch verwendet':'Included in checkpoint · used automatically'):(de?'Im Checkpoint enthalten':'Included in checkpoint')):(de?'Nicht im Checkpoint enthalten':'Not included in checkpoint')}</span></li>)}</ul></details>}
  </div>;
}
