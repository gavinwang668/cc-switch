import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle2, Loader2, ShieldCheck, XCircle } from "lucide-react";
import { toast } from "sonner";
import ApiKeyInput from "../ApiKeyInput";
import { Button } from "@/components/ui/button";
import { providersApi } from "@/lib/api/providers";
import type { ProviderCategory } from "@/types";
import { extractErrorMessage } from "@/utils/errorUtils";

type VerifyStatus = "idle" | "verifying" | "success" | "failed";

interface ApiKeySectionProps {
  id?: string;
  label?: string;
  value: string;
  onChange: (value: string) => void;
  category?: ProviderCategory;
  shouldShowLink: boolean;
  websiteUrl: string;
  /** 用于验证 Key 的 baseUrl（通常来自 ANTHROPIC_BASE_URL / OPENAI_BASE_URL 等） */
  baseUrl?: string;
  placeholder?: {
    official: string;
    thirdParty: string;
  };
  disabled?: boolean;
  isPartner?: boolean;
  partnerPromotionKey?: string;
}

export function ApiKeySection({
  id,
  label,
  value,
  onChange,
  category,
  shouldShowLink,
  websiteUrl,
  baseUrl,
  placeholder,
  disabled,
  partnerPromotionKey,
}: ApiKeySectionProps) {
  const { t } = useTranslation();
  const [verifyStatus, setVerifyStatus] = useState<VerifyStatus>("idle");
  const [verifyError, setVerifyError] = useState<string | null>(null);

  const defaultPlaceholder = {
    official: t("providerForm.officialNoApiKey", {
      defaultValue: "官方供应商无需 API Key",
    }),
    thirdParty: t("providerForm.apiKeyAutoFill", {
      defaultValue: "输入 API Key，将自动填充到配置",
    }),
  };

  const finalPlaceholder = placeholder || defaultPlaceholder;

  const canVerify =
    !disabled &&
    !!value &&
    !!baseUrl &&
    verifyStatus !== "verifying";

  const handleVerify = async () => {
    if (!canVerify) {
      toast.error(
        t("providerForm.verifyKeyMissingInput", {
          defaultValue: "请先填写 API Key 和请求地址",
        }),
      );
      return;
    }

    setVerifyStatus("verifying");
    setVerifyError(null);
    try {
      const ok = await providersApi.verifyApiKey(baseUrl!, value);
      if (ok) {
        setVerifyStatus("success");
        toast.success(
          t("providerForm.verifyKeySuccess", {
            defaultValue: "API Key 验证成功",
          }),
        );
      } else {
        setVerifyStatus("failed");
        setVerifyError(
          t("providerForm.verifyKeyAuthFailed", {
            defaultValue: "API Key 无效或权限不足",
          }),
        );
        toast.error(
          t("providerForm.verifyKeyAuthFailed", {
            defaultValue: "API Key 无效或权限不足",
          }),
        );
      }
    } catch (error) {
      setVerifyStatus("failed");
      const detail = extractErrorMessage(error);
      setVerifyError(detail);
      toast.error(
        t("providerForm.verifyKeyFailed", {
          defaultValue: "API Key 验证失败：{{error}}",
          error: detail,
        }),
      );
    }
  };

  const renderVerifyButton = () => {
    if (disabled) return null;
    if (category === "official") return null;
    if (!baseUrl || !value) return null;

    return (
      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={() => void handleVerify()}
        disabled={!canVerify}
        className="gap-1.5"
        title={t("providerForm.verifyKeyTooltip", {
          defaultValue: "向服务端发送请求验证 API Key 是否有效",
        })}
      >
        {verifyStatus === "verifying" ? (
          <Loader2 className="h-3.5 w-3.5 animate-spin" />
        ) : verifyStatus === "success" ? (
          <CheckCircle2 className="h-3.5 w-3.5 text-green-500" />
        ) : verifyStatus === "failed" ? (
          <XCircle className="h-3.5 w-3.5 text-red-500" />
        ) : (
          <ShieldCheck className="h-3.5 w-3.5" />
        )}
        {verifyStatus === "success"
          ? t("providerForm.verifyKeySuccessShort", {
              defaultValue: "已验证",
            })
          : verifyStatus === "failed"
            ? t("providerForm.verifyKeyFailedShort", {
                defaultValue: "验证失败",
              })
            : t("providerForm.verifyKey", {
                defaultValue: "验证 Key",
              })}
      </Button>
    );
  };

  return (
    <div className="space-y-1">
      <ApiKeyInput
        id={id}
        label={label}
        value={value}
        onChange={onChange}
        placeholder={
          category === "official"
            ? finalPlaceholder.official
            : finalPlaceholder.thirdParty
        }
        disabled={disabled ?? category === "official"}
      />

      {/* 验证 Key 按钮 + 错误信息 */}
      {(renderVerifyButton() || verifyError) && (
        <div className="flex items-start gap-2 -mt-1 pl-1">
          {renderVerifyButton()}
          {verifyError && verifyStatus === "failed" && (
            <p className="text-xs text-red-500 dark:text-red-400 flex-1 pt-1.5">
              {verifyError}
            </p>
          )}
        </div>
      )}

      {/* API Key 获取链接 */}
      {shouldShowLink && websiteUrl && (
        <div className="space-y-2 -mt-1 pl-1">
          <a
            href={websiteUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs text-blue-400 dark:text-blue-500 hover:text-blue-500 dark:hover:text-blue-400 transition-colors"
          >
            {t("providerForm.getApiKey", {
              defaultValue: "获取 API Key",
            })}
          </a>

          {/* 促销信息（与 isPartner 解耦：仅凭 partnerPromotionKey 即可展示，星标仍由 isPartner 控制） */}
          {partnerPromotionKey && (
            <div className="rounded-md bg-blue-50 dark:bg-blue-950/30 p-2.5 border border-blue-200 dark:border-blue-800">
              <p className="text-xs leading-relaxed text-blue-700 dark:text-blue-300">
                💡{" "}
                {t(`providerForm.partnerPromotion.${partnerPromotionKey}`, {
                  defaultValue: "",
                })}
              </p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
