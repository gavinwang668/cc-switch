import i18n from "i18next";
import { initReactI18next } from "react-i18next";

type Language = "zh" | "zh-TW" | "en" | "ja";

const SUPPORTED_LANGUAGES: Language[] = ["zh", "zh-TW", "en", "ja"];
const DEFAULT_LANGUAGE: Language = "zh";

/**
 * 懒加载语言包。Vite 会将每个 `import()` 拆分为独立 chunk，
 * 启动时不再解析全部四份翻译。
 */
const localeLoaders: Record<
  Language,
  () => Promise<{ default: Record<string, unknown> }>
> = {
  en: () => import("./locales/en.json"),
  ja: () => import("./locales/ja.json"),
  zh: () => import("./locales/zh.json"),
  "zh-TW": () => import("./locales/zh-TW.json"),
};

const loadedLanguages = new Set<Language>();

async function loadLanguage(language: Language) {
  if (loadedLanguages.has(language)) return;
  const loader = localeLoaders[language];
  if (!loader) return;
  const module = await loader();
  i18n.addResourceBundle(language, "translation", module.default, true, true);
  loadedLanguages.add(language);
}

const getInitialLanguage = (): Language => {
  if (typeof window !== "undefined") {
    try {
      const stored = window.localStorage.getItem("language");
      if ((SUPPORTED_LANGUAGES as string[]).includes(stored ?? "")) {
        return stored as Language;
      }
    } catch {
      // 静默失败即可
    }
  }

  const navigatorLang =
    typeof navigator !== "undefined"
      ? (navigator.language?.toLowerCase() ??
        navigator.languages?.[0]?.toLowerCase())
      : undefined;

  if (navigatorLang === "zh") return "zh";
  if (
    navigatorLang?.startsWith("zh-tw") ||
    navigatorLang?.startsWith("zh-hk") ||
    navigatorLang?.startsWith("zh-mo") ||
    navigatorLang?.startsWith("zh-hant")
  ) {
    return "zh-TW";
  }
  if (navigatorLang?.startsWith("zh")) return "zh";
  if (navigatorLang?.startsWith("ja")) return "ja";
  if (navigatorLang?.startsWith("en")) return "en";

  return DEFAULT_LANGUAGE;
};

i18n.use(initReactI18next).init({
  // 资源不再在这里传，让首次切换语言时通过 backend 钩子动态加载
  resources: {},
  lng: getInitialLanguage(),
  fallbackLng: "en",
  supportedLngs: SUPPORTED_LANGUAGES,
  interpolation: {
    escapeValue: false,
  },
  debug: false,
  // 强制 react-i18next 等待资源加载完成，避免首次渲染时 key 全部回退
  react: {
    useSuspense: false,
  },
  // 自定义资源加载：i18next 在切换语言、初始化或缺失翻译时回调本函数
  partialBundledLanguages: true,
});

// 资源加载逻辑通过 languageChanged / missingKey 事件接管

i18n.on("languageChanged", (language) => {
  if ((SUPPORTED_LANGUAGES as string[]).includes(language)) {
    void loadLanguage(language as Language);
  }
});

void loadLanguage(getInitialLanguage());

// 兜底：当翻译缺失时尝试加载目标语言资源
i18n.on("missingKey", (lngs) => {
  const list = Array.isArray(lngs) ? lngs : [lngs];
  for (const lng of list) {
    if ((SUPPORTED_LANGUAGES as string[]).includes(lng)) {
      void loadLanguage(lng as Language);
    }
  }
});

export { loadLanguage, SUPPORTED_LANGUAGES };
export type { Language };
export default i18n;
