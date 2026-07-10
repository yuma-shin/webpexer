import { useTranslation } from "react-i18next";

interface FileListProps {
  files: string[];
  onRemoveFile: (index: number) => void;
  onClearAll: () => void;
}

export function FileList({ files, onRemoveFile, onClearAll }: FileListProps) {
  const { t } = useTranslation();
  if (files.length === 0) return null;

  return (
    <div className="card overflow-hidden">
      <div className="flex items-center justify-between px-5 py-3.5 border-b border-gray-100 dark:border-gray-800">
        <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">
          {t("file.list.count", { count: files.length })}
        </span>
        <button
          type="button"
          onClick={onClearAll}
          className="text-xs font-medium text-red-500 hover:text-red-600 transition-colors"
        >
          {t("file.list.clearAll")}
        </button>
      </div>
      <ul className="max-h-48 overflow-y-auto divide-y divide-gray-50 dark:divide-gray-800">
        {files.map((filePath, index) => {
          const fileName = filePath.split(/[/\\]/).pop() || filePath;
          return (
            <li
              key={`${filePath}-${index}`}
              className="flex items-center justify-between px-5 py-2.5 hover:bg-surface-50 transition-colors"
            >
              <span
                className="truncate text-sm text-gray-600 dark:text-gray-400"
                title={filePath}
              >
                {fileName}
              </span>
              <button
                type="button"
                onClick={() => onRemoveFile(index)}
                className="ml-3 flex-shrink-0 flex h-6 w-6 items-center justify-center rounded-full text-gray-300 hover:bg-red-50 hover:text-red-500 transition-all"
                aria-label={t("file.list.remove", { fileName })}
              >
                <svg
                  className="h-3.5 w-3.5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
