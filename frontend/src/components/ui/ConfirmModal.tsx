import React from "react";
import {
  ExclamationTriangleIcon,
  TrashIcon,
} from "@heroicons/react/24/outline";

import type { ReactNode } from "react";

interface ConfirmModalProps {
  open: boolean;
  title?: string;
  message: string | ReactNode;
  destructive?: boolean;
  confirmText?: string;
  cancelText?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

const ConfirmModal: React.FC<ConfirmModalProps> = ({
  open,
  title,
  message,
  destructive = false,
  confirmText = destructive ? "Delete" : "Confirm",
  cancelText = "Cancel",
  onConfirm,
  onCancel,
}) => {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-30">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-sm mx-auto p-6 relative animate-fade-in">
        <div className="flex flex-col items-center text-center">
          <div className="mb-4">
            <span
              className={`inline-flex items-center justify-center h-14 w-14 rounded-full border-2 ${
                destructive
                  ? "border-red-200 bg-red-50"
                  : "border-yellow-200 bg-yellow-50"
              }`}
            >
              {destructive ? (
                <TrashIcon className="h-7 w-7 text-red-500" />
              ) : (
                <ExclamationTriangleIcon className="h-7 w-7 text-yellow-500" />
              )}
            </span>
          </div>
          {title && <h2 className="text-lg font-semibold mb-2">{title}</h2>}
          <div className="text-gray-700 mb-6 leading-relaxed">{message}</div>
          <div className="flex gap-3 w-full">
            <button
              className="flex-1 py-2 rounded bg-gray-100 hover:bg-gray-200 text-gray-700 font-medium transition"
              onClick={onCancel}
            >
              {cancelText}
            </button>
            <button
              className={`flex-1 py-2 rounded font-medium transition ${
                destructive
                  ? "bg-red-500 hover:bg-red-600 text-white"
                  : "bg-blue-500 hover:bg-blue-600 text-white"
              }`}
              onClick={onConfirm}
            >
              {confirmText}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ConfirmModal;
