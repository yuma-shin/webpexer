import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useFileSelection } from "../hooks/useFileSelection";
import { useSettings } from "../hooks/useSettings";
import { useConversion } from "../hooks/useConversion";
import { useHistory } from "../hooks/useHistory";
import { DropZone } from "../components/file/DropZone";
import { FileList } from "../components/file/FileList";
import { FormatSelector } from "../components/conversion/FormatSelector";
import { QualitySlider } from "../components/conversion/QualitySlider";
import { ResizeOptions } from "../components/conversion/ResizeOptions";
import { OutputSettings } from "../components/conversion/OutputSettings";
import { ConvertButton } from "../components/conversion/ConvertButton";
import { ProgressDetail } from "../components/progress/ProgressDetail";
import { HistoryPanel } from "../components/history/HistoryPanel";
import type { ConversionParams, FileFormat, HistoryEntry } from "../types";
import type { Notification } from "../components/layout/NotificationArea";

/** Lossy image formats where quality setting applies */
const LOSSY_FORMATS: FileFormat[] = ["jpeg", "webp", "avif"];

/** All image formats */
const IMAGE_FORMATS: FileFormat[] = [
  "png",
  "jpeg",
  "webp",
  "gif",
  "bmp",
  "tiff",
  "avif",
  "ico",
];

interface MainViewProps {
  onNotification: (notification: Omit<Notification, "id">) => void;
}

export function MainView({ onNotification }: MainViewProps) {
  const { t } = useTranslation();

  // --- Hooks ---
  const { files, addFiles, removeFile, clearFiles } = useFileSelection();
  const { settings, updateSetting, isLoaded: settingsLoaded } = useSettings();
  const {
    status,
    progress,
    result,
    error: conversionError,
    startConversion,
    cancel,
    reset,
  } = useConversion();
  const { entries, clearHistory, restoreSettings } = useHistory();

  // --- Derived state ---
  const hasFiles = files.length > 0;
  const isConverting = status === "converting";
  const isLossyFormat =
    settings.outputFormat !== null &&
    LOSSY_FORMATS.includes(settings.outputFormat);
  const isImageFormat =
    settings.outputFormat !== null &&
    IMAGE_FORMATS.includes(settings.outputFormat);
  const canConvert =
    hasFiles && settings.outputFormat !== null && !isConverting;

  // --- Handle conversion completion ---
  useEffect(() => {
    if (status === "completed" && result) {
      const message = t("notifications.success", {
        count: result.successCount,
      });
      onNotification({
        type: result.failedCount > 0 ? "info" : "success",
        message,
      });
      reset();
    }
  }, [status, result, onNotification, reset, t]);

  // --- Handle conversion error ---
  useEffect(() => {
    if (status === "error" && conversionError) {
      onNotification({
        type: "error",
        message: conversionError,
      });
      reset();
    }
  }, [status, conversionError, onNotification, reset]);

  // --- Handle conversion cancelled ---
  useEffect(() => {
    if (status === "cancelled") {
      onNotification({
        type: "info",
        message: t("notifications.cancelled", {
          completed: progress?.processedCount ?? 0,
        }),
      });
      reset();
    }
  }, [status, onNotification, reset, t]);

  // --- Start conversion ---
  const handleStartConversion = useCallback(() => {
    if (!settings.outputFormat || files.length === 0) return;

    const params: ConversionParams = {
      inputPaths: files,
      outputFormat: settings.outputFormat,
      outputDir: settings.outputDir,
      options: {
        quality: settings.quality,
        resize: settings.resize,
        encoding: null,
        csvDelimiter: null,
        fileNaming: settings.fileNaming,
        conflictResolution: settings.conflictResolution,
        deleteSource: settings.deleteSource,
      },
    };

    startConversion(params);
  }, [files, settings, startConversion]);

  // --- History restore ---
  const handleHistoryRestore = useCallback(
    (entry: HistoryEntry) => {
      const restored = restoreSettings(entry);
      updateSetting("outputFormat", restored.outputFormat);
      updateSetting("deleteSource", restored.deleteSource);
    },
    [restoreSettings, updateSetting],
  );

  // Don't render until settings are loaded
  if (!settingsLoaded) {
    return null;
  }

  return (
    <div className="flex flex-1 gap-6">
      {/* Main content area */}
      <div className="flex flex-1 flex-col space-y-5">
        {/* DropZone */}
        <DropZone onFilesSelected={addFiles} />

        {/* File list (visible when files are selected) */}
        {hasFiles && (
          <>
            <FileList
              files={files}
              onRemoveFile={removeFile}
              onClearAll={clearFiles}
            />

            {/* Format selector */}
            <div className="card p-5">
              <label className="mb-2 block text-sm font-semibold text-gray-700 dark:text-gray-300">
                {t("format.select")}
              </label>
              <FormatSelector
                value={settings.outputFormat}
                onChange={(format) => updateSetting("outputFormat", format)}
              />
            </div>

            {/* Quality slider (only for lossy formats) */}
            {isLossyFormat && (
              <QualitySlider
                value={settings.quality}
                onChange={(val) => updateSetting("quality", val)}
              />
            )}

            {/* Resize options (only for image formats) */}
            {isImageFormat && (
              <ResizeOptions
                value={settings.resize}
                onChange={(val) => updateSetting("resize", val)}
              />
            )}

            {/* Output settings */}
            <OutputSettings
              outputDir={settings.outputDir}
              onOutputDirChange={(dir) => updateSetting("outputDir", dir)}
              fileNaming={settings.fileNaming}
              onFileNamingChange={(pattern) =>
                updateSetting("fileNaming", pattern)
              }
              conflictResolution={settings.conflictResolution}
              onConflictResolutionChange={(resolution) =>
                updateSetting("conflictResolution", resolution)
              }
              deleteSource={settings.deleteSource}
              onDeleteSourceChange={(enabled) =>
                updateSetting("deleteSource", enabled)
              }
            />

            {/* Convert button or Progress detail */}
            {isConverting && progress ? (
              <ProgressDetail
                currentFile={progress.currentFile}
                processedCount={progress.processedCount}
                totalCount={progress.totalCount}
                onCancel={cancel}
              />
            ) : (
              <ConvertButton
                disabled={!canConvert}
                onClick={handleStartConversion}
                isConverting={isConverting}
              />
            )}
          </>
        )}
      </div>

      {/* History sidebar */}
      <aside className="hidden w-80 flex-shrink-0 lg:block">
        <HistoryPanel
          entries={entries}
          onRestore={handleHistoryRestore}
          onClear={clearHistory}
        />
      </aside>
    </div>
  );
}
