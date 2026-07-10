import { useTranslation } from "react-i18next";

interface ConvertButtonProps {
  disabled: boolean;
  onClick: () => void;
  isConverting: boolean;
}

export function ConvertButton({
  disabled,
  onClick,
  isConverting,
}: ConvertButtonProps) {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      disabled={disabled || isConverting}
      onClick={onClick}
      className="inline-flex w-full items-center justify-center gap-2.5 rounded-2xl bg-primary-600 hover:bg-primary-700 px-6 py-4 text-base font-bold text-white shadow-soft transition-all hover:shadow-soft-lg hover:scale-[1.01] active:scale-[0.99] disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:scale-100 disabled:hover:shadow-soft"
    >
      {isConverting && (
        <svg className="h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle
            className="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            strokeWidth="4"
          />
          <path
            className="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          />
        </svg>
      )}
      {isConverting ? t("conversion.cancel") : t("conversion.start")}
    </button>
  );
}
