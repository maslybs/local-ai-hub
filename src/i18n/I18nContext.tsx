import React from 'react';
import type { AppConfig } from '@/lib/backend';
import { backend } from '@/lib/backend';
import { resolveLanguage, t as translate, type SupportedLang, type LangChoice } from './i18n';

type I18nCtx = {
  lang: SupportedLang;
  choice: LangChoice;
  setChoice: (c: LangChoice) => Promise<void>;
  t: (key: string, vars?: Record<string, string | number>) => string;
};

const Ctx = React.createContext<I18nCtx | null>(null);

export function I18nProvider({ children }: { children: React.ReactNode }) {
  const [choice, setChoiceState] = React.useState<LangChoice>('system');
  const [lang, setLang] = React.useState<SupportedLang>('en');

  React.useEffect(() => {
    let alive = true;
    (async () => {
      try {
        const cfg = await backend.getConfig();
        const c = (cfg.ui?.language ?? 'system') as LangChoice;
        if (!alive) return;
        setChoiceState(c);
        setLang(resolveLanguage(c));
      } catch {
        // Non-tauri dev (browser) - just use system lang.
        const c: LangChoice = 'system';
        if (!alive) return;
        setChoiceState(c);
        setLang(resolveLanguage(c));
      }
    })();
    return () => { alive = false; };
  }, []);

  React.useEffect(() => {
    const onLangChanged = () => setLang(resolveLanguage(choice));
    window.addEventListener('languagechange', onLangChanged);
    return () => window.removeEventListener('languagechange', onLangChanged);
  }, [choice]);

  const setChoice = React.useCallback(async (c: LangChoice) => {
    setChoiceState(c);
    setLang(resolveLanguage(c));

    try {
      const cfg = await backend.getConfig();
      const next: AppConfig = cfg as any;
      next.ui = next.ui ?? { language: null };
      next.ui.language = c === 'system' ? null : c;
      await backend.saveConfig(next);
    } catch {
      // ignore in browser dev
    }
  }, []);

  const t = React.useCallback((key: string, vars?: Record<string, string | number>) => {
    return translate(lang, key, vars);
  }, [lang]);

  const value: I18nCtx = React.useMemo(() => ({
    lang,
    choice,
    setChoice,
    t,
  }), [lang, choice, setChoice, t]);

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useI18n(): I18nCtx {
  const v = React.useContext(Ctx);
  if (!v) throw new Error('useI18n must be used within I18nProvider');
  return v;
}

