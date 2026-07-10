import { useTranslation } from "react-i18next";

interface QualitySliderProps {
  value: number;
  onChange: (value: number) => void;
}

export function QualitySlider({ value, onChange }: QualitySliderProps) {
  const { t } = useTranslation();
  return (
    <div className="card p-5">
      <div className="flex items-center justify-between mb-3">
        <label
          htmlFor="quality-slider"
          className="text-sm font-semibold text-gray-700 dark:text-gray-300"
        >
          {t("settings.quality")}
        </label>
        <span className="flex h-8 w-12 items-center justify-center rounded-lg bg-primary-50 text-sm font-bold text-primary-600 dark:bg-primary-900/30 dark:text-primary-300">
          {value}
        </span>
      </div>
      <input
        id="quality-slider"
        type="range"
        min={1}
        max={100}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full h-2 cursor-pointer appearance-none rounded-full bg-gray-100 accent-primary-500 dark:bg-gray-700"
      />
      <div className="mt-1.5 flex justify-between text-xs text-gray-400">
        <span>1</span>
        <span>100</span>
      </div>
    </div>
  );
}
