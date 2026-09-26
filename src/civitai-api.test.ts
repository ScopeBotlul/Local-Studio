import {describe,expect,it} from 'vitest';
import {civitaiDirectDownloadEligible,type CivitaiFile} from './civitai-api';

const safeFile: CivitaiFile = {
  id: 1,
  name: 'model.safetensors',
  sizeBytes: 1024,
  sha256: 'a'.repeat(64),
  format: 'SafeTensor',
  pickleScan: 'Success',
  virusScan: 'Success',
  primary: true,
  downloadUrl: 'https://civitai.com/api/download/models/1',
};

describe('Civitai direct-download eligibility', () => {
  it('accepts only a primary, hashed Safetensors file with successful scans', () => {
    expect(civitaiDirectDownloadEligible(safeFile)).toBe(true);
    expect(civitaiDirectDownloadEligible({...safeFile,primary:false})).toBe(false);
    expect(civitaiDirectDownloadEligible({...safeFile,sha256:null})).toBe(false);
    expect(civitaiDirectDownloadEligible({...safeFile,format:'PickleTensor'})).toBe(false);
    expect(civitaiDirectDownloadEligible({...safeFile,virusScan:'Danger'})).toBe(false);
    expect(civitaiDirectDownloadEligible({...safeFile,pickleScan:null})).toBe(false);
  });
});
