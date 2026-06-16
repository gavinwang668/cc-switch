import type { ReactNode } from "react";
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach, vi } from "vitest";
import { RouterProvider } from "@/lib/router";
import { useAppRouter } from "@/hooks/useAppRouter";

const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
  };
})();

Object.defineProperty(window, "localStorage", { value: localStorageMock });

function wrapper({ children }: { children: ReactNode }) {
  return <RouterProvider>{children}</RouterProvider>;
}

describe("useAppRouter", () => {
  beforeEach(() => {
    localStorageMock.clear();
  });

  it("初始视图为 providers", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    expect(result.current.currentView).toBe("providers");
  });

  it("navigate 切换视图并持久化到 localStorage", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.navigate("settings"));
    expect(result.current.currentView).toBe("settings");
    expect(localStorageMock.setItem).toHaveBeenCalledWith(
      "cc-switch-last-view",
      "settings",
    );
  });

  it("openSettings 同时设置默认 tab 并导航", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.openSettings("proxy"));
    expect(result.current.currentView).toBe("settings");
    expect(result.current.settingsDefaultTab).toBe("proxy");
  });

  it("openProviders 导航到 providers", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.navigate("mcp"));
    act(() => result.current.openProviders());
    expect(result.current.currentView).toBe("providers");
  });

  it("openSessions 导航到 sessions", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.openSessions());
    expect(result.current.currentView).toBe("sessions");
  });

  it("openMcp 导航到 mcp", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.openMcp());
    expect(result.current.currentView).toBe("mcp");
  });

  it("openSkills 导航到 skills", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.openSkills());
    expect(result.current.currentView).toBe("skills");
  });

  it("openSkillsDiscovery 导航到 skillsDiscovery", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.openSkillsDiscovery());
    expect(result.current.currentView).toBe("skillsDiscovery");
  });

  it("goBack 从 skillsDiscovery 返回 skills", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.navigate("skillsDiscovery"));
    act(() => result.current.goBack());
    expect(result.current.currentView).toBe("skills");
  });

  it("goBack 从其他视图返回 providers", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.navigate("settings"));
    act(() => result.current.goBack());
    expect(result.current.currentView).toBe("providers");
  });

  it("openPrompts / openAgents / openUniversal / openWorkspace", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.openPrompts());
    expect(result.current.currentView).toBe("prompts");
    act(() => result.current.openAgents());
    expect(result.current.currentView).toBe("agents");
    act(() => result.current.openUniversal());
    expect(result.current.currentView).toBe("universal");
    act(() => result.current.openWorkspace());
    expect(result.current.currentView).toBe("workspace");
  });

  it("openOpenclawEnv / openOpenclawTools / openOpenclawAgents / openHermesMemory", () => {
    const { result } = renderHook(() => useAppRouter(), { wrapper });
    act(() => result.current.openOpenclawEnv());
    expect(result.current.currentView).toBe("openclawEnv");
    act(() => result.current.openOpenclawTools());
    expect(result.current.currentView).toBe("openclawTools");
    act(() => result.current.openOpenclawAgents());
    expect(result.current.currentView).toBe("openclawAgents");
    act(() => result.current.openHermesMemory());
    expect(result.current.currentView).toBe("hermesMemory");
  });
});