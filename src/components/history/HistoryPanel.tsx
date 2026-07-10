import { useTranslation } from "react-i18next";
import type { HistoryEntry } from "../../types";
import { HistoryItem } from "./HistoryItem";

interface HistoryPanelProps {
  entries: HistoryEntry[];
  onRestore: (entry: HistoryEntry) => void;
  onClear: () => void;
}

export function HistoryPanel({
  entries,
  onRestore,
  onClear,
}: HistoryPanelProps) {
  const { t } = useTranslation();

  return (
    <div className="card overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-gray-100 px-5 py-3.5 dark:border-gray-800">
        <h2 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
          {t("history.title")}
        </h2>
        {entries.length > 0 && (
          <button
            type="button"
            onClick={onClear}
            className="text-xs font-medium text-red-500 hover:text-red-600 transition-colors"
          >
            {t("history.clear")}
          </button>
        )}
      </div>

      {/* Content */}
      {entries.length === 0 ? (
        <div className="px-5 py-10 text-center">
          <p className="text-sm text-gray-400 dark:text-gray-500">
            {t("history.empty")}
          </p>
        </div>
      ) : (
        <ul className="max-h-80 overflow-y-auto divide-y divide-gray-50 dark:divide-gray-800">
          {entries.map((entry) => (
            <li key={entry.id}>
              <HistoryItem entry={entry} onRestore={onRestore} />
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
