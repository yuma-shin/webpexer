import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";

interface DropZoneProps {
  onFilesSelected: (paths: string[]) => void;
}

export function DropZone({ onFilesSelected }: DropZoneProps) {
  const { t } = useTranslation();
  const [isDragOver, setIsDragOver] = useState(false);

  useEffect(() => {
    const webview = getCurrentWebviewWindow();
    const unlisten = webview.onDragDropEvent((event) => {
      if (event.payload.type === "over") setIsDragOver(true);
      else if (event.payload.type === "drop") {
        setIsDragOver(false);
        const paths = event.payload.paths;
        if (paths && paths.length > 0) onFilesSelected(paths);
      } else if (event.payload.type === "leave") setIsDragOver(false);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [onFilesSelected]);

  const handleSelectFolder = useCallback(async () => {
    try {
      const selected = await open({ directory: true });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        if (paths.length > 0) onFilesSelected(paths);
      }
    } catch {
      // Dialog cancelled or error
    }
  }, [onFilesSelected]);

  const handleSelectFiles = useCallback(
    async (e: React.MouseEvent) => {
      e.stopPropagation();
      try {
        const selected = await open({ multiple: true, directory: false });
        if (selected) {
          const paths = Array.isArray(selected) ? selected : [selected];
          if (paths.length > 0) onFilesSelected(paths);
        }
      } catch {
        // Dialog cancelled or error
      }
    },
    [onFilesSelected],
  );

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={handleSelectFolder}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleSelectFolder();
        }
      }}
      className={`card flex w-full cursor-pointer flex-col items-center justify-center p-10 transition-all ${
        isDragOver
          ? "!border-primary-400 !bg-primary-50/80 shadow-soft-lg scale-[1.01]"
          : "hover:shadow-soft-lg hover:scale-[1.005]"
      }`}
    >
      <div
        className={`mb-4 flex h-16 w-16 items-center justify-center rounded-2xl ${
          isDragOver ? "bg-primary-100" : "bg-primary-100"
        }`}
      >
        <svg
          className={`h-8 w-8 ${isDragOver ? "text-primary-600" : "text-primary-500"}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
          />
        </svg>
      </div>
      <p
        className={`text-base font-semibold ${
          isDragOver ? "text-primary-700" : "text-gray-700 dark:text-gray-200"
        }`}
      >
        {t("file.dropzone.title")}
      </p>
      <p className="mt-1 text-sm text-gray-400 dark:text-gray-500">
        {t("file.dropzone.description")}
      </p>
      <button
        type="button"
        onClick={handleSelectFiles}
        className="mt-3 text-sm font-medium text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300 underline underline-offset-2"
      >
        {t("file.dropzone.selectFiles")}
      </button>
    </div>
  );
}
