import { en } from './en';
import { th } from './th';

export type LangKey = keyof typeof en;
export type Lang = 'en' | 'th';

const translations: Record<Lang, Record<string, string>> = { en, th };

export function t(lang: Lang, key: LangKey, vars?: Record<string, string | number>): string {
  let text = translations[lang]?.[key] ?? translations['en'][key] ?? key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      text = text.replaceAll(`{${k}}`, String(v));
    }
  }
  return text;
}

export { en, th };
