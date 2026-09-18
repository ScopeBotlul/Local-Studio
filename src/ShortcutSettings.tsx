import { useState } from 'react';
import { chordFromEvent, defaultShortcuts, shortcutIssue, shortcutLabels, shortcutText, type ShortcutCommand, type Shortcuts } from './shortcuts';

export default function ShortcutSettings({ value, onChange, disabled, de }: { value: Shortcuts; onChange: (value: Shortcuts) => void; disabled: boolean; de: boolean }) {
  const [recording, setRecording] = useState<ShortcutCommand | null>(null);
  const [error, setError] = useState('');
  return <section className="panel settings-section shortcut-settings">
    <div className="settings-section-title"><div><h2>{de ? 'Tastenkürzel' : 'Keyboard shortcuts'}</h2><p>{de ? 'Ein Kürzel anklicken und die gewünschte Kombination drücken. Escape bricht ab. Die Belegung gilt nach dem Speichern.' : 'Click a shortcut and press the desired combination. Escape cancels. Bindings apply after saving.'}</p></div></div>
    {error && <p className="notice warning" role="alert">{error}</p>}
    {(Object.keys(defaultShortcuts) as ShortcutCommand[]).map(id => <div className="setting-row" key={id}><label htmlFor={`shortcut-${id}`}>{shortcutLabels[id][de ? 0 : 1]}</label><div className="shortcut-control">
      <button id={`shortcut-${id}`} type="button" className="button secondary shortcut-key" data-shortcut-recorder disabled={disabled} aria-pressed={recording === id} onBlur={() => setRecording(null)} onClick={() => { setError(''); setRecording(id); }} onKeyDown={e => {
        if (recording !== id) return;
        e.preventDefault(); e.stopPropagation();
        if (e.key === 'Escape') { setRecording(null); setError(''); return; }
        if (e.repeat) return;
        const chord = chordFromEvent(e.nativeEvent); if (!chord) return;
        const issue = shortcutIssue(chord, value, id);
        if (issue) { setError(issue === 'reserved' ? (de ? 'Diese Kombination ist für Windows, Textbearbeitung oder Projekte reserviert.' : 'This combination is reserved for Windows, text editing or projects.') : (de ? 'Diese Kombination wird bereits von einer anderen Aktion verwendet.' : 'This combination is already used by another action.')); return; }
        onChange({ ...value, [id]: chord }); setRecording(null); setError('');
      }}>{recording === id ? (de ? 'Tasten drücken …' : 'Press keys …') : shortcutText(value[id], de) || (de ? 'Nicht belegt' : 'Unassigned')}</button>
      <button className="text-button" type="button" disabled={disabled || !value[id]} aria-label={`${de ? 'Kürzel entfernen' : 'Unbind shortcut'}: ${shortcutLabels[id][de ? 0 : 1]}`} onClick={() => { onChange({ ...value, [id]: '' }); setError(''); }}>{de ? 'Entfernen' : 'Unbind'}</button>
    </div></div>)}
    <p className="hub-hint">{de ? 'Galerieaktionen gelten in der Galerie; Bildkürzel im fokussierten Bildbereich. Textfelder behalten ihre normalen Bearbeitungstasten. Strg + Mausrad ändert weiterhin die Oberflächengröße.' : 'Gallery actions apply in the gallery; image shortcuts in the focused viewport. Text fields retain their normal editing keys. Ctrl + mouse wheel still changes the interface size.'}</p>
    <button type="button" className="button secondary" disabled={disabled} onClick={() => { onChange({ ...defaultShortcuts }); setRecording(null); setError(''); }}>{de ? 'Standardbelegung wiederherstellen' : 'Restore default shortcuts'}</button>
  </section>;
}
