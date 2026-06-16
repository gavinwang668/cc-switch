export interface NotificationEvent {
  id: string;
  type: "info" | "warning" | "error" | "success";
  title: string;
  message: string;
  timestamp: number;
  read: boolean;
}

export interface NotificationPreferences {
  enabled: boolean;
  showDesktopNotifications: boolean;
  showInAppNotifications: boolean;
  soundEnabled: boolean;
  events: {
    providerSwitch: boolean;
    failoverTriggered: boolean;
    circuitBreakerOpen: boolean;
    mcpConnectionFailed: boolean;
    budgetAlert: boolean;
    syncCompleted: boolean;
  };
}

export const DEFAULT_NOTIFICATION_PREFERENCES: NotificationPreferences = {
  enabled: true,
  showDesktopNotifications: true,
  showInAppNotifications: true,
  soundEnabled: false,
  events: {
    providerSwitch: false,
    failoverTriggered: true,
    circuitBreakerOpen: true,
    mcpConnectionFailed: true,
    budgetAlert: true,
    syncCompleted: false,
  },
};
