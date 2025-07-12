import React from "react";

interface CreateFolderFormProps {
  value: string;
  creating: boolean;
  onChange: (value: string) => void;
  onCreate: () => void;
  onCancel: () => void;
}

export default function CreateFolderForm({
  value,
  creating,
  onChange,
  onCreate,
  onCancel,
}: CreateFolderFormProps) {
  return (
    <div className="absolute top-16 left-4 right-4 bg-white border rounded-lg shadow-lg p-4 z-10">
      <div className="flex items-center space-x-3">
        <input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder="Folder name"
          className="flex-1 min-w-0 block px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
          onKeyPress={(e) => e.key === "Enter" && onCreate()}
          autoFocus
        />
        <button
          onClick={onCreate}
          disabled={creating || !value.trim()}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {creating ? "Creating..." : "Create"}
        </button>
        <button
          onClick={onCancel}
          className="inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
