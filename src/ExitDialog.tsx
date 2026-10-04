import { useEffect, useRef, useState } from 'react';
import type { Language } from './types';
export type ExitChoice = 'save' | 'discard' | 'close' | 'cancel' | 'keep';
export interface ExitPrompt { warnings: string[]; unsaved: number; gifUnsaved: number; imagesRunning: boolean; restoreSession: boolean; project?: boolean; projectDirty?: boolean; resolve: (choice: ExitChoice, saveProject?: boolean) => void; }
export default function ExitDialog({ prompt, language, onChoose: choose }: { prompt: ExitPrompt; language: Language; onChoose: (choice: ExitChoice, saveProject?: boolean) => void }) {
  const dialog = useRef<HTMLDialogElement>(null); const de = language === 'de';
  useEffect(() => { const element = dialog.current; element?.showModal(); return () => element?.close(); }, []);
  const [saveProject, setSaveProject] = useState(!!prompt.projectDirty);
  const onChoose = (choice: ExitChoice) => choose(choice, saveProject);
  const images = prompt.unsaved > 0 || prompt.imagesRunning;
  const media = images || prompt.gifUnsaved > 0;
  return <dialog ref={dialog} className="image-exit-dialog" aria-labelledby="image-exit-title" onCancel={e => { e.preventDefault(); onChoose('cancel'); }}>
    <h2 id="image-exit-title">{de ? 'Local Studio beenden?' : 'Close Local Studio?'}</h2>
    {prompt.unsaved > 0 && <p>{de ? `${prompt.unsaved} ungespeicherte Bilder sind noch vorhanden.` : `${prompt.unsaved} images have not been saved.`}</p>}
    {prompt.gifUnsaved > 0 && <p>{de ? `${prompt.gifUnsaved} ungespeicherte GIFs / Videos sind noch vorhanden.` : `${prompt.gifUnsaved} GIFs / videos have not been saved.`}</p>}
    {prompt.warnings.map(warning => <p key={warning}>{warning}</p>)}
    {images && <p>{de ? 'Speichern legt alle fertigen Bilder in der Galerie ab. Verwerfen löscht ihre temporären Bilddateien. Bereits gespeicherte Galeriebilder bleiben erhalten.' : 'Save puts all completed images in the gallery. Discard deletes their temporary image files. Previously saved gallery images are kept.'}</p>}
    {prompt.gifUnsaved > 0 && <p>{de?'Speichern legt fertige GIFs und Videos im Galerie-Hauptordner ab. Verwerfen löscht ihre temporären Ergebnisse.':'Save puts completed GIFs and videos in the gallery root. Discard removes their temporary results.'}</p>}
    {prompt.project && <label className="project-exit-option"><input type="checkbox" checked={saveProject} onChange={e => setSaveProject(e.target.checked)} />{de ? "Projektdatei vor dem Beenden speichern" : "Save project file before closing"}</label>}
    <div className="image-dialog-actions">
      <button autoFocus className="button secondary" onClick={() => onChoose('cancel')}>{de ? 'Abbrechen' : 'Cancel'}</button>
      {media && prompt.restoreSession && <button className="button secondary" onClick={() => onChoose('keep')}>{de ? 'Arbeitsstand behalten und beenden' : 'Keep workspace and close'}</button>}
      {media ? <><button className="button secondary" onClick={() => onChoose('discard')}>{de ? 'Medien verwerfen und beenden' : 'Discard media and close'}</button><button className="button primary" onClick={() => onChoose('save')}>{de ? 'Medien speichern und beenden' : 'Save media and close'}</button></> : <button className="button primary" onClick={() => onChoose('close')}>{de ? 'Beenden' : 'Close'}</button>}
    </div>
  </dialog>;
}
