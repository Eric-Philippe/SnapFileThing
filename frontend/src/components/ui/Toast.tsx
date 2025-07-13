import React from "react";
import {
  CheckCircleIcon,
  ExclamationTriangleIcon,
  XCircleIcon,
  InformationCircleIcon,
  XMarkIcon,
} from "@heroicons/react/24/outline";
import type { ToastType } from "./ToastProvider";

const icons = {
  success: CheckCircleIcon,
  error: XCircleIcon,
  warning: ExclamationTriangleIcon,
  info: InformationCircleIcon,
};

const colors = {
  success: "bg-green-50 border-green-200 text-green-800",
  error: "bg-red-50 border-red-200 text-red-800",
  warning: "bg-yellow-50 border-yellow-200 text-yellow-800",
  info: "bg-blue-50 border-blue-200 text-blue-800",
};

interface ToastProps {
  type: ToastType;
  message: string;
  onClose: () => void;
}

const Toast: React.FC<ToastProps> = ({ type, message, onClose }) => {
  const Icon = icons[type] || InformationCircleIcon;
  return (
    <div
      className={`flex items-center gap-3 px-4 py-3 rounded-lg border shadow-md min-w-[220px] max-w-xs ${colors[type]}`}
      role="alert"
    >
      <span className="flex items-center justify-center h-8 w-8 rounded-full border-2 bg-white border-opacity-30">
        <Icon
          className={`h-6 w-6 ${
            type === "error"
              ? "text-red-500"
              : type === "success"
              ? "text-green-500"
              : type === "warning"
              ? "text-yellow-500"
              : "text-blue-500"
          }`}
        />
      </span>
      <span className="flex-1 text-sm font-medium">{message}</span>
      <button
        className="ml-2 p-1 rounded hover:bg-black/10 transition"
        onClick={onClose}
        aria-label="Close"
      >
        <XMarkIcon className="h-5 w-5 text-gray-400" />
      </button>
    </div>
  );
};

export default Toast;
