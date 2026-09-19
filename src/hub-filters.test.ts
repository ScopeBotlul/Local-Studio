import {describe,expect,it} from 'vitest';
import {hubCategories,hubTasks,hubTaskLabel,hubTaskSupport,taskForCategory,tasksForCategory} from './hub-filters';
describe('Hub search categories',()=>{
 it('selects a real task when switching media types instead of retaining the old filter',()=>{
  expect(taskForCategory('video','text-to-image')).toBe('text-to-video');
  expect(taskForCategory('speech','text-generation')).toBe('automatic-speech-recognition');
  expect(taskForCategory('all','image-to-video')).toBe('');
 });
 it('retains an explicitly selected task within the same category',()=>{
  expect(taskForCategory('video','image-to-video')).toBe('image-to-video');
  expect(taskForCategory('speech','text-to-speech')).toBe('text-to-speech');
 });
 it('keeps speech and chat distinct and exposes all tasks with unique identifiers',()=>{
  expect(tasksForCategory('speech').map(t=>t.id)).toEqual(['automatic-speech-recognition','text-to-speech']);
  expect(tasksForCategory('text').map(t=>t.id)).toEqual(['text-generation']);
  expect(new Set(hubTasks.map(t=>t.id)).size).toBe(hubTasks.length);
  for(const category of hubCategories)expect(tasksForCategory(category).length).toBeGreaterThan(0);
 });
 it('does not label searchable video or audio models as executable',()=>{
  for(const task of ['text-to-video','video-to-video','text-to-audio','audio-to-audio','text-to-speech'])expect(hubTaskSupport(task,true)).toContain('noch kein lokaler Modelladapter');
  expect(hubTaskSupport('text-to-image',true)).toContain('SDXL');
  expect(hubTaskLabel('automatic-speech-recognition',true)).toContain('Transkription');
  expect(hubTaskLabel('unknown-task',false)).toBe('unknown-task');
 });
});
