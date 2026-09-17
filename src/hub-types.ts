export interface HfAccount { username: string; displayName: string; }
export interface HfAuthStatus {
  account: HfAccount | null; method: string | null; pending: boolean;
  oauthConfigured: boolean; expired: boolean; verified: boolean; error: string | null;
}
export interface HfQuery { search: string; task: string; sort: string; cursor: string | null; }
export interface HfModel {
  id: string; task: string | null; library: string | null; downloads: number; likes: number;
  gated: boolean; private: boolean; license: string | null; revision: string | null;
}
export interface HfSearchPage { models: HfModel[]; nextCursor: string | null; }
export interface HfModelDetail {
  model: HfModel; revision: string;
  files: { path: string; size: number | null; sha256: string | null }[];
  card: string | null; cardError: string | null;
}
