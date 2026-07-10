import { useTranslation } from "react-i18next";

interface ProgressBarProps {
  processedCount: number;
  totalCount: number;
}

export function ProgressBar({ processedCount, totalCount }: ProgressBarProps) {
  const { t } = useTranslation();

  const percentage =
    totalCount > 0 ? Math.round((processedCount / totalCount) * 100) : 0;

  return (
    <div className="w-full">
      <div className="mb-1.5 flex items-center justify-between text-sm">
        <span className="text-gray-700 dark:text-gray-300">
          {t("progress.processing", {
            current: processedCount,
            total: totalCount,
          })}
        </span>
        <span className="font-bold text-primary-600 dark:text-primary-300">
          {percentage}%
        </span>
      </div>
      <div
        className="h-3 w-full overflow-hidden rounded-full bg-gray-100 dark:bg-gray-700"
        role="progressbar"
        aria-valuenow={percentage}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div
          className="h-full rounded-full bg-primary-500 transition-all duration-300 ease-in-out"
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
}
