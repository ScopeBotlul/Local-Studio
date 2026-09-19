export const hubCategories=['all','image','video','audio','speech','text','analysis'] as const;
export type HubCategory=typeof hubCategories[number];
export const hubCategoryLabels:Record<HubCategory,[string,string]>={
 all:['Alle','All'],image:['Bild','Image'],video:['Video','Video'],audio:['Audio & Musik','Audio & music'],speech:['Sprache','Speech'],text:['Text & Chat','Text & chat'],analysis:['Analyse','Analysis'],
};
// These are Hub search tags, not promises of local model compatibility.
export const hubTasks=[
 {id:'text-to-image',category:'image',label:['Bild aus Text','Text to image']},
 {id:'image-to-image',category:'image',label:['Bild zu Bild','Image to image']},
 {id:'image-text-to-image',category:'image',label:['Bild mit Text bearbeiten','Image and text to image']},
 {id:'mask-generation',category:'image',label:['Masken erzeugen','Mask generation']},
 {id:'text-to-video',category:'video',label:['Video aus Text','Text to video']},
 {id:'image-to-video',category:'video',label:['Video aus Bild','Image to video']},
 {id:'image-text-to-video',category:'video',label:['Video aus Bild und Text','Image and text to video']},
 {id:'video-to-video',category:'video',label:['Video zu Video','Video to video']},
 {id:'text-to-audio',category:'audio',label:['Audio / Musik aus Text','Audio / music from text']},
 {id:'audio-to-audio',category:'audio',label:['Audio bearbeiten / trennen','Audio processing / separation']},
 {id:'automatic-speech-recognition',category:'speech',label:['Spracherkennung / Transkription','Speech recognition / transcription']},
 {id:'text-to-speech',category:'speech',label:['Sprache erzeugen','Text to speech']},
 {id:'text-generation',category:'text',label:['Text / Chat','Text / chat']},
 {id:'image-text-to-text',category:'analysis',label:['Bilder mit Text analysieren','Image and text analysis']},
 {id:'image-to-text',category:'analysis',label:['Bildbeschreibung','Image captioning']},
 {id:'video-text-to-text',category:'analysis',label:['Videoanalyse','Video analysis']},
 {id:'image-segmentation',category:'analysis',label:['Bildsegmentierung','Image segmentation']},
 {id:'depth-estimation',category:'analysis',label:['Tiefenschätzung','Depth estimation']},
 {id:'object-detection',category:'analysis',label:['Objekterkennung','Object detection']},
 {id:'audio-classification',category:'analysis',label:['Audioanalyse','Audio classification']},
] as const;
export const tasksForCategory=(category:HubCategory)=>hubTasks.filter(task=>category==='all'||task.category===category);
export function taskForCategory(category:HubCategory,current:string){
 if(category==='all')return '';
 const tasks=tasksForCategory(category);
 return tasks.some(task=>task.id===current)?current:tasks[0].id;
}
export const hubTaskLabel=(task:string,de:boolean)=>hubTasks.find(item=>item.id===task)?.label[de?0:1]??task;
export function hubTaskSupport(task:string,de:boolean){
 if(task==='text-to-image'||task==='image-to-image')return de?'Bildstudio: vollständige SDXL-Checkpoints im unterstützten Format. Andere Bildmodelle sind nicht automatisch ausführbar.':'Image studio: complete SDXL checkpoints in the supported format. Other image models are not automatically executable.';
 if(task==='text-generation')return de?'Assistent: unterstützte GGUF-Sprachmodelle. Format und Hardware werden nach dem Import geprüft.':'Assistant: supported GGUF language models. Format and hardware are checked after import.';
 if(task==='automatic-speech-recognition')return de?'Transkription: Whisper-Modelle im unterstützten Format. Ein Suchtreffer allein bestätigt keine Ausführbarkeit.':'Transcription: Whisper models in the supported format. A search result alone does not confirm compatibility.';
 if(!task)return de?'Lokale Modell-Ausführung besteht derzeit für SDXL-Bilder, GGUF-Chat und Whisper-Transkription. Andere Aufgaben lassen sich bereits suchen und herunterladen.':'Local model execution currently supports SDXL images, GGUF chat and Whisper transcription. Other tasks can already be searched and downloaded.';
 return de?'Suche und Download verfügbar. Für diese Aufgabe ist noch kein lokaler Modelladapter integriert.':'Search and download are available. A local model adapter for this task is not integrated yet.';
}
