import { useRouter } from "@/lib/router";

export function useAppRouter() {
  const router = useRouter();

  return {
    ...router,
    openSettings: (tab = "general") => {
      router.setSettingsDefaultTab(tab);
      router.navigate("settings");
    },
    openPrompts: () => router.navigate("prompts"),
    openSkills: () => router.navigate("skills"),
    openSkillsDiscovery: () => router.navigate("skillsDiscovery"),
    openMcp: () => router.navigate("mcp"),
    openAgents: () => router.navigate("agents"),
    openUniversal: () => router.navigate("universal"),
    openSessions: () => router.navigate("sessions"),
    openWorkspace: () => router.navigate("workspace"),
    openOpenclawEnv: () => router.navigate("openclawEnv"),
    openOpenclawTools: () => router.navigate("openclawTools"),
    openOpenclawAgents: () => router.navigate("openclawAgents"),
    openHermesMemory: () => router.navigate("hermesMemory"),
    openProviders: () => router.navigate("providers"),
  };
}
