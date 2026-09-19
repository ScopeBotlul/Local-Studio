import type {UpdateStatus} from './update-api';

// One delayed check per app start. Never interrupt an existing modal dialog.
export function scheduleStartupUpdate(options: {
  status: () => Promise<UpdateStatus>;
  check: () => Promise<UpdateStatus>;
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
    void options.status()
      .then(status => live && status.phase === 'idle' ? options.check() : status)
      .then(status => { if (live && status.phase === 'available' && status.latest) show(); })
      .catch(() => {}); // Offline startup stays quiet; manual checks report errors.
  }, 15000);
  return () => { live = false; clearTimeout(timer); };
}
