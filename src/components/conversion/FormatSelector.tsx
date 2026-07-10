import { Fragment } from "react";
import { useTranslation } from "react-i18next";
import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/react";
import type { FileFormat, FormatCategory, FormatInfo } from "../../types";

/** Hardcoded format list grouped by category */
const FORMATS: FormatInfo[] = [
  // Image formats
  { format: "png", name: "PNG", extensions: [".png"], category: "image" },
  {
    format: "jpeg",
    name: "JPEG",
    extensions: [".jpg", ".jpeg"],
    category: "image",
  },
  { format: "webp", name: "WebP", extensions: [".webp"], category: "image" },
  { format: "gif", name: "GIF", extensions: [".gif"], category: "image" },
  { format: "bmp", name: "BMP", extensions: [".bmp"], category: "image" },
  {
    format: "tiff",
    name: "TIFF",
    extensions: [".tiff", ".tif"],
    category: "image",
  },
  { format: "avif", name: "AVIF", extensions: [".avif"], category: "image" },
  { format: "ico", name: "ICO", extensions: [".ico"], category: "image" },
  // Text formats
  { format: "json", name: "JSON", extensions: [".json"], category: "text" },
  {
    format: "yaml",
    name: "YAML",
    extensions: [".yaml", ".yml"],
    category: "text",
  },
  { format: "toml", name: "TOML", extensions: [".toml"], category: "text" },
  { format: "xml", name: "XML", extensions: [".xml"], category: "text" },
  { format: "csv", name: "CSV", extensions: [".csv"], category: "text" },
  {
    format: "markdown",
    name: "Markdown",
    extensions: [".md"],
    category: "text",
  },
  {
    format: "plaintext",
    name: "Plain Text",
    extensions: [".txt"],
    category: "text",
  },
];

const CATEGORIES: FormatCategory[] = ["image", "text"];

interface FormatSelectorProps {
  value: FileFormat | null;
  onChange: (format: FileFormat) => void;
}

export function FormatSelector({ value, onChange }: FormatSelectorProps) {
  const { t } = useTranslation();

  const selectedFormat = value ? FORMATS.find((f) => f.format === value) : null;

  return (
    <div className="w-full">
      <Listbox value={value ?? undefined} onChange={onChange}>
        <ListboxButton className="card relative w-full cursor-pointer !rounded-xl py-3 pl-4 pr-10 text-left text-sm transition-all hover:shadow-soft-lg">
          {selectedFormat ? (
            <span className="flex items-center gap-2">
              <span className="font-semibold text-gray-700 dark:text-gray-200">
                {selectedFormat.name}
              </span>
              <span className="text-xs text-gray-400 dark:text-gray-500">
                {selectedFormat.extensions.join(", ")}
              </span>
            </span>
          ) : (
            <span className="text-gray-400 dark:text-gray-500">
              {t("format.select")}
            </span>
          )}
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

        <ListboxOptions
          anchor="bottom start"
          className="z-50 mt-2 max-h-60 w-[var(--button-width)] overflow-auto rounded-2xl bg-white p-2 shadow-soft-lg focus:outline-none dark:bg-gray-800 border border-gray-100 dark:border-gray-700"
        >
          {CATEGORIES.map((category) => {
            const categoryFormats = FORMATS.filter(
              (f) => f.category === category,
            );
            return (
              <Fragment key={category}>
                <div className="px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-gray-400 dark:text-gray-500">
                  {t(`format.${category}`)}
                </div>
                {categoryFormats.map((format) => (
                  <ListboxOption
                    key={format.format}
                    value={format.format}
                    className="relative cursor-pointer select-none rounded-xl px-3 py-2.5 text-gray-700 data-[focus]:bg-primary-50 data-[selected]:bg-primary-50 data-[selected]:text-primary-700 dark:text-gray-200 dark:data-[focus]:bg-gray-700 dark:data-[selected]:bg-primary-900/30 dark:data-[selected]:text-primary-300"
                  >
                    <span className="flex items-center gap-2">
                      <span className="font-medium">{format.name}</span>
                      <span className="text-xs text-gray-400 dark:text-gray-500">
                        {format.extensions.join(", ")}
                      </span>
                    </span>
                  </ListboxOption>
                ))}
              </Fragment>
            );
          })}
        </ListboxOptions>
      </Listbox>
    </div>
  );
}
