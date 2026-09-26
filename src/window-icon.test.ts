import { describe, expect, it } from 'vitest';
import { accentInk } from './window-icon';

describe('accent icon contrast', () => {
  it('keeps the logo visible for light and dark accent colors', () => {
    expect(accentInk('#f2d15f')).toBe('#102421');
    expect(accentInk('#651b2c')).toBe('#ffffff');
    expect(accentInk('invalid')).toBe('#102421');
  });
});
