import React, { useState, useEffect, useRef, useCallback } from "react";
import Toolbar from "./Toolbar";
import Breadcrumbs from "./Breadcrumbs";
import CreateFolderForm from "./CreateFolderForm";
import ContextMenu from "./ContextMenu";
import FileUrlsModal from "./FileUrlsModal";
import GridView from "./GridView";
import ListView from "./ListView";
import ConfirmModal from "../ui/ConfirmModal";
import { ToastProvider, useToast } from "../ui/ToastProvider";
// Confirm modal state type
type ConfirmModalState = {
  open: boolean;
  message: string | React.ReactNode;
  destructive?: boolean;
  onConfirm: (() => void) | null;
};
import {
  FolderIcon,
  DocumentIcon,
  PhotoIcon,
  MusicalNoteIcon,
  VideoCameraIcon,
  ArchiveBoxIcon,
  ArrowUpTrayIcon,
} from "@heroicons/react/24/outline";
import type {
  FolderInfo,
  CreateFolderRequest,
  MoveFileRequest,
  MoveFolderRequest,
} from "../../types/api";
import { filesApi, foldersApi, uploadApi } from "../../lib/api";

interface DesktopFileExplorerProps {
  currentFolderId?: string;
  onFolderChange: (folderId?: string) => void;
  onError: (error: string) => void;
}

export interface ExplorerItem {
  id: string;
  name: string;
  type: "folder" | "file";
  size?: number;
  mimeType?: string;
  uploadedAt?: string;
  isImage?: boolean;
  urls?: {
    original: string;
    thumbnail?: string;
    qoi?: string;
  };
  fileCount?: number;
  folderCount?: number;
  folderSize?: number;
}

interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  item?: ExplorerItem;
  isBackground?: boolean;
}

interface ExplorerState {
  items: ExplorerItem[];
  currentFolder?: FolderInfo;
  breadcrumbs: FolderInfo[];
  loading: boolean;
  creating: boolean;
  newFolderName: string;
  showCreateForm: boolean;
  selectedItems: Set<string>;
  contextMenu: ContextMenuState;
  viewMode: "grid" | "list";
  draggedItem?: ExplorerItem;
  dragOverItem?: ExplorerItem;
}

function FileExplorerInner({
  currentFolderId,
  onFolderChange,
  onError,
}: DesktopFileExplorerProps) {
  const { showToast } = useToast();
  const [showUrlsModal, setShowUrlsModal] = useState<ExplorerItem | null>(null);

  // Confirm modal state
  const [confirmModal, setConfirmModal] = useState<ConfirmModalState>({
    open: false,
    message: "",
    destructive: false,
    onConfirm: null,
  });

  const handleShowUrls = (item: ExplorerItem) => {
    setShowUrlsModal(item);
    setState((prev) => ({
      ...prev,
      contextMenu: { ...prev.contextMenu, visible: false },
    }));
  };

  const [state, setState] = useState<ExplorerState>({
    items: [],
    breadcrumbs: [],
    loading: false,
    creating: false,
    newFolderName: "",
    showCreateForm: false,
    selectedItems: new Set(),
    contextMenu: { visible: false, x: 0, y: 0 },
    viewMode: "grid",
    draggedItem: undefined,
    dragOverItem: undefined,
  });

  const fileInputRef = useRef<HTMLInputElement>(null);
  const contextMenuRef = useRef<HTMLDivElement>(null);
  const [dragOver, setDragOver] = useState(false);

  const loadContent = useCallback(
    async (folderId?: string) => {
      setState((prev) => ({ ...prev, loading: true }));
      try {
        // Load both folders and files
        const [foldersResponse, filesResponse] = await Promise.all([
          foldersApi.listFolders(folderId),
          filesApi.listFiles(0, 100, folderId), // Load more items for desktop view
        ]);

        // Combine folders and files into a single array
        const folderItems: ExplorerItem[] = foldersResponse.folders.map(
          (folder) => ({
            id: folder.id,
            name: folder.name,
            type: "folder" as const,
            fileCount: folder.file_count,
            folderCount: folder.folder_count,
            folderSize: folder.size,
          })
        );

        const fileItems: ExplorerItem[] = filesResponse.files.map((file) => ({
          id: file.filename,
          name: file.filename,
          type: "file" as const,
          size: file.size,
          mimeType: file.mime_type,
          uploadedAt: file.uploaded_at,
          isImage: file.is_image,
          urls: file.urls,
        }));

        // Sort: folders first, then files, both alphabetically
        const allItems = [
          ...folderItems.sort((a, b) => a.name.localeCompare(b.name)),
          ...fileItems.sort((a, b) => a.name.localeCompare(b.name)),
        ];

        // Add back folder if we're not at root
        const finalItems: ExplorerItem[] = [];
        if (foldersResponse.current_folder) {
          finalItems.push({
            id: "__back__",
            name: "../",
            type: "folder" as const,
            fileCount: 0,
            folderCount: 0,
            folderSize: 0,
          });
        }
        finalItems.push(...allItems);

        setState((prev) => ({
          ...prev,
          items: finalItems,
          currentFolder: foldersResponse.current_folder,
          breadcrumbs: foldersResponse.breadcrumbs,
          loading: false,
          selectedItems: new Set(), // Clear selection when folder changes
        }));
      } catch (error) {
        onError(
          `Failed to load content: ${
            error instanceof Error ? error.message : "Unknown error"
          }`
        );
        setState((prev) => ({ ...prev, loading: false }));
      }
    },
    [onError]
  );

  useEffect(() => {
    loadContent(currentFolderId);
  }, [currentFolderId, loadContent]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Ctrl/Cmd + A - Select all (excluding back folder)
      if ((event.ctrlKey || event.metaKey) && event.key === "a") {
        event.preventDefault();
        const allIds = state.items
          .filter((item) => item.id !== "__back__")
          .map((item) => item.id);
        setState((prev) => ({ ...prev, selectedItems: new Set(allIds) }));
      }

      // Delete key - Delete selected items (excluding back folder)
      if (event.key === "Delete" && state.selectedItems.size > 0) {
        event.preventDefault();
        const selectedItems = state.items.filter(
          (item) => state.selectedItems.has(item.id) && item.id !== "__back__"
        );
        if (selectedItems.length === 1) {
          handleDeleteItem(selectedItems[0]);
        } else if (selectedItems.length > 1) {
          handleDeleteMultipleItems(selectedItems);
        }
      }

      // Escape key - Clear selection and close context menu
      if (event.key === "Escape") {
        setState((prev) => ({
          ...prev,
          selectedItems: new Set(),
          contextMenu: { ...prev.contextMenu, visible: false },
          showCreateForm: false,
        }));
      }

      // F2 key - Start rename (we'll show create folder form for now)
      if (event.key === "F2") {
        event.preventDefault();
        setState((prev) => ({ ...prev, showCreateForm: true }));
      }

      // Ctrl/Cmd + V - Upload from clipboard
      if ((event.ctrlKey || event.metaKey) && event.key === "v") {
        // Skip clipboard paste for now
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [state.items, state.selectedItems.size]);

  // Handle clicks outside context menu
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        contextMenuRef.current &&
        !contextMenuRef.current.contains(event.target as Node)
      ) {
        setState((prev) => ({
          ...prev,
          contextMenu: { ...prev.contextMenu, visible: false },
        }));
      }
    };

    if (state.contextMenu.visible) {
      document.addEventListener("mousedown", handleClickOutside);
      return () =>
        document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [state.contextMenu.visible]);

  const getFileIcon = (mimeType?: string) => {
    if (!mimeType) return DocumentIcon;
    if (mimeType.startsWith("image/")) return PhotoIcon;
    if (mimeType.startsWith("video/")) return VideoCameraIcon;
    if (mimeType.startsWith("audio/")) return MusicalNoteIcon;
    if (mimeType.includes("zip") || mimeType.includes("archive"))
      return ArchiveBoxIcon;
    return DocumentIcon;
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const handleItemClick = (item: ExplorerItem, event: React.MouseEvent) => {
    // Don't allow selection of back folder
    if (item.id === "__back__") {
      return;
    }

    // Handle selection
    if (event.ctrlKey || event.metaKey) {
      // Multi-select
      setState((prev) => {
        const newSelected = new Set(prev.selectedItems);
        if (newSelected.has(item.id)) {
          newSelected.delete(item.id);
        } else {
          newSelected.add(item.id);
        }
        return { ...prev, selectedItems: newSelected };
      });
    } else if (event.shiftKey && state.selectedItems.size > 0) {
      // Range select (simplified implementation)
      setState((prev) => ({ ...prev, selectedItems: new Set([item.id]) }));
    } else {
      // Single select
      setState((prev) => ({ ...prev, selectedItems: new Set([item.id]) }));
    }
  };

  const handleItemDoubleClick = (item: ExplorerItem) => {
    if (item.id === "__back__") {
      // Navigate to parent folder (the parent of the current folder)
      const parentFolderId = state.currentFolder?.parent_id;
      onFolderChange(parentFolderId);
    } else if (item.type === "folder") {
      onFolderChange(item.id);
    } else if (item.type === "file" && item.urls) {
      // Open file in new tab
      window.open(item.urls.original, "_blank");
    }
  };

  const handleContextMenu = (event: React.MouseEvent, item?: ExplorerItem) => {
    event.preventDefault();

    // Don't show context menu for back folder
    if (item?.id === "__back__") {
      return;
    }

    setState((prev) => ({
      ...prev,
      contextMenu: {
        visible: true,
        x: event.clientX,
        y: event.clientY,
        item,
        isBackground: !item,
      },
    }));
  };

  const handleBackgroundClick = (event: React.MouseEvent) => {
    if (event.target === event.currentTarget) {
      setState((prev) => ({
        ...prev,
        selectedItems: new Set(),
        contextMenu: { ...prev.contextMenu, visible: false },
      }));
    }
  };

  const handleBreadcrumbClick = (folderId?: string) => {
    onFolderChange(folderId);
  };

  const handleCreateFolder = async () => {
    if (!state.newFolderName.trim()) return;

    setState((prev) => ({ ...prev, creating: true }));
    try {
      const request: CreateFolderRequest = {
        name: state.newFolderName.trim(),
        parent_id: currentFolderId,
      };
      await foldersApi.createFolder(request);
      setState((prev) => ({
        ...prev,
        creating: false,
        newFolderName: "",
        showCreateForm: false,
        contextMenu: { ...prev.contextMenu, visible: false },
      }));
      loadContent(currentFolderId);
      showToast({
        type: "success",
        message: `Folder "${request.name}" created successfully!`,
      });
    } catch (error) {
      onError(
        `Failed to create folder: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
      setState((prev) => ({ ...prev, creating: false }));
      showToast({
        type: "error",
        message: `Failed to create folder.`,
      });
    }
  };

  const handleDeleteItem = (item: ExplorerItem) => {
    if (item.id === "__back__") return;
    const confirmMessage =
      item.type === "folder"
        ? `Are you sure you want to delete the folder "${item.name}"? This action cannot be undone.`
        : `Are you sure you want to delete this file ? This action cannot be undone.`;
    setConfirmModal({
      open: true,
      message: (
        <>
          Are you sure you want to delete the {item.type} <b>{item.name}</b>?
          This action cannot be undone.
        </>
      ),
      destructive: true,
      onConfirm: async () => {
        try {
          if (item.type === "folder") {
            await foldersApi.deleteFolder(item.id);
          } else {
            await filesApi.deleteFile(item.id);
          }
          setState((prev) => ({
            ...prev,
            selectedItems: new Set(),
            contextMenu: { ...prev.contextMenu, visible: false },
          }));
          loadContent(currentFolderId);
          showToast({
            type: "success",
            message: `${item.type === "folder" ? "Folder" : "File"} deleted.`,
          });
        } catch (error) {
          onError(
            `Failed to delete ${item.type}: ${
              error instanceof Error ? error.message : "Unknown error"
            }`
          );
          showToast({
            type: "error",
            message: `Failed to delete ${item.type}.`,
          });
        } finally {
          setConfirmModal((prev) => ({ ...prev, open: false }));
        }
      },
    });
  };

  const handleDeleteMultipleItems = (items: ExplorerItem[]) => {
    const itemsToDelete = items.filter((item) => item.id !== "__back__");
    if (itemsToDelete.length === 0) return;
    const confirmMessage = `Are you sure you want to delete ${itemsToDelete.length} items? This action cannot be undone.`;
    setConfirmModal({
      open: true,
      message: (
        <>
          Are you sure you want to delete <b>{itemsToDelete.length} items</b>?
          This action cannot be undone.
        </>
      ),
      destructive: true,
      onConfirm: async () => {
        try {
          for (const item of itemsToDelete) {
            if (item.type === "folder") {
              await foldersApi.deleteFolder(item.id);
            } else {
              await filesApi.deleteFile(item.id);
            }
          }
          setState((prev) => ({
            ...prev,
            selectedItems: new Set(),
            contextMenu: { ...prev.contextMenu, visible: false },
          }));
          loadContent(currentFolderId);
          showToast({
            type: "success",
            message: `${itemsToDelete.length} items deleted.`,
          });
        } catch (error) {
          onError(
            `Failed to delete items: ${
              error instanceof Error ? error.message : "Unknown error"
            }`
          );
          showToast({
            type: "error",
            message: `Failed to delete items.`,
          });
        } finally {
          setConfirmModal((prev) => ({ ...prev, open: false }));
        }
      },
    });
  };
  // ConfirmModal handler
  const handleConfirmModalCancel = () => {
    setConfirmModal((prev) => ({ ...prev, open: false }));
  };

  const handleFileUpload = async (files: FileList) => {
    if (!files.length) return;

    try {
      const fileArray = Array.from(files);

      for (const file of fileArray) {
        await uploadApi.uploadFile(file, currentFolderId);
      }

      setState((prev) => ({
        ...prev,
        contextMenu: { ...prev.contextMenu, visible: false },
      }));

      loadContent(currentFolderId);
      onError(""); // Clear any previous errors
      showToast({
        type: "success",
        message: `${
          fileArray.length > 1 ? `${fileArray.length} files` : `File`
        } uploaded successfully!`,
      });
    } catch (error) {
      onError(
        `Failed to upload files: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
      showToast({
        type: "error",
        message: `Failed to upload files.`,
      });
    }
  };

  // Drag and drop handlers for moving items
  const handleItemDragStart = (e: React.DragEvent, item: ExplorerItem) => {
    if (item.id === "__back__") {
      e.preventDefault();
      return;
    }
    e.dataTransfer.setData("text/plain", item.id);
    e.dataTransfer.effectAllowed = "move";
    setState((prev) => ({ ...prev, draggedItem: item }));
  };

  const handleItemDragEnd = () => {
    setState((prev) => ({
      ...prev,
      draggedItem: undefined,
      dragOverItem: undefined,
    }));
  };

  const handleItemDragOver = (e: React.DragEvent, targetItem: ExplorerItem) => {
    if (targetItem.id === "__back__") {
      // Allow dropping on back folder to move to parent
      if (state.draggedItem) {
        e.preventDefault();
        e.stopPropagation();
        setState((prev) => ({ ...prev, dragOverItem: targetItem }));
      }
    } else if (
      targetItem.type === "folder" &&
      state.draggedItem &&
      state.draggedItem.id !== targetItem.id
    ) {
      e.preventDefault();
      e.stopPropagation();
      setState((prev) => ({ ...prev, dragOverItem: targetItem }));
    }
  };

  const handleItemDragLeave = (
    e: React.DragEvent,
    targetItem: ExplorerItem
  ) => {
    e.preventDefault();
    e.stopPropagation();
    if (state.dragOverItem?.id === targetItem.id) {
      setState((prev) => ({ ...prev, dragOverItem: undefined }));
    }
  };

  const handleItemDrop = async (
    draggedItem: ExplorerItem,
    targetFolderId: string | null
  ) => {
    if (!draggedItem || draggedItem.id === "__back__") return;

    try {
      if (draggedItem.type === "file") {
        const request: MoveFileRequest = {
          folder_id: targetFolderId || undefined,
        };
        await filesApi.moveFile(draggedItem.id, request);
      } else if (draggedItem.type === "folder") {
        const request: MoveFolderRequest = {
          parent_id: targetFolderId || undefined,
        };
        await foldersApi.moveFolder(draggedItem.id, request);
      }

      // Reload content to reflect changes
      loadContent(currentFolderId);
      onError(""); // Clear any previous errors
      showToast({
        type: "success",
        message: `${
          draggedItem.type === "folder" ? "Folder" : "File"
        } moved successfully!`,
      });
    } catch (error) {
      onError(
        `Failed to move ${draggedItem.type}: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
      showToast({
        type: "error",
        message: `Failed to move ${draggedItem.type}.`,
      });
    }
  };

  const handleFolderDrop = (e: React.DragEvent, targetFolder: ExplorerItem) => {
    e.preventDefault();
    e.stopPropagation();

    if (state.draggedItem && state.draggedItem.id !== targetFolder.id) {
      if (targetFolder.id === "__back__") {
        // Move to parent folder (the parent of the current folder)
        const parentFolderId = state.currentFolder?.parent_id;
        handleItemDrop(state.draggedItem, parentFolderId || null);
      } else {
        handleItemDrop(state.draggedItem, targetFolder.id);
      }
    }

    setState((prev) => ({ ...prev, dragOverItem: undefined }));
  };

  // Background drag and drop handlers for file uploads
  const handleDragOverBackground = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();

    // Check if we're dragging files from outside or items within the app
    const hasFiles = e.dataTransfer.types.includes("Files");
    const hasText = e.dataTransfer.types.includes("text/plain");

    if (hasFiles) {
      setDragOver(true);
      e.dataTransfer.dropEffect = "copy";
    } else if (hasText && state.draggedItem) {
      e.dataTransfer.dropEffect = "move";
    }
  };

  const handleDragLeaveBackground = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    // Only hide drag over if we're leaving the main container
    if (e.currentTarget === e.target) {
      setDragOver(false);
    }
  };

  const handleDropOnBackground = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);

    // Check if we're dropping files from outside the browser
    const files = e.dataTransfer.files;
    if (files.length > 0) {
      handleFileUpload(files);
      return;
    }

    // Check if we're dropping an item from within the app to move to current folder
    if (state.draggedItem) {
      handleItemDrop(state.draggedItem, currentFolderId || null);
    }

    setState((prev) => ({
      ...prev,
      draggedItem: undefined,
      dragOverItem: undefined,
    }));
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setState((prev) => ({
        ...prev,
        contextMenu: { ...prev.contextMenu, visible: false },
      }));
      showToast({
        type: "success",
        message: "Link copied to clipboard!",
      });
    } catch {
      // Clipboard operation failed
      showToast({
        type: "error",
        message: "Failed to copy link.",
      });
    }
  };

  return (
    <div className="relative bg-white rounded-lg shadow-sm border h-full flex flex-col">
      {/* Confirm Modal */}
      <ConfirmModal
        open={confirmModal.open}
        message={confirmModal.message}
        destructive={confirmModal.destructive}
        onConfirm={() => {
          if (confirmModal.onConfirm) confirmModal.onConfirm();
        }}
        onCancel={handleConfirmModalCancel}
      />
      <Toolbar
        selectedCount={state.selectedItems.size}
        onNewFolder={() =>
          setState((prev) => ({ ...prev, showCreateForm: true }))
        }
        onUploadFiles={() => fileInputRef.current?.click()}
        onClearSelection={() =>
          setState((prev) => ({ ...prev, selectedItems: new Set() }))
        }
        onDeleteSelected={() => {
          const selectedItemsArray = state.items.filter(
            (item) => state.selectedItems.has(item.id) && item.id !== "__back__"
          );
          if (selectedItemsArray.length === 1) {
            handleDeleteItem(selectedItemsArray[0]);
          } else if (selectedItemsArray.length > 1) {
            handleDeleteMultipleItems(selectedItemsArray);
          }
        }}
        onExportAll={async () => {
          try {
            await filesApi.exportAllFiles();
            showToast({
              type: "info",
              message:
                "Export started. You will be notified when your files are ready.",
            });
          } catch (error: any) {
            onError(error?.message || "Failed to export files");
            showToast({
              type: "error",
              message: "Failed to export files.",
            });
          }
        }}
        onImport={() => {
          // Open file dialog for ZIP import
          const input = document.createElement("input");
          input.type = "file";
          input.accept = ".zip,application/zip";
          input.onchange = async (e: any) => {
            const file = e.target.files?.[0];
            if (!file) return;
            try {
              const res = await filesApi.importFiles(
                file,
                state.currentFolder?.id
              );
              showToast({
                type: "success",
                message: res.message || "Import successful!",
              });
            } catch (error: any) {
              onError(error?.message || "Failed to import files");
              showToast({
                type: "error",
                message: error?.message || "Failed to import files.",
              });
            }
          };
          input.click();
        }}
        viewMode={state.viewMode}
        onToggleView={() =>
          setState((prev) => ({
            ...prev,
            viewMode: prev.viewMode === "grid" ? "list" : "grid",
          }))
        }
      />
      <input
        ref={fileInputRef}
        type="file"
        multiple
        className="hidden"
        onChange={(e) => e.target.files && handleFileUpload(e.target.files)}
      />
      <Breadcrumbs
        breadcrumbs={state.breadcrumbs}
        onClick={handleBreadcrumbClick}
      />
      {state.showCreateForm && (
        <CreateFolderForm
          value={state.newFolderName}
          creating={state.creating}
          onChange={(value) =>
            setState((prev) => ({ ...prev, newFolderName: value }))
          }
          onCreate={handleCreateFolder}
          onCancel={() =>
            setState((prev) => ({
              ...prev,
              showCreateForm: false,
              newFolderName: "",
            }))
          }
        />
      )}

      {/* Main Content */}
      <div
        className={`flex-1 overflow-auto relative ${
          dragOver ? "bg-blue-50 border-2 border-dashed border-blue-300" : ""
        }`}
        onClick={handleBackgroundClick}
        onContextMenu={(e) => handleContextMenu(e)}
        onDragOver={handleDragOverBackground}
        onDragLeave={handleDragLeaveBackground}
        onDrop={handleDropOnBackground}
      >
        {dragOver && (
          <div className="absolute inset-0 bg-blue-50 bg-opacity-75 flex items-center justify-center z-20 pointer-events-none">
            <div className="text-center">
              <ArrowUpTrayIcon className="h-16 w-16 text-blue-500 mx-auto mb-4" />
              <p className="text-xl font-semibold text-blue-700">
                Drop files here to upload
              </p>
            </div>
          </div>
        )}
        {state.loading ? (
          <div className="flex items-center justify-center h-64">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
            <p className="ml-3 text-sm text-gray-600">Loading...</p>
          </div>
        ) : state.items.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-gray-500">
            <FolderIcon className="h-16 w-16 mb-4" />
            <p className="text-lg font-medium">This folder is empty</p>
            <p className="text-sm">
              Right-click to create folders or upload files
            </p>
          </div>
        ) : null}
        {state.viewMode === "grid" ? (
          <GridView
            items={state.items}
            selectedItems={state.selectedItems}
            dragOverItem={state.dragOverItem}
            onItemClick={handleItemClick}
            onItemDoubleClick={handleItemDoubleClick}
            onContextMenu={handleContextMenu}
            onItemDragStart={handleItemDragStart}
            onItemDragEnd={handleItemDragEnd}
            onItemDragOver={handleItemDragOver}
            onItemDragLeave={handleItemDragLeave}
            onFolderDrop={handleFolderDrop}
            onDeleteItem={handleDeleteItem}
            getFileIcon={getFileIcon}
            formatFileSize={formatFileSize}
            copyToClipboard={copyToClipboard}
          />
        ) : (
          <ListView
            items={state.items}
            selectedItems={state.selectedItems}
            dragOverItem={state.dragOverItem}
            onItemClick={handleItemClick}
            onItemDoubleClick={handleItemDoubleClick}
            onContextMenu={handleContextMenu}
            onItemDragStart={handleItemDragStart}
            onItemDragEnd={handleItemDragEnd}
            onItemDragOver={handleItemDragOver}
            onItemDragLeave={handleItemDragLeave}
            onFolderDrop={handleFolderDrop}
            onShowUrls={handleShowUrls}
            onDelete={handleDeleteItem}
            getFileIcon={getFileIcon}
            formatFileSize={formatFileSize}
            formatDate={formatDate}
            copyToClipboard={copyToClipboard}
          />
        )}
      </div>

      <ContextMenu
        visible={state.contextMenu.visible}
        x={state.contextMenu.x}
        y={state.contextMenu.y}
        item={state.contextMenu.item}
        isBackground={state.contextMenu.isBackground}
        contextMenuRef={contextMenuRef}
        onOpenFolder={onFolderChange}
        onOpenFile={(url) => window.open(url, "_blank")}
        onCopyLink={copyToClipboard}
        onDownload={(url) => window.open(url, "_blank")}
        onDelete={() =>
          state.contextMenu.item && handleDeleteItem(state.contextMenu.item)
        }
        onNewFolder={() =>
          setState((prev) => ({ ...prev, showCreateForm: true }))
        }
        onUploadFiles={() => fileInputRef.current?.click()}
        onShowUrls={handleShowUrls}
      />

      {showUrlsModal && (
        <FileUrlsModal
          file={showUrlsModal}
          onClose={() => setShowUrlsModal(null)}
        />
      )}
    </div>
  );
}

export default function FileExplorer(props: DesktopFileExplorerProps) {
  return (
    <ToastProvider>
      <FileExplorerInner {...props} />
    </ToastProvider>
  );
}
