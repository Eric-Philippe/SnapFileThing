import React from "react";
import { FolderIcon, TrashIcon } from "@heroicons/react/24/outline";
import { ExplorerItem } from "./FileExplorer";

interface GridViewProps {
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
  onDeleteItem: (item: ExplorerItem) => void;
  getFileIcon: (mimeType?: string) => React.ElementType;
  formatFileSize: (bytes: number) => string;
  copyToClipboard: (text: string) => void;
}

export default function GridView({
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
  onDeleteItem,
  getFileIcon,
  formatFileSize,
  copyToClipboard,
}: GridViewProps) {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8 gap-4 p-4">
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
            className={`flex flex-col items-center p-3 rounded-lg cursor-pointer hover:bg-blue-50 relative group ${
              isSelected ? "bg-blue-100 ring-2 ring-blue-500" : ""
            } ${isDraggedOver ? "bg-green-100 ring-2 ring-green-500" : ""}`}
            onClick={(e) => !isBackFolder && onItemClick(item, e)}
            onDoubleClick={() => onItemDoubleClick(item)}
            onContextMenu={(e) => {
              if (!isBackFolder) {
                e.preventDefault();
                e.stopPropagation();
                onContextMenu(e, item);
              }
            }}
            onDragStart={(e) => onItemDragStart(e, item)}
            onDragEnd={onItemDragEnd}
            onDragOver={(e) => onItemDragOver(e, item)}
            onDragLeave={(e) => onItemDragLeave(e, item)}
            onDrop={(e) => onFolderDrop(e, item)}
          >
            {/* Action buttons - appears on hover or when selected, but not for back folder */}
            {!isBackFolder && (
              <div className="absolute top-1 right-1 flex gap-1">
                {item.type === "file" && item.urls?.original && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      copyToClipboard(item.urls!.original);
                    }}
                    className={`p-1 bg-white text-gray-600 rounded-full hover:bg-gray-200 transition-opacity border border-gray-200 shadow-sm ${
                      isSelected
                        ? "opacity-100"
                        : "opacity-0 group-hover:opacity-100"
                    }`}
                    title="Copy Direct Link"
                  >
                    {/* Use a link/copy icon, fallback to TrashIcon if not available */}
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      className="w-3 h-3"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M8 17l4 4 4-4m0-5V3m-8 4v12a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-3"
                      />
                    </svg>
                  </button>
                )}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDeleteItem(item);
                  }}
                  className={`p-1 bg-red-600 text-white rounded-full hover:bg-red-700 transition-opacity ${
                    isSelected
                      ? "opacity-100"
                      : "opacity-0 group-hover:opacity-100"
                  }`}
                  title="Delete"
                >
                  <TrashIcon className="w-3 h-3" />
                </button>
              </div>
            )}
            {/* Thumbnail or Icon */}
            <div className="w-16 h-16 flex items-center justify-center mb-2">
              {item.type === "file" && item.isImage && item.urls?.thumbnail ? (
                <img
                  src={item.urls.thumbnail}
                  alt={item.name}
                  className="w-16 h-16 object-cover rounded-md"
                />
              ) : (
                <Icon
                  className={`w-12 h-12 ${
                    item.type === "folder" || isBackFolder
                      ? "text-blue-500"
                      : "text-gray-400"
                  }`}
                />
              )}
            </div>
            {/* Name */}
            <div className="text-center">
              <p
                className={`text-xs font-medium text-gray-900 truncate w-20 ${
                  isBackFolder ? "text-blue-600 font-bold" : ""
                }`}
                title={item.name}
              >
                {item.name}
              </p>
              {item.type === "file" && item.size && (
                <p className="text-xs text-gray-500 mt-1">
                  {formatFileSize(item.size)}
                </p>
              )}
              {item.type === "folder" && !isBackFolder && (
                <p className="text-xs text-gray-500 mt-1">
                  {(item.fileCount || 0) + (item.folderCount || 0)} items
                </p>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
