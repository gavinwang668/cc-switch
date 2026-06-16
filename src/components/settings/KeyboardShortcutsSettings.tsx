import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { Keyboard, RotateCcw } from "lucide-react";

interface Shortcut {
  id: string;
  name: string;
  description: string;
  shortcut: string;
  defaultShortcut: string;
}

const DEFAULT_SHORTCUTS: Shortcut[] = [
  {
    id: "open-settings",
    name: "settings.shortcuts.openSettings",
    description: "settings.shortcuts.openSettingsDesc",
    shortcut: "Ctrl+,",
    defaultShortcut: "Ctrl+,",
  },
  {
    id: "switch-view-1",
    name: "settings.shortcuts.switchView1",
    description: "settings.shortcuts.switchView1Desc",
    shortcut: "Ctrl+Shift+1",
    defaultShortcut: "Ctrl+Shift+1",
  },
  {
    id: "switch-view-2",
    name: "settings.shortcuts.switchView2",
    description: "settings.shortcuts.switchView2Desc",
    shortcut: "Ctrl+Shift+2",
    defaultShortcut: "Ctrl+Shift+2",
  },
  {
    id: "switch-view-3",
    name: "settings.shortcuts.switchView3",
    description: "settings.shortcuts.switchView3Desc",
    shortcut: "Ctrl+Shift+3",
    defaultShortcut: "Ctrl+Shift+3",
  },
  {
    id: "switch-view-4",
    name: "settings.shortcuts.switchView4",
    description: "settings.shortcuts.switchView4Desc",
    shortcut: "Ctrl+Shift+4",
    defaultShortcut: "Ctrl+Shift+4",
  },
  {
    id: "switch-view-5",
    name: "settings.shortcuts.switchView5",
    description: "settings.shortcuts.switchView5Desc",
    shortcut: "Ctrl+Shift+5",
    defaultShortcut: "Ctrl+Shift+5",
  },
  {
    id: "go-back",
    name: "settings.shortcuts.goBack",
    description: "settings.shortcuts.goBackDesc",
    shortcut: "Escape",
    defaultShortcut: "Escape",
  },
];

export const KeyboardShortcutsSettings: React.FC = () => {
  const { t } = useTranslation();
  const [shortcuts, setShortcuts] = useState<Shortcut[]>(DEFAULT_SHORTCUTS);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [recordingShortcut, setRecordingShortcut] = useState<string>("");

  useEffect(() => {
    const saved = localStorage.getItem("keyboard-shortcuts");
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        setShortcuts((prev) =>
          prev.map((s) => {
            const custom = parsed.find((c: Shortcut) => c.id === s.id);
            return custom ? { ...s, shortcut: custom.shortcut } : s;
          }),
        );
      } catch (error) {
        console.error("Failed to load keyboard shortcuts:", error);
      }
    }
  }, []);

  const saveShortcuts = (newShortcuts: Shortcut[]) => {
    setShortcuts(newShortcuts);
    localStorage.setItem("keyboard-shortcuts", JSON.stringify(newShortcuts));
  };

  const handleEdit = (id: string) => {
    setEditingId(id);
    setRecordingShortcut("");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    e.preventDefault();

    const keys: string[] = [];
    if (e.ctrlKey) keys.push("Ctrl");
    if (e.shiftKey) keys.push("Shift");
    if (e.altKey) keys.push("Alt");
    if (e.metaKey) keys.push("Meta");

    const key = e.key;
    if (!["Control", "Shift", "Alt", "Meta"].includes(key)) {
      keys.push(key === "," ? "," : key.toUpperCase());
    }

    const shortcut = keys.join("+");
    setRecordingShortcut(shortcut);
  };

  const handleSave = (id: string) => {
    if (!recordingShortcut) {
      toast.error(t("settings.shortcuts.noShortcut"));
      return;
    }

    const newShortcuts = shortcuts.map((s) =>
      s.id === id ? { ...s, shortcut: recordingShortcut } : s,
    );
    saveShortcuts(newShortcuts);
    setEditingId(null);
    setRecordingShortcut("");
    toast.success(t("settings.shortcuts.saved"));
  };

  const handleCancel = () => {
    setEditingId(null);
    setRecordingShortcut("");
  };

  const handleReset = (id: string) => {
    const newShortcuts = shortcuts.map((s) =>
      s.id === id ? { ...s, shortcut: s.defaultShortcut } : s,
    );
    saveShortcuts(newShortcuts);
    toast.success(t("settings.shortcuts.reset"));
  };

  const handleResetAll = () => {
    const newShortcuts = shortcuts.map((s) => ({
      ...s,
      shortcut: s.defaultShortcut,
    }));
    saveShortcuts(newShortcuts);
    toast.success(t("settings.shortcuts.resetAll"));
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Keyboard className="h-5 w-5 text-primary" />
            <CardTitle>{t("settings.shortcuts.title")}</CardTitle>
          </div>
          <Button variant="outline" size="sm" onClick={handleResetAll}>
            <RotateCcw className="h-4 w-4 mr-2" />
            {t("settings.shortcuts.resetAll")}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {shortcuts.map((shortcut) => (
          <div
            key={shortcut.id}
            className="flex items-center justify-between p-3 rounded-lg border border-border/50 hover:bg-muted/50 transition-colors"
          >
            <div className="flex-1">
              <Label className="font-medium">{t(shortcut.name)}</Label>
              <p className="text-sm text-muted-foreground mt-1">
                {t(shortcut.description)}
              </p>
            </div>
            <div className="flex items-center gap-2 ml-4">
              {editingId === shortcut.id ? (
                <>
                  <Input
                    value={recordingShortcut}
                    onKeyDown={(e) => handleKeyDown(e)}
                    placeholder={t("settings.shortcuts.pressKeys")}
                    className="w-40 text-center font-mono"
                    autoFocus
                  />
                  <Button
                    size="sm"
                    onClick={() => handleSave(shortcut.id)}
                    disabled={!recordingShortcut}
                  >
                    {t("common.save")}
                  </Button>
                  <Button size="sm" variant="outline" onClick={handleCancel}>
                    {t("common.cancel")}
                  </Button>
                </>
              ) : (
                <>
                  <code className="px-3 py-1.5 bg-muted rounded-md font-mono text-sm">
                    {shortcut.shortcut}
                  </code>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => handleEdit(shortcut.id)}
                  >
                    {t("common.edit")}
                  </Button>
                  {shortcut.shortcut !== shortcut.defaultShortcut && (
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleReset(shortcut.id)}
                    >
                      <RotateCcw className="h-4 w-4" />
                    </Button>
                  )}
                </>
              )}
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
};
