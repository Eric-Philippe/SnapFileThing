import React from "react";
import {
  FolderPlusIcon,
  ArrowUpTrayIcon,
  TrashIcon,
} from "@heroicons/react/24/outline";

interface ToolbarProps {
  selectedCount: number;
  onNewFolder: () => void;
  onUploadFiles: () => void;
  onClearSelection: () => void;
  onDeleteSelected: () => void;
  onExportAll: () => void;
  onImport: () => void;
  viewMode: "grid" | "list";
  onToggleView: () => void;
}

export default function Toolbar({
  selectedCount,
  onNewFolder,
  onUploadFiles,
  onClearSelection,
  onDeleteSelected,
  onExportAll,
  onImport,
  viewMode,
  onToggleView,
}: ToolbarProps) {
  return (
    <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between p-4 border-b bg-white gap-2 sm:gap-0">
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-4 w-full sm:w-auto">
        <button
          onClick={onNewFolder}
          className="inline-flex items-center justify-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 w-full sm:w-auto"
        >
          <FolderPlusIcon className="h-4 w-4 mr-2" />
          New Folder
        </button>
        <button
          onClick={onUploadFiles}
          className="inline-flex items-center justify-center px-3 py-2 border border-gray-300 text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 w-full sm:w-auto"
        >
          <ArrowUpTrayIcon className="h-4 w-4 mr-2" />
          Upload Files
        </button>
      </div>
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-4 w-full sm:w-auto justify-end">
        {selectedCount > 0 && (
          <>
            <span className="text-sm text-gray-600 w-full sm:w-auto">
              {selectedCount} item{selectedCount !== 1 ? "s" : ""} selected
            </span>
            <button
              onClick={onClearSelection}
              className="inline-flex items-center justify-center px-3 py-2 border border-gray-300 text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 w-full sm:w-auto"
            >
              Clear Selection
            </button>
            <button
              onClick={onDeleteSelected}
              className="inline-flex items-center justify-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 w-full sm:w-auto"
            >
              <TrashIcon className="h-4 w-4 mr-2" />
              Delete Selected
            </button>
          </>
        )}
        <div className="flex gap-2">
          <button
            onClick={onExportAll}
            className="inline-flex items-center justify-center px-3 py-2 border border-green-500 text-sm leading-4 font-medium rounded-md text-green-700 bg-white hover:bg-green-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 w-full sm:w-auto"
          >
            Export All
          </button>
          <button
            onClick={onImport}
            className="inline-flex items-center justify-center px-3 py-2 border border-blue-500 text-sm leading-4 font-medium rounded-md text-blue-700 bg-white hover:bg-blue-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 w-full sm:w-auto"
          >
            Import ZIP
          </button>
        </div>
        <button
          onClick={onToggleView}
          className="hidden sm:block p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-md w-full sm:w-auto"
        >
          {viewMode === "grid" ? "List" : "Grid"}
        </button>
      </div>
    </div>
  );
}
