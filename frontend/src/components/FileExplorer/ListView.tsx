import React from "react";
import { FolderIcon, TrashIcon } from "@heroicons/react/24/outline";
import { ExplorerItem } from "./FileExplorer";

interface ListViewProps {
  items: ExplorerItem[];
  selectedItems: Set<string>;
  dragOverItem?: ExplorerItem;
  onItemClick: (item: ExplorerItem, event: React.MouseEvent) => void;
  onItemDoubleClick: (item: ExplorerItem) => void;
  onContextMenu: (event: React.MouseEvent, item: ExplorerItem) => void;
  onItemDragStart: (e: React.DragEvent, item: ExplorerItem) => void;
  onItemDragEnd: () => void;
  onItemDragOver: (e: React.DragEvent, item: ExplorerItem) => void;
  onItemDragLeave: (e: React.DragEvent, item: ExplorerItem) => void;
  onFolderDrop: (e: React.DragEvent, item: ExplorerItem) => void;
  getFileIcon: (mimeType?: string) => React.ElementType;
  formatFileSize: (bytes: number) => string;
  formatDate: (dateString: string) => string;
}

export default function ListView({
  items,
  selectedItems,
  dragOverItem,
  onItemClick,
  onItemDoubleClick,
  onContextMenu,
  onItemDragStart,
  onItemDragEnd,
  onItemDragOver,
  onItemDragLeave,
  onFolderDrop,
  getFileIcon,
  formatFileSize,
  formatDate,
}: ListViewProps) {
  return (
    <div className="divide-y divide-gray-200">
      {/* Header */}
      <div className="grid grid-cols-12 gap-4 p-4 text-xs font-medium text-gray-500 uppercase tracking-wider bg-gray-50">
        <div className="col-span-6">Name</div>
        <div className="col-span-2">Size</div>
        <div className="col-span-2">Type</div>
        <div className="col-span-1">Modified</div>
        <div className="col-span-1">Actions</div>
      </div>
      {/* Items */}
      {items.map((item) => {
        const isSelected = selectedItems.has(item.id);
        const isDraggedOver = dragOverItem?.id === item.id;
        const isBackFolder = item.id === "__back__";
        const Icon = isBackFolder
          ? FolderIcon
          : item.type === "folder"
          ? FolderIcon
          : getFileIcon(item.mimeType);
        return (
          <div
            key={item.id}
            draggable={!isBackFolder}
            className={`grid grid-cols-12 gap-4 p-4 hover:bg-gray-50 cursor-pointer group relative ${
              isSelected ? "bg-blue-50 ring-1 ring-blue-500" : ""
            } ${isDraggedOver ? "bg-green-50 ring-1 ring-green-500" : ""}`}
            onClick={(e) => !isBackFolder && onItemClick(item, e)}
            onDoubleClick={() => onItemDoubleClick(item)}
            onContextMenu={(e) => !isBackFolder && onContextMenu(e, item)}
            onDragStart={(e) => onItemDragStart(e, item)}
            onDragEnd={onItemDragEnd}
            onDragOver={(e) => onItemDragOver(e, item)}
            onDragLeave={(e) => onItemDragLeave(e, item)}
            onDrop={(e) => onFolderDrop(e, item)}
          >
            <div className="col-span-6 flex items-center space-x-3">
              <Icon
                className={`h-5 w-5 ${
                  item.type === "folder" || isBackFolder
                    ? "text-blue-500"
                    : "text-gray-400"
                }`}
              />
              <span
                className={`text-sm font-medium text-gray-900 truncate ${
                  isBackFolder ? "text-blue-600 font-bold" : ""
                }`}
              >
                {item.name}
              </span>
            </div>
            <div className="col-span-2 text-sm text-gray-500">
              {item.type === "file" && item.size
                ? formatFileSize(item.size)
                : item.type === "folder" &&
                  item.folderSize !== undefined &&
                  !isBackFolder
                ? formatFileSize(item.folderSize)
                : "—"}
            </div>
            <div className="col-span-2 text-sm text-gray-500">
              {isBackFolder
                ? "Back"
                : item.type === "folder"
                ? "Folder"
                : item.mimeType || "File"}
            </div>
            <div className="col-span-1 text-sm text-gray-500">
              {item.uploadedAt && !isBackFolder
                ? formatDate(item.uploadedAt)
                : "—"}
            </div>
            <div className="col-span-1 flex justify-end">
              {!isBackFolder && (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    // You may want to pass a delete handler as prop for better separation
                  }}
                  className={`p-1 text-red-600 hover:text-red-800 hover:bg-red-50 rounded transition-opacity ${
                    isSelected
                      ? "opacity-100"
                      : "opacity-0 group-hover:opacity-100"
                  }`}
                  title="Delete"
                >
                  <TrashIcon className="w-4 h-4" />
                </button>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
