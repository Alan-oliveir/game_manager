import { invoke } from '@tauri-apps/api/core';
import i18n, { type BackendModule } from 'i18next';
import { initReactI18next } from 'react-i18next';

export const LOCALE_STORAGE_KEY = 'playlite-lang';
export const DEFAULT_LANGUAGE = 'pt-BR';
export const SUPPORTED_LANGUAGES = ['en', 'pt-BR'] as const;
export const NAMESPACES = [
  'common',
  'dialog',
  'settings',
  'library',
  'updater',
  'wishlist',
  'trending',
  'playlist',
  'game_detail',
  'platforms',
  'subscription',
  'errors',
] as const;

export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

const normalizeLanguage = (
  language: string | null | undefined
): SupportedLanguage | undefined => {
  if (!language) {
    return undefined;
  }

  const directMatch = SUPPORTED_LANGUAGES.find(
    supportedLanguage => supportedLanguage === language
  );

  if (directMatch) {
    return directMatch;
  }

  const normalized = language.toLowerCase();

  return (
    SUPPORTED_LANGUAGES.find(
      supportedLanguage => supportedLanguage.toLowerCase() === normalized
    ) ??
    SUPPORTED_LANGUAGES.find(
      supportedLanguage =>
        supportedLanguage.split('-')[0].toLowerCase() ===
        normalized.split('-')[0]
    )
  );
};

export const getStoredLanguage = (): SupportedLanguage | undefined => {
  if (typeof window === 'undefined') {
    return undefined;
  }

  return normalizeLanguage(window.localStorage.getItem(LOCALE_STORAGE_KEY));
};

export const setStoredLanguage = (language: string): void => {
  if (typeof window === 'undefined') {
    return;
  }

  const normalized = normalizeLanguage(language) ?? DEFAULT_LANGUAGE;
  window.localStorage.setItem(LOCALE_STORAGE_KEY, normalized);
};

const backend: BackendModule = {
  type: 'backend',
  init() {},
  read(language, namespace, callback) {
    import(`./locales/${language}/${namespace}.json`)
      .then(module => callback(null, module.default))
      .catch(error => {
        if (language !== DEFAULT_LANGUAGE) {
          import(`./locales/${DEFAULT_LANGUAGE}/${namespace}.json`)
            .then(module => callback(null, module.default))
            .catch(fallbackError => callback(fallbackError as Error, false));

          return;
        }

        callback(error as Error, false);
      });
  },
  create() {},
};

const systemLanguage = normalizeLanguage(navigator.language);
const initialLanguage =
  getStoredLanguage() ?? systemLanguage ?? DEFAULT_LANGUAGE;

void i18n
  .use(backend)
  .use(initReactI18next)
  .init({
    lng: initialLanguage,
    fallbackLng: DEFAULT_LANGUAGE,
    supportedLngs: SUPPORTED_LANGUAGES,
    ns: NAMESPACES,
    defaultNS: 'common',
    interpolation: {
      escapeValue: false,
    },
    react: {
      useSuspense: false,
    },
  });

const syncLanguageWithBackend = (language: string): void => {
  const normalized = normalizeLanguage(language) ?? DEFAULT_LANGUAGE;
  invoke('set_app_language', { language: normalized }).catch(error => {
    console.error('Falha ao sincronizar idioma com o backend:', error);
  });
};

i18n.on('languageChanged', setStoredLanguage);
i18n.on('languageChanged', syncLanguageWithBackend);

export default i18n;
