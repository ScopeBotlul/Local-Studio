export const defaultShortcuts = {
  projectNew:'Ctrl+N',projectOpen:'Ctrl+O',projectSaveAs:'Ctrl+Shift+S',projectClose:'Ctrl+W',
  undo:'Ctrl+Z', redo:'Ctrl+Y',
  projectSave: 'Ctrl+S', generate: 'Ctrl+Enter', delete: 'Delete', rename: 'F2', selectAll: 'Ctrl+A', compare: 'C',
  imageFit: 'F', imageActual: '1', imageReset: '0', uiZoomIn: 'Ctrl+Plus', uiZoomOut: 'Ctrl+Minus', uiZoomReset: 'Ctrl+0',
};
export type ShortcutCommand = keyof typeof defaultShortcuts;
export type Shortcuts = Record<ShortcutCommand, string>;
export const shortcutLabels: Record<ShortcutCommand, [string, string]> = {
  projectNew:['Neues Projekt','New project'],projectOpen:['Projekt öffnen','Open project'],projectSaveAs:['Projekt speichern unter','Save project as'],projectClose:['Projekt schließen','Close project'],
  undo:['Bearbeitung rückgängig','Undo edit'],redo:['Bearbeitung wiederholen','Redo edit'],
  projectSave: ['Projekt speichern', 'Save project'],
  generate: ['Bild generieren / einreihen', 'Generate / queue image'], delete: ['In Papierkorb verschieben', 'Move to trash'],
  rename: ['Umbenennen', 'Rename'], selectAll: ['Galerieseite auswählen', 'Select gallery page'], compare: ['Zwei Bilder vergleichen', 'Compare two images'],
  imageFit: ['Bild einpassen', 'Fit image'], imageActual: ['Bild in 100 % anzeigen', 'Actual image size'], imageReset: ['Bildansicht zurücksetzen', 'Reset image view'],
  uiZoomIn: ['Oberfläche vergrößern', 'Increase interface size'], uiZoomOut: ['Oberfläche verkleinern', 'Decrease interface size'], uiZoomReset: ['Oberfläche auf 100 %', 'Reset interface size'],
};
const reserved = new Set(['Alt+F4', 'Ctrl+C', 'Ctrl+V', 'Ctrl+X', 'Ctrl+Shift+Z', 'Ctrl+Alt+Delete']);
interface KeyEvent { key: string; ctrlKey: boolean; altKey: boolean; shiftKey: boolean; metaKey: boolean; isComposing?: boolean; getModifierState?: (key: string) => boolean }
export function chordFromEvent(event: KeyEvent): string | null {
  if (event.metaKey || event.isComposing || event.key === 'Process' || event.getModifierState?.('AltGraph')) return null;
  let key = event.key;
  if (key === '+' || key === '=') key = 'Plus';
  else if (key === '-') key = 'Minus';
  else if (/^[a-z0-9]$/i.test(key)) key = key.toUpperCase();
  else if (!/^(Enter|Delete|Insert|Home|End|PageUp|PageDown|F[1-9]|F1[0-2])$/.test(key)) return null;
  return [event.ctrlKey && 'Ctrl', event.altKey && 'Alt', event.shiftKey && key !== 'Plus' && 'Shift', key].filter(Boolean).join('+');
}
export function shortcutIssue(chord: string, bindings: Shortcuts, command: ShortcutCommand): 'reserved' | 'conflict' | null {
  if (!chord) return null;
  if (reserved.has(chord)) return 'reserved';
  return Object.entries(bindings).some(([id, value]) => id !== command && value === chord) ? 'conflict' : null;
}
export function shortcutFor(event: KeyEvent, bindings: Shortcuts): ShortcutCommand | undefined {
  const chord = chordFromEvent(event);
  return chord ? (Object.keys(defaultShortcuts) as ShortcutCommand[]).find(key => bindings[key] === chord) : undefined;
}
export function shortcutText(chord: string, de: boolean) { return chord.replace('Ctrl', de ? 'Strg' : 'Ctrl').replace('Shift', de ? 'Umschalt' : 'Shift').replace('Plus', '+').replace('Minus', '−'); }
