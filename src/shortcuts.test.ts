import { describe, expect, it } from 'vitest';
import { chordFromEvent, defaultShortcuts, shortcutFor, shortcutIssue } from './shortcuts';
const event = (key: string, extra = {}) => ({ key, ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...extra });
describe('shortcuts', () => {
  it('normalizes keyboard layouts and ignores IME, AltGr and system modifiers', () => {
    expect(chordFromEvent(event('+', { ctrlKey: true, shiftKey: true }))).toBe('Ctrl+Plus');
    expect(chordFromEvent(event('=', { ctrlKey: true }))).toBe('Ctrl+Plus');
    expect(chordFromEvent(event('g', { ctrlKey: true, shiftKey: true }))).toBe('Ctrl+Shift+G');
    expect(chordFromEvent(event('g', { isComposing: true }))).toBeNull();
    expect(chordFromEvent(event('g', { metaKey: true }))).toBeNull();
    expect(chordFromEvent(event('g', { getModifierState: () => true }))).toBeNull();
    expect(chordFromEvent(event('Escape'))).toBeNull();
  });
  it('detects conflicts and reserved editing bindings before persistence', () => {
    expect(shortcutIssue('Ctrl+A', defaultShortcuts, 'generate')).toBe('conflict');
    expect(shortcutIssue('Ctrl+Z', defaultShortcuts, 'generate')).toBe('conflict');
    expect(shortcutIssue('Ctrl+Z', defaultShortcuts, 'undo')).toBeNull();
    expect(shortcutIssue('Ctrl+S', defaultShortcuts, 'generate')).toBe('conflict');
    expect(shortcutIssue('Ctrl+Enter', defaultShortcuts, 'generate')).toBeNull();
    expect(shortcutIssue('', defaultShortcuts, 'generate')).toBeNull();
  });
  it('uses only saved bindings and supports removing a binding', () => {
    const custom = { ...defaultShortcuts, delete: 'Ctrl+D', rename: '' };
    expect(shortcutFor(event('Delete'), custom)).toBeUndefined();
    expect(shortcutFor(event('d', { ctrlKey: true }), custom)).toBe('delete');
    expect(shortcutFor(event('F2'), custom)).toBeUndefined();
  });
});
