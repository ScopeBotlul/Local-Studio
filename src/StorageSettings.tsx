import type { Settings, StoragePaths } from './types';

const folders: [keyof StoragePaths, string, string, string][] = [
  ['models', 'Modelle', 'Models', 'models'], ['assistantModels', 'Assistentenmodelle', 'Assistant models', 'models\\assistant'],
  ['visionModels', 'Visionmodelle', 'Vision models', 'models\\vision'], ['downloads', 'Downloads', 'Downloads', 'downloads'],
  ['gallery', 'Galerie', 'Gallery', 'gallery'], ['projects', 'Projekte', 'Projects', 'projects'],
  ['temporary', 'Temporäre Dateien', 'Temporary files', 'temporary'], ['recovery', 'Recovery', 'Recovery', 'recovery'],
  ['cache', 'Cache', 'Cache', 'cache'], ['proxies', 'Proxydateien', 'Proxy files', 'proxies'],
];
export default function StorageSettings({ settings, de, disabled, onChange, chooseFolder }: {
  settings: Settings; de: boolean; disabled: boolean; onChange: (value: Settings['storageOverrides']) => void;
  chooseFolder: (path: string, onChoose: (path: string) => void) => Promise<void>;
}) {
  function update(key: keyof StoragePaths, value: string) {
    const next = { ...settings.storageOverrides }; if (value) next[key] = value; else delete next[key]; onChange(next);
  }
  return <details className="storage-paths storage-editor"><summary>{de ? 'Einzelne Speicherorte anpassen' : 'Customize individual storage folders'}</summary>
    <p className="hub-hint">{de ? 'Leer bedeutet Standard unter dem Datenordner. Änderungen verschieben oder löschen keine vorhandenen Dateien. Laufende Downloads und Bildaufträge behalten ihren bisherigen Speicherort. Projektarbeitskopien liegen unter Recovery. Die Sitzungsdatenbank bleibt im Konfigurationsordner; Proxies sind für spätere Medienabläufe vorgesehen.' : 'Empty means the default under the data folder. Changes do not move or delete existing files. Active downloads and image jobs retain their previous location. Project working copies use the recovery folder. The session database stays in the configuration folder; proxies are reserved for future media workflows.'}</p>
    {folders.map(([key, german, english, suffix]) => {
      const fallback = settings.dataRoot.replace(/[\\/]+$/, '') + '\\' + suffix;
      const value = settings.storageOverrides[key] ?? '';
      return <div className="storage-field" key={key}><label htmlFor={`storage-${key}`}>{de ? german : english}</label><div className="path-field"><input id={`storage-${key}`} spellCheck={false} disabled={disabled} value={value} placeholder={fallback} onChange={e => update(key, e.target.value)} /><button type="button" className="button secondary" disabled={disabled} onClick={() => void chooseFolder(value || fallback, path => update(key, path))}>{de ? 'Wählen' : 'Browse'}</button><button className="text-button" type="button" disabled={disabled || !value} onClick={() => update(key, '')}>{de ? 'Standard' : 'Default'}</button></div></div>;
    })}
  </details>;
}
