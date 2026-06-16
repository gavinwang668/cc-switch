import React, { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { toast } from "sonner";
import { Bell, BellOff } from "lucide-react";
import type { NotificationPreferences } from "@/types/notification";
import { DEFAULT_NOTIFICATION_PREFERENCES } from "@/types/notification";

export const NotificationSettings: React.FC = () => {
  const { t } = useTranslation();
  const [preferences, setPreferences] = useState<NotificationPreferences>(
    DEFAULT_NOTIFICATION_PREFERENCES,
  );
  const [permission, setPermission] =
    useState<NotificationPermission>("default");

  useEffect(() => {
    // Load saved preferences
    const saved = localStorage.getItem("notification-preferences");
    if (saved) {
      try {
        setPreferences(JSON.parse(saved));
      } catch (error) {
        console.error("Failed to load notification preferences:", error);
      }
    }

    // Check notification permission
    if ("Notification" in window) {
      setPermission(Notification.permission);
    }
  }, []);

  const savePreferences = (newPreferences: NotificationPreferences) => {
    setPreferences(newPreferences);
    localStorage.setItem(
      "notification-preferences",
      JSON.stringify(newPreferences),
    );
  };

  const handleToggle = (key: keyof NotificationPreferences) => {
    const newPreferences = { ...preferences, [key]: !preferences[key] };
    savePreferences(newPreferences);
  };

  const handleEventToggle = (
    event: keyof NotificationPreferences["events"],
  ) => {
    const newPreferences = {
      ...preferences,
      events: {
        ...preferences.events,
        [event]: !preferences.events[event],
      },
    };
    savePreferences(newPreferences);
  };

  const requestPermission = async () => {
    if (!("Notification" in window)) {
      toast.error(t("settings.notifications.notSupported"));
      return;
    }

    const result = await Notification.requestPermission();
    setPermission(result);

    if (result === "granted") {
      toast.success(t("settings.notifications.permissionGranted"));
    } else if (result === "denied") {
      toast.error(t("settings.notifications.permissionDenied"));
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {preferences.enabled ? (
              <Bell className="h-5 w-5 text-primary" />
            ) : (
              <BellOff className="h-5 w-5 text-muted-foreground" />
            )}
            <CardTitle>{t("settings.notifications.title")}</CardTitle>
          </div>
          <Switch
            checked={preferences.enabled}
            onCheckedChange={() => handleToggle("enabled")}
          />
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {preferences.enabled && (
          <>
            {/* Permission Status */}
            <div className="space-y-2">
              <Label>{t("settings.notifications.permission")}</Label>
              <div className="flex items-center gap-2">
                <code className="px-3 py-1.5 bg-muted rounded-md text-sm">
                  {permission === "granted"
                    ? t("settings.notifications.granted")
                    : permission === "denied"
                      ? t("settings.notifications.denied")
                      : t("settings.notifications.default")}
                </code>
                {permission !== "granted" && (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={requestPermission}
                  >
                    {t("settings.notifications.requestPermission")}
                  </Button>
                )}
              </div>
            </div>

            {/* Notification Types */}
            <div className="space-y-4">
              <Label>{t("settings.notifications.types")}</Label>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <Label htmlFor="desktop-notifications" className="flex-1">
                    {t("settings.notifications.desktopNotifications")}
                  </Label>
                  <Switch
                    id="desktop-notifications"
                    checked={preferences.showDesktopNotifications}
                    onCheckedChange={() =>
                      handleToggle("showDesktopNotifications")
                    }
                  />
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="inapp-notifications" className="flex-1">
                    {t("settings.notifications.inAppNotifications")}
                  </Label>
                  <Switch
                    id="inapp-notifications"
                    checked={preferences.showInAppNotifications}
                    onCheckedChange={() =>
                      handleToggle("showInAppNotifications")
                    }
                  />
                </div>
                <div className="flex items-center justify-between">
                  <Label htmlFor="sound-enabled" className="flex-1">
                    {t("settings.notifications.soundEnabled")}
                  </Label>
                  <Switch
                    id="sound-enabled"
                    checked={preferences.soundEnabled}
                    onCheckedChange={() => handleToggle("soundEnabled")}
                  />
                </div>
              </div>
            </div>

            {/* Event Filters */}
            <div className="space-y-4">
              <Label>{t("settings.notifications.eventFilters")}</Label>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <Label htmlFor="provider-switch">
                      {t("settings.notifications.events.providerSwitch")}
                    </Label>
                    <p className="text-xs text-muted-foreground mt-1">
                      {t("settings.notifications.events.providerSwitchDesc")}
                    </p>
                  </div>
                  <Switch
                    id="provider-switch"
                    checked={preferences.events.providerSwitch}
                    onCheckedChange={() => handleEventToggle("providerSwitch")}
                  />
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <Label htmlFor="failover-triggered">
                      {t("settings.notifications.events.failoverTriggered")}
                    </Label>
                    <p className="text-xs text-muted-foreground mt-1">
                      {t("settings.notifications.events.failoverTriggeredDesc")}
                    </p>
                  </div>
                  <Switch
                    id="failover-triggered"
                    checked={preferences.events.failoverTriggered}
                    onCheckedChange={() =>
                      handleEventToggle("failoverTriggered")
                    }
                  />
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <Label htmlFor="circuit-breaker-open">
                      {t("settings.notifications.events.circuitBreakerOpen")}
                    </Label>
                    <p className="text-xs text-muted-foreground mt-1">
                      {t(
                        "settings.notifications.events.circuitBreakerOpenDesc",
                      )}
                    </p>
                  </div>
                  <Switch
                    id="circuit-breaker-open"
                    checked={preferences.events.circuitBreakerOpen}
                    onCheckedChange={() =>
                      handleEventToggle("circuitBreakerOpen")
                    }
                  />
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <Label htmlFor="mcp-connection-failed">
                      {t("settings.notifications.events.mcpConnectionFailed")}
                    </Label>
                    <p className="text-xs text-muted-foreground mt-1">
                      {t(
                        "settings.notifications.events.mcpConnectionFailedDesc",
                      )}
                    </p>
                  </div>
                  <Switch
                    id="mcp-connection-failed"
                    checked={preferences.events.mcpConnectionFailed}
                    onCheckedChange={() =>
                      handleEventToggle("mcpConnectionFailed")
                    }
                  />
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <Label htmlFor="budget-alert">
                      {t("settings.notifications.events.budgetAlert")}
                    </Label>
                    <p className="text-xs text-muted-foreground mt-1">
                      {t("settings.notifications.events.budgetAlertDesc")}
                    </p>
                  </div>
                  <Switch
                    id="budget-alert"
                    checked={preferences.events.budgetAlert}
                    onCheckedChange={() => handleEventToggle("budgetAlert")}
                  />
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex-1">
                    <Label htmlFor="sync-completed">
                      {t("settings.notifications.events.syncCompleted")}
                    </Label>
                    <p className="text-xs text-muted-foreground mt-1">
                      {t("settings.notifications.events.syncCompletedDesc")}
                    </p>
                  </div>
                  <Switch
                    id="sync-completed"
                    checked={preferences.events.syncCompleted}
                    onCheckedChange={() => handleEventToggle("syncCompleted")}
                  />
                </div>
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  );
};
