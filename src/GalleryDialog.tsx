import { useEffect,useRef,type ReactNode } from 'react';
export default function GalleryDialog({title,busy,onClose,children}:{title:string;busy:boolean;onClose:()=>void;children:ReactNode}){
  const ref=useRef<HTMLDialogElement>(null);
  useEffect(()=>{const dialog=ref.current;dialog?.showModal();return()=>dialog?.close();},[]);
  return <dialog ref={ref} className="gallery-dialog" aria-label={title} onCancel={e=>{e.preventDefault();if(!busy)onClose();}}><h2>{title}</h2>{children}</dialog>;
}
