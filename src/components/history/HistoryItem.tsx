import { useTranslation } from "react-i18next";
import type { HistoryEntry } from "../../types";

interface HistoryItemProps {
  entry: HistoryEntry;
  onRestore: (entry: HistoryEntry) => void;
}

export function HistoryItem({ entry, onRestore }: HistoryItemProps) {
  const { t } = useTranslation();

  const formattedDate = formatDateTime(entry.executedAt);
  const folderName =
    entry.sourceFolder.split(/[/\\]/).pop() || entry.sourceFolder;

  return (
    <button
      type="button"
      onClick={() => onRestore(entry)}
      className="w-full text-left px-5 py-3 hover:bg-surface-50 dark:hover:bg-gray-800/50 transition-colors focus:outline-none focus:bg-surface-50 dark:focus:bg-gray-800/50"
      title={t("history.restore")}
    >
      <div className="flex items-center justify-between gap-2">
        {/* Left: folder path + format badge */}
        <div className="flex-1 min-w-0">
          <p
            className="text-sm font-medium text-gray-700 dark:text-gray-300 truncate"
            title={entry.sourceFolder}
          >
            {folderName}
          </p>
          <div className="mt-1 flex items-center gap-2">
            <span className="inline-flex items-center rounded-lg bg-primary-50 px-2 py-0.5 text-xs font-semibold text-primary-600 dark:bg-primary-900/30 dark:text-primary-300">
              {entry.outputFormat.toUpperCase()}
            </span>
            <span className="text-xs text-gray-400 dark:text-gray-500">
              {entry.fileCount} {entry.fileCount === 1 ? "file" : "files"}
            </span>
          </div>
        </div>

        {/* Right: datetime + status */}
        <div className="flex-shrink-0 text-right">
          <p className="text-xs text-gray-400 dark:text-gray-500">
            {formattedDate}
          </p>
          <StatusBadge status={entry.status} />
        </div>
      </div>
    </button>
  );
}

function StatusBadge({ status }: { status: "success" | "failed" }) {
  if (status === "success") {
    return (
      <span className="mt-1 inline-flex items-center rounded-lg bg-green-50 px-2 py-0.5 text-xs font-semibold text-green-600 dark:bg-green-900/30 dark:text-green-300">
        Success
      </span>
    );
  }

  return (
    <span className="mt-1 inline-flex items-center rounded-lg bg-red-50 px-2 py-0.5 text-xs font-semibold text-red-600 dark:bg-red-900/30 dark:text-red-300">
      Failed
    </span>
  );
}

function formatDateTime(isoString: string): string {
  try {
    const date = new Date(isoString);
    return date.toLocaleString(undefined, {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return isoString;
  }
}
