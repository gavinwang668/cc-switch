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
const failedLanguages = new Set<Language>();

async function loadLanguage(language: Language): Promise<boolean> {
  if (loadedLanguages.has(language)) return true;
  // 已经失败过的语言不再重试，避免无限循环的 missingKey 触发
  if (failedLanguages.has(language)) return false;
  const loader = localeLoaders[language];
  if (!loader) return false;

  try {
    const module = await loader();
    i18n.addResourceBundle(language, "translation", module.default, true, true);
    loadedLanguages.add(language);
    return true;
  } catch (error) {
    failedLanguages.add(language);
    console.error(`[i18n] 加载语言包失败 (${language}):`, error);
    // 如果默认语言加载失败，尝试回退到英文
    if (language === DEFAULT_LANGUAGE && language !== "en") {
      console.warn("[i18n] 默认语言加载失败，回退到英文");
      const enLoaded = await loadLanguage("en");
      if (enLoaded) {
        // 设置标志避免 languageChanged 处理器重复加载
        languageChangedFromFallback = true;
        i18n.changeLanguage("en");
      }
    }
    return false;
  }
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

let languageChangedFromFallback = false;

i18n.on("languageChanged", (language) => {
  // 如果是从 fallback 触发的切换，跳过避免循环
  if (languageChangedFromFallback) {
    languageChangedFromFallback = false;
    return;
  }
  if ((SUPPORTED_LANGUAGES as string[]).includes(language)) {
    void loadLanguage(language as Language);
  }
});

void loadLanguage(getInitialLanguage());

// 兜底：当翻译缺失时尝试加载目标语言资源
// 只对尚未失败的语言尝试加载，避免无限触发 missingKey
i18n.on("missingKey", (lngs) => {
  const list = Array.isArray(lngs) ? lngs : [lngs];
  for (const lng of list) {
    if (
      (SUPPORTED_LANGUAGES as string[]).includes(lng) &&
      !failedLanguages.has(lng as Language)
    ) {
      void loadLanguage(lng as Language);
    }
  }
});

export { loadLanguage, SUPPORTED_LANGUAGES };
export type { Language };
export default i18n;
