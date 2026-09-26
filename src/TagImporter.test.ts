import {describe,expect,it} from 'vitest';
import {flatTags,parseTags} from './TagImporter';

describe('tag import',()=>{
 it('removes headings, markers, counts, duplicates and optional censor tags',()=>{
  const input='Character\n? cartethyia_(wuthering_waves) 2.4k\nGeneral\n? 1girl 8.4M\n? areola slip 76k\n? bar censor 243k\n? 1girl 8.4M';
  const parsed=parseTags(input,true);
  expect(parsed.Character).toEqual(['cartethyia (wuthering waves)']);
  expect(parsed.Body).toEqual(['1girl','areola slip']);
  expect(flatTags(parsed)).not.toContain('bar censor');
 });
 it('keeps censorship tags when the setting is off',()=>expect(flatTags(parseTags('bar_censor 12k',false))).toEqual(['bar censor']));
});
