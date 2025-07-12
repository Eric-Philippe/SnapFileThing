import React, { RefObject } from "react";
import {
  FolderPlusIcon,
  ArrowUpTrayIcon,
  EyeIcon,
  ClipboardDocumentIcon,
  ArrowDownTrayIcon,
  TrashIcon,
} from "@heroicons/react/24/outline";
import { ExplorerItem } from "./FileExplorer";

interface ContextMenuProps {
  visible: boolean;
  x: number;
  y: number;
  item?: ExplorerItem;
  isBackground?: boolean;
  contextMenuRef: RefObject<HTMLDivElement>;
  onOpenFolder: (folderId: string) => void;
  onOpenFile: (url: string) => void;
  onCopyLink: (url: string) => void;
  onDownload: (url: string) => void;
  onDelete: () => void;
  onNewFolder: () => void;
  onUploadFiles: () => void;
}

export default function ContextMenu({
  visible,
  x,
  y,
  item,
  isBackground,
  contextMenuRef,
  onOpenFolder,
  onOpenFile,
  onCopyLink,
  onDownload,
  onDelete,
  onNewFolder,
  onUploadFiles,
}: ContextMenuProps) {
  if (!visible) return null;
  return (
    <div
      ref={contextMenuRef}
      className="fixed bg-white rounded-md shadow-lg border py-2 z-50 min-w-[180px]"
      style={{ left: x, top: y }}
    >
      {isBackground ? (
        <>
          <button
            onClick={onNewFolder}
            className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 flex items-center"
          >
            <FolderPlusIcon className="h-4 w-4 mr-3" />
            New Folder
          </button>
          <button
            onClick={onUploadFiles}
            className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 flex items-center"
          >
            <ArrowUpTrayIcon className="h-4 w-4 mr-3" />
            Upload Files
          </button>
        </>
      ) : item ? (
        <>
          {item.type === "folder" ? (
            <button
              onClick={() => onOpenFolder(item.id)}
              className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 flex items-center"
            >
              <EyeIcon className="h-4 w-4 mr-3" />
              Open
            </button>
          ) : (
            <>
              <button
                onClick={() => item.urls && onOpenFile(item.urls.original)}
                className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 flex items-center"
              >
                <EyeIcon className="h-4 w-4 mr-3" />
                Open
              </button>
              <button
                onClick={() => item.urls && onCopyLink(item.urls.original)}
                className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 flex items-center"
              >
                <ClipboardDocumentIcon className="h-4 w-4 mr-3" />
                Copy Link
              </button>
              <button
                onClick={() => item.urls && onDownload(item.urls.original)}
                className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 flex items-center"
              >
                <ArrowDownTrayIcon className="h-4 w-4 mr-3" />
                Download
              </button>
            </>
          )}
          <hr className="my-1" />
          <button
            onClick={onDelete}
            className="w-full text-left px-4 py-2 text-sm hover:bg-red-50 text-red-600 flex items-center"
          >
            <TrashIcon className="h-4 w-4 mr-3" />
            Delete
          </button>
        </>
      ) : null}
    </div>
  );
}
