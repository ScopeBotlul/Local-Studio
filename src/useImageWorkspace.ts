import { useCallback, useEffect, useRef, useState, type SetStateAction } from 'react';
import { imageApi, type ImageRequest, type ImageWorkspace } from './image-api';
import {usePrivacy,useModelPrivacy} from './Privacy';

export const defaultImageRequest: ImageRequest = { modelPath: '', prompt: '', negativePrompt: '', width: 512, height: 512, steps: 25, guidance: 5, seed: 42, sampler: 'euler' };
export function useImageWorkspace(enabled: boolean) {
  const [request, renderRequest] = useState(defaultImageRequest);
  const [ready, setReady] = useState(false);
  const [locked,setLocked]=useState(false);
  const privacy=usePrivacy(),modelRestricted=useModelPrivacy(request.modelPath);
  const mayWrite=useRef(true);
  mayWrite.current=!privacy.status.locked||(!locked&&!modelRestricted);
  const [recovery, setRecovery] = useState(false);
  const [error, setError] = useState('');
  const value = useRef<ImageWorkspace>({ request: null, models: {} });
  const canSave = useRef(false);
  const queue = useRef<Promise<unknown>>(Promise.resolve());
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const flush = useCallback(async () => {
    if (timer.current) { clearTimeout(timer.current); timer.current = null; }
    if (!canSave.current||!mayWrite.current) return;
    const workspace = structuredClone(value.current);
    if (workspace.request?.modelPath.toLowerCase().endsWith('.safetensors')) workspace.models[workspace.request.modelPath] = { ...workspace.request, prompt: '', negativePrompt: '' };
    const paths = Object.keys(workspace.models);
    for (const path of paths.slice(0, Math.max(0, paths.length - 256))) delete workspace.models[path];
    value.current.models = workspace.models;
    const write = queue.current.catch(() => {}).then(() => imageApi.saveWorkspace(workspace));
    queue.current = write; await write;
  }, []);
  useEffect(() => {
    if (!enabled) return;
    let live = true;
    void imageApi.workspace().then(data => {
      if (!live) return;
      value.current = data.workspace; canSave.current = !data.recoveryAvailable;setLocked(!!data.locked);
      setRecovery(data.recoveryAvailable); renderRequest(data.recoveryAvailable ? defaultImageRequest : data.workspace.request ?? defaultImageRequest); setReady(true);
    }).catch(e => { if (live) setError(String(e)); });
    return () => { live = false; if (timer.current) clearTimeout(timer.current); };
  }, [enabled]);
  const setRequest = useCallback((action: SetStateAction<ImageRequest>) => {
    if (!canSave.current) return;
    const previous = value.current.request ?? defaultImageRequest;
    const next = typeof action === 'function' ? action(previous) : action;
    if(next.modelPath!==previous.modelPath)setLocked(false);
    value.current = { ...value.current, request: next };
    renderRequest(next);
    if (timer.current) clearTimeout(timer.current);
    timer.current = setTimeout(() => void flush().then(() => setError('')).catch(e => setError(String(e))), 450);
  }, [flush]);
  const selectModel = useCallback((modelPath: string) => {
    if (!canSave.current) return;
    const current = value.current.request ?? defaultImageRequest;
    if (modelPath === current.modelPath) return;
    if (current.modelPath) value.current.models[current.modelPath] = { ...current, prompt: '', negativePrompt: '' };
    const preferences = value.current.models[modelPath] ?? defaultImageRequest;
    setRequest({ ...preferences, modelPath, prompt: current.prompt, negativePrompt: current.negativePrompt, reference: current.reference ?? null, ...(current.reference ? {width:current.reference.width,height:current.reference.height} : {}) });
  }, [setRequest]);
  const restore = useCallback((next: ImageRequest) => {
    selectModel(next.modelPath); setRequest({ ...next });
  }, [selectModel, setRequest]);
  const recover = useCallback(async () => {
    const data = await imageApi.recover(); value.current = data.workspace;
    canSave.current = true;setLocked(!!data.locked); setRecovery(false); renderRequest(data.workspace.request ?? defaultImageRequest);
  }, []);
  return { locked,request, setRequest, selectModel, restore, ready, recovery, recover, flush, error };
}
