import {invoke} from '@tauri-apps/api/core';

export interface BugReportInput{title:string;description:string;steps:string;uiContext:string}
export interface BugReportDraft{report:string;included:string[];excluded:string[]}
export interface BugReportResult{path:string;report:string}

export const bugReportApi={
 preview:(input:BugReportInput)=>invoke<BugReportDraft>('bug_report_preview',{input}),
 submit:(input:BugReportInput)=>invoke<BugReportResult>('bug_report_submit',{input}),
 openFolder:()=>invoke<void>('bug_report_open_folder'),
};

export function bugReportError(error:unknown,de:boolean){
 const messages:Record<string,[string,string]>={
  bug_report_input:['Titel oder Beschreibung ist zu lang.','The title or description is too long.'],
  bug_report_size:['Der Diagnosebericht ist zu groß. Bitte starte Local Studio neu und versuche es erneut.','The diagnostic report is too large. Restart Local Studio and try again.'],
  bug_report_storage:['Der Diagnosebericht konnte nicht lokal gespeichert werden.','The diagnostic report could not be saved locally.'],
  bug_report_open:['GitHub oder der Berichtordner konnte nicht geöffnet werden.','GitHub or the report folder could not be opened.'],
 };
 return messages[String(error)]?.[de?0:1]??String(error);
}
