import { invoke } from '@tauri-apps/api/core';
import type { HfAuthStatus, HfModelDetail, HfQuery, HfSearchPage } from './hub-types';
export const hubApi = {
  status: () => invoke<HfAuthStatus>('hf_status'),
  startLogin: () => invoke<HfAuthStatus>('hf_start_login'),
  cancelLogin: () => invoke<HfAuthStatus>('hf_cancel_login'),
  connectToken: (token: string) => invoke<HfAuthStatus>('hf_connect_token', { token }),
  logout: () => invoke<HfAuthStatus>('hf_logout'),
  verify: () => invoke<HfAuthStatus>('hf_verify'),
  search: (query: HfQuery) => invoke<HfSearchPage>('hf_search', { query }),
  size: (repo: string, revision: string) => invoke<number | null>('hf_model_size', { repo, revision }),
  detail: (repo: string, revision: string) => invoke<HfModelDetail>('hf_model_detail', { repo, revision }),
  open: (page: 'tokens' | 'applications' | 'home' | 'model', repo?: string) => invoke<void>('hf_open_page', { page, repo: repo ?? null }),
};
