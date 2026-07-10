import { useEffect, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { initI18n } from "./lib/i18n";
import { Header } from "./components/layout/Header";
import {
  NotificationArea,
  type Notification,
} from "./components/layout/NotificationArea";
import { MainView } from "./pages/MainView";

function App() {
  const [i18nReady, setI18nReady] = useState(false);
  const [startupError, setStartupError] = useState<string | null>(null);
  const [notifications, setNotifications] = useState<Notification[]>([]);

  useEffect(() => {
    initI18n()
      .then(() => setI18nReady(true))
      .catch((err) => {
        console.error("Failed to initialize i18n:", err);
        setStartupError(err instanceof Error ? err.message : String(err));
      });
  }, []);

  const dismissNotification = useCallback((id: string) => {
    setNotifications((prev) => prev.filter((n) => n.id !== id));
  }, []);

  const addNotification = useCallback(
    (notification: Omit<Notification, "id">) => {
      const id = `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
      setNotifications((prev) => [...prev, { ...notification, id }]);
      setTimeout(() => {
        setNotifications((prev) => prev.filter((n) => n.id !== id));
      }, 5000);
    },
    [],
  );

  if (startupError) {
    return <StartupErrorScreen error={startupError} />;
  }

  if (!i18nReady) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-3 border-primary-200 border-t-primary-600" />
      </div>
    );
  }

  return (
    <div className="flex min-h-screen flex-col">
      {/* Ambient background */}
      <div className="fixed inset-0 -z-10 overflow-hidden">
        <div className="absolute -top-40 -right-40 h-80 w-80 rounded-full bg-primary-200/30 blur-3xl dark:bg-primary-500/10" />
        <div className="absolute top-1/3 -left-20 h-60 w-60 rounded-full bg-primary-100/40 blur-3xl dark:bg-primary-600/5" />
        <div className="absolute -bottom-20 right-1/4 h-72 w-72 rounded-full bg-primary-200/20 blur-3xl dark:bg-primary-400/5" />
        <div className="absolute inset-0 bg-gray-50/80 dark:bg-gray-950/90" />
      </div>
      <Header />
      <main className="flex flex-1 flex-col px-6 py-4">
        <MainView onNotification={addNotification} />
      </main>
      <NotificationArea
        notifications={notifications}
        onDismiss={dismissNotification}
      />
    </div>
  );
}

function StartupErrorScreen({ error }: { error: string }) {
  const { t, ready } = useTranslation(undefined, { useSuspense: false });
  const title = ready ? t("errors.startup") : "Startup Error / 起動エラー";
  const description = ready
    ? t("errors.startupDescription")
    : "The application failed to initialize. Please restart the app.";
  const restartLabel = ready ? t("errors.restart") : "Restart / 再起動";

  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-4 p-8">
      <div className="card p-8 text-center max-w-md">
        <div className="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-red-100">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-7 w-7 text-red-500"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
              clipRule="evenodd"
            />
          </svg>
        </div>
        <h1 className="text-xl font-bold text-gray-900 dark:text-white">
          {title}
        </h1>
        <p className="mt-2 text-sm text-gray-500">{description}</p>
        <details className="mt-3 text-xs text-gray-400">
          <summary className="cursor-pointer">Details</summary>
          <pre className="mt-1 whitespace-pre-wrap break-all rounded-xl bg-gray-50 p-3 dark:bg-gray-800">
            {error}
          </pre>
        </details>
        <button
          type="button"
          onClick={() => window.location.reload()}
          className="mt-5 rounded-xl bg-primary-600 hover:bg-primary-700 px-6 py-2.5 text-sm font-semibold text-white shadow-soft transition-all hover:shadow-soft-lg"
        >
          {restartLabel}
        </button>
      </div>
    </div>
  );
}

export default App;
