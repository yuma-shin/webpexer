import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { ResizeOptions as ResizeOptionsType } from "../../types";

interface ResizeOptionsProps {
  value: ResizeOptionsType | null;
  onChange: (options: ResizeOptionsType | null) => void;
}

export function ResizeOptions({ value, onChange }: ResizeOptionsProps) {
  const { t } = useTranslation();
  const [enabled, setEnabled] = useState(value !== null);

  const handleToggle = () => {
    if (enabled) {
      setEnabled(false);
      onChange(null);
    } else {
      setEnabled(true);
      onChange({ width: null, height: null, maintainAspectRatio: true });
    }
  };

  const handleWidthChange = (raw: string) => {
    const num = raw === "" ? null : Math.min(16383, Math.max(1, Number(raw)));
    onChange({
      width: num,
      height: value?.height ?? null,
      maintainAspectRatio: value?.maintainAspectRatio ?? true,
    });
  };

  const handleHeightChange = (raw: string) => {
    const num = raw === "" ? null : Math.min(16383, Math.max(1, Number(raw)));
    onChange({
      width: value?.width ?? null,
      height: num,
      maintainAspectRatio: value?.maintainAspectRatio ?? true,
    });
  };

  const handleAspectRatioChange = (checked: boolean) => {
    onChange({
      width: value?.width ?? null,
      height: value?.height ?? null,
      maintainAspectRatio: checked,
    });
  };

  return (
    <div className="card p-5">
      {/* Toggle */}
      <label className="flex cursor-pointer items-center gap-2.5">
        <input
          type="checkbox"
          checked={enabled}
          onChange={handleToggle}
          className="h-4 w-4 rounded border-gray-300 text-primary-500 focus:ring-primary-400 dark:border-gray-600 dark:bg-gray-700"
        />
        <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">
          {t("settings.resize")}
        </span>
      </label>

      {/* Options (visible when enabled) */}
      {enabled && (
        <div className="mt-4 space-y-3 rounded-xl border border-primary-100 bg-surface-50 p-4 dark:border-gray-700 dark:bg-gray-800/50">
          <div className="flex gap-3">
            {/* Width */}
            <div className="flex-1">
              <label
                htmlFor="resize-width"
                className="mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400"
              >
                {t("settings.width")}
              </label>
              <input
                id="resize-width"
                type="number"
                min={1}
                max={16383}
                placeholder="—"
                value={value?.width ?? ""}
                onChange={(e) => handleWidthChange(e.target.value)}
                className="w-full rounded-xl border border-gray-100 bg-white px-3 py-2 text-sm text-gray-700 placeholder-gray-400 focus:border-primary-300 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 dark:placeholder-gray-500"
              />
            </div>
            {/* Height */}
            <div className="flex-1">
              <label
                htmlFor="resize-height"
                className="mb-1 block text-xs font-medium text-gray-500 dark:text-gray-400"
              >
                {t("settings.height")}
              </label>
              <input
                id="resize-height"
                type="number"
                min={1}
                max={16383}
                placeholder="—"
                value={value?.height ?? ""}
                onChange={(e) => handleHeightChange(e.target.value)}
                className="w-full rounded-xl border border-gray-100 bg-white px-3 py-2 text-sm text-gray-700 placeholder-gray-400 focus:border-primary-300 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 dark:placeholder-gray-500"
              />
            </div>
          </div>

          {/* Aspect ratio */}
          <label className="flex cursor-pointer items-center gap-2">
            <input
              type="checkbox"
              checked={value?.maintainAspectRatio ?? true}
              onChange={(e) => handleAspectRatioChange(e.target.checked)}
              className="h-4 w-4 rounded border-gray-300 text-primary-500 focus:ring-primary-400 dark:border-gray-600 dark:bg-gray-700"
            />
            <span className="text-xs text-gray-600 dark:text-gray-400">
              {t("settings.maintainAspectRatio")}
            </span>
          </label>
        </div>
      )}
    </div>
  );
}
