import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogPanel,
  DialogTitle,
  Description,
} from "@headlessui/react";
import type { LargeFileInfo } from "../../types";

interface LargeFileDialogProps {
  isOpen: boolean;
  files: LargeFileInfo[];
  onContinue: () => void;
  onSkip: () => void;
  onClose: () => void;
}

/**
 * 100MB超ファイルの確認ダイアログ
 *
 * バッチ変換開始前に大容量ファイルが検出された場合に表示し、
 * ユーザーに続行かスキップかの選択を促す。
 */
export function LargeFileDialog({
  isOpen,
  files,
  onContinue,
  onSkip,
  onClose,
}: LargeFileDialogProps) {
  const { t } = useTranslation();

  /**
   * ファイルサイズを人間が読みやすい形式にフォーマットする
   */
  function formatSize(sizeMb: number): string {
    if (sizeMb >= 1024) {
      return `${(sizeMb / 1024).toFixed(1)} GB`;
    }
    return `${sizeMb.toFixed(1)} MB`;
  }

  return (
    <Dialog open={isOpen} onClose={onClose} className="relative z-50">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/30 dark:bg-black/50"
        aria-hidden="true"
      />

      {/* Full-screen container to center the panel */}
      <div className="fixed inset-0 flex items-center justify-center p-4">
        <DialogPanel className="w-full max-w-md rounded-lg border border-gray-200 bg-white p-6 shadow-elevation-3 dark:border-gray-700 dark:bg-gray-800">
          {/* Warning icon + Title */}
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-amber-100 dark:bg-amber-900/30">
              <svg
                className="h-5 w-5 text-amber-600 dark:text-amber-400"
                viewBox="0 0 20 20"
                fill="currentColor"
                aria-hidden="true"
              >
                <path
                  fillRule="evenodd"
                  d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.168 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z"
                  clipRule="evenodd"
                />
              </svg>
            </div>
            <DialogTitle className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {t("largeFile.title", "Large Files Detected")}
            </DialogTitle>
          </div>

          {/* Description */}
          <Description className="mt-3 text-sm text-gray-600 dark:text-gray-400">
            {t(
              "largeFile.description",
              "The following files exceed 100MB. Large files may take longer to convert.",
            )}
          </Description>

          {/* File list */}
          <ul className="mt-4 max-h-48 space-y-2 overflow-y-auto" role="list">
            {files.map((file) => (
              <li
                key={file.name}
                className="flex items-center justify-between rounded-md bg-gray-50 px-3 py-2 dark:bg-gray-700/50"
              >
                <span className="truncate text-sm font-medium text-gray-800 dark:text-gray-200">
                  {file.name}
                </span>
                <span className="ml-3 flex-shrink-0 text-sm text-amber-600 dark:text-amber-400">
                  {formatSize(file.sizeMb)}
                </span>
              </li>
            ))}
          </ul>

          {/* Action buttons */}
          <div className="mt-6 flex gap-3">
            <button
              type="button"
              onClick={onSkip}
              className="flex-1 rounded-md border border-gray-200 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm transition-colors hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-primary-500 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
            >
              {t("largeFile.skip", "Skip Large Files")}
            </button>
            <button
              type="button"
              onClick={onContinue}
              className="flex-1 rounded-md bg-primary-600 px-4 py-2 text-sm font-medium text-white shadow-sm transition-colors hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 dark:bg-primary-500 dark:hover:bg-primary-600 dark:focus:ring-offset-gray-800"
            >
              {t("largeFile.continue", "Continue")}
            </button>
          </div>
        </DialogPanel>
      </div>
    </Dialog>
  );
}
