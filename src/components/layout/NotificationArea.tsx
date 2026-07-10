import { Transition } from "@headlessui/react";

export interface Notification {
  id: string;
  type: "success" | "error" | "info";
  message: string;
}

interface NotificationAreaProps {
  notifications: Notification[];
  onDismiss: (id: string) => void;
}

const typeStyles: Record<
  Notification["type"],
  { container: string; icon: string }
> = {
  success: {
    container: "border-l-4 border-green-400 bg-white/90 dark:bg-gray-800/90",
    icon: "text-green-500",
  },
  error: {
    container: "border-l-4 border-red-400 bg-white/90 dark:bg-gray-800/90",
    icon: "text-red-500",
  },
  info: {
    container: "border-l-4 border-primary-400 bg-white/90 dark:bg-gray-800/90",
    icon: "text-primary-500",
  },
};

export function NotificationArea({
  notifications,
  onDismiss,
}: NotificationAreaProps) {
  return (
    <div className="pointer-events-none fixed bottom-4 right-4 z-50 flex flex-col gap-3">
      {notifications.map((notification) => {
        const styles = typeStyles[notification.type];

        return (
          <Transition
            key={notification.id}
            appear
            show={true}
            enter="transition-all duration-300 ease-out"
            enterFrom="translate-x-full opacity-0"
            enterTo="translate-x-0 opacity-100"
            leave="transition-all duration-200 ease-in"
            leaveFrom="translate-x-0 opacity-100"
            leaveTo="translate-x-full opacity-0"
          >
            <div
              className={`pointer-events-auto flex w-80 items-start gap-3 rounded-2xl p-4 shadow-soft-lg backdrop-blur-sm ${styles.container}`}
            >
              {/* Icon */}
              <div className={`mt-0.5 flex-shrink-0 ${styles.icon}`}>
                {notification.type === "success" && (
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fillRule="evenodd"
                      d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                      clipRule="evenodd"
                    />
                  </svg>
                )}
                {notification.type === "error" && (
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fillRule="evenodd"
                      d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
                      clipRule="evenodd"
                    />
                  </svg>
                )}
                {notification.type === "info" && (
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    className="h-5 w-5"
                    viewBox="0 0 20 20"
                    fill="currentColor"
                  >
                    <path
                      fillRule="evenodd"
                      d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                      clipRule="evenodd"
                    />
                  </svg>
                )}
              </div>

              {/* Message */}
              <p className="flex-1 text-sm text-gray-700 dark:text-gray-300">
                {notification.message}
              </p>

              {/* Dismiss button */}
              <button
                type="button"
                onClick={() => onDismiss(notification.id)}
                className="flex-shrink-0 rounded-full p-1 text-gray-400 transition-colors hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-700 dark:hover:text-gray-200"
                aria-label="Dismiss"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  className="h-4 w-4"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fillRule="evenodd"
                    d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                    clipRule="evenodd"
                  />
                </svg>
              </button>
            </div>
          </Transition>
        );
      })}
    </div>
  );
}
