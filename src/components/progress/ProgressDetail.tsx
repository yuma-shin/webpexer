import { useTranslation } from "react-i18next";
import { ProgressBar } from "./ProgressBar";

interface ProgressDetailProps {
  currentFile: string;
  processedCount: number;
  totalCount: number;
  onCancel: () => void;
}

export function ProgressDetail({
  currentFile,
  processedCount,
  totalCount,
  onCancel,
}: ProgressDetailProps) {
  const { t } = useTranslation();

  return (
    <div className="card w-full space-y-3 p-5">
      {/* Progress bar */}
      <ProgressBar processedCount={processedCount} totalCount={totalCount} />

      {/* Current file name */}
      <p
        className="truncate text-sm text-gray-500 dark:text-gray-400"
        title={currentFile}
      >
        {currentFile}
      </p>

      {/* Cancel button */}
      <button
        type="button"
        onClick={onCancel}
        className="rounded-xl bg-red-500 px-5 py-2.5 text-sm font-semibold text-white shadow-soft transition-all hover:bg-red-600 hover:shadow-soft-lg"
      >
        {t("conversion.cancel")}
      </button>
    </div>
  );
}
