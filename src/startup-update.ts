import type {UpdateStatus} from './update-api';
import type {ComfyStatus} from './comfy-api';

// One delayed check per app start. Never interrupt an existing modal dialog.
export function scheduleStartupUpdate(options: {
  status: () => Promise<UpdateStatus>;
  check: () => Promise<UpdateStatus>;
  comfyStatus: () => Promise<ComfyStatus>;
  comfyCheck: () => Promise<ComfyStatus>;
  onStarted: () => void;
  canShow: () => boolean;
  onAvailable: () => void;
}) {
  let live = true;
  let timer: ReturnType<typeof setTimeout>;
  const show = () => {
    if (!live) return;
    if (options.canShow()) options.onAvailable();
    else timer = setTimeout(show, 500);
  };
  timer = setTimeout(() => {
    options.onStarted();
    const local=options.status().then(status => live&&status.phase==='idle'?options.check():status);
    const comfy=options.comfyStatus().then(status => live&&status.installed&&status.update.phase==='idle'?options.comfyCheck():status);
    void Promise.allSettled([local,comfy]).then(results=>{
      if(!live)return;
      const localAvailable=results[0].status==='fulfilled'&&results[0].value.phase==='available'&&!!results[0].value.latest;
      const comfyAvailable=results[1].status==='fulfilled'&&results[1].value.update.phase==='available';
      if(localAvailable||comfyAvailable)show();
    }); // Offline startup stays quiet; manual checks report errors in the update center.
  }, 15000);
  return () => { live = false; clearTimeout(timer); };
}
