import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  useRef,
} from "react";
import Toast from "./Toast";

export type ToastType = "info" | "success" | "warning" | "error";

export interface ToastOptions {
  type?: ToastType;
  message: string;
  duration?: number; // ms
}

interface ToastContextProps {
  showToast: (options: ToastOptions) => void;
}

const ToastContext = createContext<ToastContextProps>({ showToast: () => {} });

export const useToast = () => useContext(ToastContext);

let toastId = 0;

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [toasts, setToasts] = useState<
    {
      id: number;
      type: ToastType;
      message: string;
    }[]
  >([]);
  const timers = useRef<{ [id: number]: NodeJS.Timeout }>({});

  const removeToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
    if (timers.current[id]) {
      clearTimeout(timers.current[id]);
      delete timers.current[id];
    }
  }, []);

  const showToast = useCallback(
    (options: ToastOptions) => {
      const id = ++toastId;
      const toast = {
        id,
        type: options.type || "info",
        message: options.message,
      };
      setToasts((prev) => [...prev, toast]);
      timers.current[id] = setTimeout(
        () => removeToast(id),
        options.duration || 3500
      );
    },
    [removeToast]
  );

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}
      <div className="fixed top-4 right-4 z-[100] flex flex-col gap-2 items-end">
        {toasts.map((toast) => (
          <Toast
            key={toast.id}
            type={toast.type}
            message={toast.message}
            onClose={() => removeToast(toast.id)}
          />
        ))}
      </div>
    </ToastContext.Provider>
  );
};
