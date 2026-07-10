import { useTranslation } from "react-i18next";
import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
} from "@headlessui/react";
import { useTheme } from "../../lib/theme";
import { changeLanguage, type SupportedLanguage } from "../../lib/i18n";

const languages: { value: SupportedLanguage; label: string }[] = [
  { value: "en", label: "EN" },
  { value: "jp", label: "JP" },
];

export function Header() {
  const { t, i18n } = useTranslation();
  const { isDark, setTheme } = useTheme();
  const currentLanguage =
    languages.find((l) => l.value === i18n.language) ?? languages[0];

  return (
    <header className="flex items-center justify-end px-5 py-3 gap-2">
      <button
        type="button"
        onClick={() => setTheme(isDark ? "light" : "dark")}
        className="flex h-8 w-8 items-center justify-center rounded-lg text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-gray-200"
        aria-label={t("header.darkMode")}
      >
        {isDark ? (
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-[18px] w-[18px]"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z"
              clipRule="evenodd"
            />
          </svg>
        ) : (
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-[18px] w-[18px]"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
          </svg>
        )}
      </button>
      <Listbox
        value={currentLanguage.value}
        onChange={(lang) => changeLanguage(lang)}
      >
        <div className="relative">
          <ListboxButton className="flex h-8 items-center rounded-lg px-2.5 text-sm font-medium text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-800 dark:hover:text-gray-200">
            {currentLanguage.label}
          </ListboxButton>
          <ListboxOptions className="absolute right-0 z-10 mt-1 w-28 rounded-xl bg-white p-1 shadow-soft-lg border border-gray-100 dark:bg-gray-800 dark:border-gray-700">
            {languages.map((lang) => (
              <ListboxOption
                key={lang.value}
                value={lang.value}
                className="cursor-pointer rounded-lg px-3 py-2 text-sm text-gray-700 data-[focus]:bg-gray-50 data-[selected]:font-semibold data-[selected]:text-primary-600 dark:text-gray-200 dark:data-[focus]:bg-gray-700"
              >
                {lang.label === "EN" ? "English" : "日本語"}
              </ListboxOption>
            ))}
          </ListboxOptions>
        </div>
      </Listbox>
    </header>
  );
}
