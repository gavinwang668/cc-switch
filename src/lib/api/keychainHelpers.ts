/**
 * Keychain 辅助函数 — 将 Provider 的 API Key 从 settingsConfig.env 中提取到系统 Keychain，
 * 并在读取时从 Keychain 恢复，避免 API Key 明文存储在 SQLite 中。
 */
import type { Provider } from "@/types";
import type { AppId } from "./types";
import { keychainApi } from "./keychain";

/** 哪些 env 键名包含这些子串视为敏感信息（API Key / Token 等） */
const SENSITIVE_ENV_PATTERNS = [
  "API_KEY",
  "AUTH_TOKEN",
  "SECRET",
  "TOKEN",
  "PASSWORD",
  "KEY",
] as const;

/** 排除的非敏感键名（即便匹配上面的 pattern 也不应存入 Keychain） */
const NON_SENSITIVE_ENV_KEYS: ReadonlySet<string> = new Set([
  "BASE_URL",
  "ANTHROPIC_BASE_URL",
  "OPENAI_BASE_URL",
  "GEMINI_BASE_URL",
  "GOOGLE_GEMINI_BASE_URL",
  "MODEL",
  "GEMINI_MODEL",
]);

/** DB 中占位符，表示该值实际存储在系统 Keychain 中 */
const KEYCHAIN_PLACEHOLDER = "__KEYCHAIN__";

/**
 * 判断一个 env 键名是否包含 API Key 等敏感信息
 */
function isSensitiveEnvKey(key: string): boolean {
  if (NON_SENSITIVE_ENV_KEYS.has(key.toUpperCase())) return false;
  const upper = key.toUpperCase();
  return SENSITIVE_ENV_PATTERNS.some((p) => upper.includes(p));
}

/**
 * 从 Provider 的 settingsConfig.env 中提取所有敏感字段，
 * 将它们的值存入系统 Keychain，并在 DB 中用占位符替换。
 *
 * 如果有任何敏感字段，则序列化为 JSON 后存入 Keychain
 * （因为 keychain 每个 provider 只能存一个字符串）。
 *
 * @returns 修改后的 provider（settingsConfig.env 中的敏感值已替换为 __KEYCHAIN__）
 */
export async function extractApiKeysFromProvider(
  provider: Provider,
  appId: AppId,
): Promise<Provider> {
  const env = (provider.settingsConfig as Record<string, unknown>)?.env as
    | Record<string, string>
    | undefined;
  if (!env || typeof env !== "object") return provider;

  const sensitiveKeys: string[] = [];
  const secrets: Record<string, string> = {};

  for (const [key, value] of Object.entries(env)) {
    if (
      isSensitiveEnvKey(key) &&
      typeof value === "string" &&
      value.length > 0
    ) {
      sensitiveKeys.push(key);
      secrets[key] = value;
    }
  }

  if (sensitiveKeys.length === 0) return provider;

  // 序列化所有敏感值，以 JSON 存入 Keychain
  const jsonValue = JSON.stringify(secrets);

  // 存入系统 Keychain
  try {
    await keychainApi.setApiKey(provider.id, appId, jsonValue);
  } catch {
    // Keychain 写入失败时静默处理，但在 provider 上设置标记
    console.warn(`[Keychain] Failed to store API keys for provider ${provider.id}`);
    const clonedProvider = structuredClone(provider);
    clonedProvider.keychainError = true;
    return clonedProvider;
  }

  // 在原对象中用占位符替换
  const clonedProvider = structuredClone(provider);
  const clonedEnv = (clonedProvider.settingsConfig as Record<string, unknown>)
    .env as Record<string, string>;
  for (const key of sensitiveKeys) {
    clonedEnv[key] = KEYCHAIN_PLACEHOLDER;
  }

  return clonedProvider;
}

/**
 * 从系统 Keychain 恢复 Provider 的 API Key 到 settingsConfig.env 中。
 *
 * @returns 修改后的 provider（敏感值已从 Keychain 恢复）
 */
export async function restoreApiKeysToProvider(
  provider: Provider,
  appId: AppId,
): Promise<Provider> {
  const env = (provider.settingsConfig as Record<string, unknown>)?.env as
    | Record<string, string>
    | undefined;
  if (!env || typeof env !== "object") return provider;

  // 检查是否有任何占位符
  const hasPlaceholder = Object.values(env).some(
    (v) => v === KEYCHAIN_PLACEHOLDER,
  );
  if (!hasPlaceholder) return provider;

  try {
    const jsonValue = await keychainApi.getApiKey(provider.id, appId);
    if (!jsonValue) {
      // Keychain 中没有数据，可能是首次使用或之前存储失败
      const clonedProvider = structuredClone(provider);
      clonedProvider.keychainError = true;
      return clonedProvider;
    }

    const secrets: Record<string, string> = JSON.parse(jsonValue);

    const clonedProvider = structuredClone(provider);
    const clonedEnv = (clonedProvider.settingsConfig as Record<string, unknown>)
      .env as Record<string, string>;

    for (const [key, value] of Object.entries(secrets)) {
      if (clonedEnv[key] === KEYCHAIN_PLACEHOLDER) {
        clonedEnv[key] = value;
      }
    }

    return clonedProvider;
  } catch {
    // Keychain 读取失败时静默返回原 provider（占位符无法恢复），并设置标记
    console.warn(`[Keychain] Failed to restore API keys for provider ${provider.id}`);
    const clonedProvider = structuredClone(provider);
    clonedProvider.keychainError = true;
    return clonedProvider;
  }
}

/**
 * 删除 Provider 在系统 Keychain 中的 API Key 条目。
 * 应在 Provider 被删除时调用。
 */
export async function deleteProviderApiKeys(
  providerId: string,
  appId: AppId,
): Promise<void> {
  try {
    await keychainApi.deleteApiKey(providerId, appId);
  } catch {
    // Keychain 删除失败时静默忽略（条目可能不存在）
  }
}

/**
 * 批量恢复多个 Provider 的 API Key。
 */
export async function restoreApiKeysToProviders(
  providers: Record<string, Provider>,
  appId: AppId,
): Promise<Record<string, Provider>> {
  const result: Record<string, Provider> = {};
  const entries = Object.entries(providers);

  await Promise.all(
    entries.map(async ([id, provider]) => {
      result[id] = await restoreApiKeysToProvider(provider, appId);
    }),
  );

  return result;
}
