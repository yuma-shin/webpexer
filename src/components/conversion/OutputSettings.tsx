import { useTranslation } from "react-i18next";
import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/react";
import { selectFolder } from "../../lib/tauri-commands";
import type { FileNamingPattern, ConflictResolution } from "../../types";

interface OutputSettingsProps {
  outputDir: string | null;
  onOutputDirChange: (dir: string | null) => void;
  fileNaming: FileNamingPattern;
  onFileNamingChange: (pattern: FileNamingPattern) => void;
  conflictResolution: ConflictResolution;
  onConflictResolutionChange: (resolution: ConflictResolution) => void;
  deleteSource: boolean;
  onDeleteSourceChange: (enabled: boolean) => void;
}

const NAMING_OPTIONS: { value: FileNamingPattern; labelKey: string }[] = [
  { value: "original", labelKey: "settings.namingOriginal" },
  { value: "original_sequential", labelKey: "settings.namingSequential" },
  { value: "original_datetime", labelKey: "settings.namingDatetime" },
];

const CONFLICT_OPTIONS: { value: ConflictResolution; labelKey: string }[] = [
  { value: "overwrite", labelKey: "settings.conflictOverwrite" },
  { value: "skip", labelKey: "settings.conflictSkip" },
  { value: "rename", labelKey: "settings.conflictRename" },
];

export function OutputSettings({
  outputDir,
  onOutputDirChange,
  fileNaming,
  onFileNamingChange,
  conflictResolution,
  onConflictResolutionChange,
  deleteSource,
  onDeleteSourceChange,
}: OutputSettingsProps) {
  const { t } = useTranslation();

  const handleBrowse = async () => {
    try {
      const folder = await selectFolder();
      if (folder) {
        onOutputDirChange(folder);
      }
    } catch {
      // Dialog cancelled or error
    }
  };

  const selectedNaming =
    NAMING_OPTIONS.find((o) => o.value === fileNaming) ?? NAMING_OPTIONS[0];

  const selectedConflict =
    CONFLICT_OPTIONS.find((o) => o.value === conflictResolution) ??
    CONFLICT_OPTIONS[0];

  return (
    <div className="card p-5 space-y-4">
      {/* Output folder */}
      <div>
        <label className="mb-1.5 block text-sm font-semibold text-gray-700 dark:text-gray-300">
          {t("settings.output")}
        </label>
        <div className="flex gap-2">
          <input
            type="text"
            readOnly
            value={outputDir ?? ""}
            placeholder={t("settings.outputDefault")}
            className="min-w-0 flex-1 rounded-xl border border-gray-100 bg-surface-50 px-3.5 py-2.5 text-sm text-gray-700 placeholder-gray-400 focus:border-primary-300 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200 dark:placeholder-gray-500"
          />
          <button
            type="button"
            onClick={handleBrowse}
            className="inline-flex items-center rounded-xl bg-primary-600 hover:bg-primary-700 px-4 py-2.5 text-sm font-semibold text-white shadow-soft transition-all hover:shadow-soft-lg"
          >
            {t("settings.browse")}
          </button>
        </div>
      </div>

      {/* File naming pattern */}
      <div>
        <label className="mb-1.5 block text-sm font-semibold text-gray-700 dark:text-gray-300">
          {t("settings.naming")}
        </label>
        <Listbox value={fileNaming} onChange={onFileNamingChange}>
          <div className="relative">
            <ListboxButton className="relative w-full cursor-pointer rounded-xl border border-gray-100 bg-surface-50 py-2.5 pl-3.5 pr-10 text-left text-sm transition-all hover:border-primary-200 focus:border-primary-300 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200">
              <span>{t(selectedNaming.labelKey)}</span>
              <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
                <svg
                  className="h-4 w-4 text-gray-400"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <path
                    fillRule="evenodd"
                    d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
                    clipRule="evenodd"
                  />
                </svg>
              </span>
            </ListboxButton>
            <ListboxOptions className="absolute z-10 mt-2 max-h-60 w-full overflow-auto rounded-2xl bg-white p-2 shadow-soft-lg focus:outline-none dark:bg-gray-800">
              {NAMING_OPTIONS.map((option) => (
                <ListboxOption
                  key={option.value}
                  value={option.value}
                  className="cursor-pointer rounded-xl px-3 py-2.5 text-sm text-gray-700 data-[focus]:bg-primary-50 data-[selected]:bg-primary-50 data-[selected]:text-primary-700 dark:text-gray-200 dark:data-[focus]:bg-gray-700 dark:data-[selected]:bg-primary-900/30 dark:data-[selected]:text-primary-300"
                >
                  {t(option.labelKey)}
                </ListboxOption>
              ))}
            </ListboxOptions>
          </div>
        </Listbox>
      </div>

      {/* Conflict resolution */}
      <div>
        <label className="mb-1.5 block text-sm font-semibold text-gray-700 dark:text-gray-300">
          {t("settings.conflict")}
        </label>
        <Listbox
          value={conflictResolution}
          onChange={onConflictResolutionChange}
        >
          <div className="relative">
            <ListboxButton className="relative w-full cursor-pointer rounded-xl border border-gray-100 bg-surface-50 py-2.5 pl-3.5 pr-10 text-left text-sm transition-all hover:border-primary-200 focus:border-primary-300 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200">
              <span>{t(selectedConflict.labelKey)}</span>
              <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
                <svg
                  className="h-4 w-4 text-gray-400"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <path
                    fillRule="evenodd"
                    d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
                    clipRule="evenodd"
                  />
                </svg>
              </span>
            </ListboxButton>
            <ListboxOptions className="absolute z-10 mt-2 max-h-60 w-full overflow-auto rounded-2xl bg-white p-2 shadow-soft-lg focus:outline-none dark:bg-gray-800">
              {CONFLICT_OPTIONS.map((option) => (
                <ListboxOption
                  key={option.value}
                  value={option.value}
                  className="cursor-pointer rounded-xl px-3 py-2.5 text-sm text-gray-700 data-[focus]:bg-primary-50 data-[selected]:bg-primary-50 data-[selected]:text-primary-700 dark:text-gray-200 dark:data-[focus]:bg-gray-700 dark:data-[selected]:bg-primary-900/30 dark:data-[selected]:text-primary-300"
                >
                  {t(option.labelKey)}
                </ListboxOption>
              ))}
            </ListboxOptions>
          </div>
        </Listbox>
      </div>

      {/* Delete source files */}
      <label className="flex cursor-pointer items-center gap-2.5">
        <input
          type="checkbox"
          checked={deleteSource}
          onChange={(e) => onDeleteSourceChange(e.target.checked)}
          className="h-4 w-4 rounded border-gray-300 text-primary-500 focus:ring-primary-400 dark:border-gray-600 dark:bg-gray-700"
        />
        <span className="text-sm text-gray-700 dark:text-gray-300">
          {t("settings.deleteSource")}
        </span>
      </label>
    </div>
  );
}
