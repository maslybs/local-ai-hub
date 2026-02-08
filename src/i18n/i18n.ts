import { en } from './translations/en';
import { uk } from './translations/uk';

export type SupportedLang = 'en' | 'uk';
export type LangChoice = SupportedLang | 'system';

const TRANSLATIONS: Record<SupportedLang, any> = {
  en,
  uk,
};

function normalizeSystemLang(raw: string | null | undefined): SupportedLang {
  const v = String(raw ?? '').toLowerCase();
  // Prefer Ukrainian for any uk/ua locale.
  if (v.startsWith('uk') || v.startsWith('ua')) return 'uk';
  return 'en';
}

export function resolveLanguage(choice: string | null | undefined): SupportedLang {
  if (!choice || choice === 'system') {
    const sys = (globalThis.navigator as any)?.language ?? 'en';
    return normalizeSystemLang(sys);
  }
  if (choice === 'en' || choice === 'uk') return choice;
  return 'en';
}

export function t(lang: SupportedLang, key: string, vars?: Record<string, string | number>): string {
  const dict = TRANSLATIONS[lang] ?? TRANSLATIONS.en;
  const parts = key.split('.');
  let cur: any = dict;
  for (const p of parts) {
    cur = cur?.[p];
  }
  const template = typeof cur === 'string' ? cur : key;
  if (!vars) return template;
  return template.replace(/\{(\w+)\}/g, (_, name) => {
    const v = vars[name];
    return v === undefined || v === null ? `{${name}}` : String(v);
  });
}

