import { invoke } from "@tauri-apps/api/core";
import type { AppId } from "./types";

export const keychainApi = {
  /**
   * 将 API Key 安全存储到系统 Keychain
   */
  async setApiKey(
    providerId: string,
    appType: AppId,
    apiKey: string,
  ): Promise<void> {
    await invoke("set_api_key", {
      providerId,
      appType,
      apiKey,
    });
  },

  /**
   * 从系统 Keychain 读取 API Key
   */
  async getApiKey(providerId: string, appType: AppId): Promise<string | null> {
    const result = await invoke<string | null>("get_api_key", {
      providerId,
      appType,
    });
    return result;
  },

  /**
   * 从系统 Keychain 删除 API Key
   */
  async deleteApiKey(providerId: string, appType: AppId): Promise<void> {
    await invoke("delete_api_key", {
      providerId,
      appType,
    });
  },
};
