import {
  createContext,
  useContext,
  useState,
  useCallback,
  type ReactNode,
} from "react";

export type View =
  | "providers"
  | "settings"
  | "prompts"
  | "skills"
  | "skillsDiscovery"
  | "mcp"
  | "agents"
  | "universal"
  | "sessions"
  | "workspace"
  | "openclawEnv"
  | "openclawTools"
  | "openclawAgents"
  | "hermesMemory";

const VALID_VIEWS: View[] = [
  "providers",
  "settings",
  "prompts",
  "skills",
  "skillsDiscovery",
  "mcp",
  "agents",
  "universal",
  "sessions",
  "workspace",
  "openclawEnv",
  "openclawTools",
  "openclawAgents",
  "hermesMemory",
];

const VIEW_STORAGE_KEY = "cc-switch-last-view";

const getInitialView = (): View => {
  const saved = localStorage.getItem(VIEW_STORAGE_KEY) as View | null;
  if (saved && VALID_VIEWS.includes(saved)) {
    return saved;
  }
  return "providers";
};

interface RouterState {
  currentView: View;
  navigate: (view: View) => void;
  goBack: () => void;
  settingsDefaultTab: string;
  setSettingsDefaultTab: (tab: string) => void;
}

const RouterContext = createContext<RouterState | null>(null);

export function RouterProvider({ children }: { children: ReactNode }) {
  const [currentView, setCurrentView] = useState<View>(getInitialView);
  const [settingsDefaultTab, setSettingsDefaultTab] = useState("general");

  const navigate = useCallback((view: View) => {
    setCurrentView(view);
    localStorage.setItem(VIEW_STORAGE_KEY, view);
  }, []);

  const goBack = useCallback(() => {
    setCurrentView((prev) =>
      prev === "skillsDiscovery" ? "skills" : "providers",
    );
  }, []);

  return (
    <RouterContext.Provider
      value={{
        currentView,
        navigate,
        goBack,
        settingsDefaultTab,
        setSettingsDefaultTab,
      }}
    >
      {children}
    </RouterContext.Provider>
  );
}

export function useRouter() {
  const ctx = useContext(RouterContext);
  if (!ctx) throw new Error("useRouter must be used within RouterProvider");
  return ctx;
}
