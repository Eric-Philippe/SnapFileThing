import { useState, useRef, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { FileInfo } from "@/types/api";
import { filesApi } from "@/lib/api";
import {
  FileText,
  Image,
  Video,
  Music,
  Archive,
  Copy,
  Trash2,
  Download,
  Eye,
  ExternalLink,
} from "lucide-react";

interface FileGridProps {
  files: FileInfo[];
  onFileDeleted?: (filename: string) => void;
}

interface ContextMenu {
  visible: boolean;
  x: number;
  y: number;
  file: FileInfo | null;
}

export function FileGrid({ files, onFileDeleted }: FileGridProps) {
  const [selectedFile, setSelectedFile] = useState<FileInfo | null>(null);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [deleteLoading, setDeleteLoading] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [contextMenu, setContextMenu] = useState<ContextMenu>({
    visible: false,
    x: 0,
    y: 0,
    file: null,
  });
  const contextMenuRef = useRef<HTMLDivElement>(null);

  const getFileIcon = (mimeType?: string) => {
    if (!mimeType) return FileText;
    if (mimeType.startsWith("image/")) return Image;
    if (mimeType.startsWith("video/")) return Video;
    if (mimeType.startsWith("audio/")) return Music;
    if (mimeType.includes("zip") || mimeType.includes("archive"))
      return Archive;
    return FileText;
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

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      // Copy successful
    } catch {
      // Failed to copy
    }
  };

  const deleteFile = async (filename: string) => {
    if (!confirm(`Are you sure you want to delete "${filename}"?`)) {
      return;
    }

    setDeleteLoading(filename);
    setError("");

    try {
      await filesApi.deleteFile(filename);
      onFileDeleted?.(filename);
      if (selectedFile?.filename === filename) {
        setSelectedFile(null);
      }
      // Remove from selected files
      setSelectedFiles((prev) => {
        const newSet = new Set(prev);
        newSet.delete(filename);
        return newSet;
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete file");
    } finally {
      setDeleteLoading(null);
    }
  };

  const deleteSelectedFiles = async () => {
    if (selectedFiles.size === 0) return;

    const fileList = Array.from(selectedFiles);
    if (
      !confirm(
        `Are you sure you want to delete ${fileList.length} selected file(s)?`
      )
    ) {
      return;
    }

    for (const filename of fileList) {
      try {
        setDeleteLoading(filename);
        await filesApi.deleteFile(filename);
        onFileDeleted?.(filename);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : `Failed to delete ${filename}`
        );
        break;
      }
    }

    setSelectedFiles(new Set());
    setDeleteLoading(null);
  };

  const handleFileClick = (file: FileInfo, event: React.MouseEvent) => {
    if (event.ctrlKey || event.metaKey) {
      // Multi-select with Ctrl/Cmd
      setSelectedFiles((prev) => {
        const newSet = new Set(prev);
        if (newSet.has(file.filename)) {
          newSet.delete(file.filename);
        } else {
          newSet.add(file.filename);
        }
        return newSet;
      });
    } else {
      // Single select
      setSelectedFiles(new Set([file.filename]));
    }
  };

  const handleContextMenu = (event: React.MouseEvent, file: FileInfo) => {
    event.preventDefault();
    setContextMenu({
      visible: true,
      x: event.clientX,
      y: event.clientY,
      file,
    });
  };

  const closeContextMenu = () => {
    setContextMenu({ visible: false, x: 0, y: 0, file: null });
  };

  // Close context menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        contextMenuRef.current &&
        !contextMenuRef.current.contains(event.target as Node)
      ) {
        closeContextMenu();
      }
    };

    if (contextMenu.visible) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [contextMenu.visible]);

  const getThumbnailUrl = (file: FileInfo) => {
    if (file.urls.thumbnail) {
      return file.urls.thumbnail;
    }
    if (file.mime_type?.startsWith("image/")) {
      return file.urls.original;
    }
    return null;
  };

  if (files.length === 0) {
    return (
      <div className="text-center py-12">
        <FileText className="mx-auto h-12 w-12 text-gray-400" />
        <h3 className="mt-4 text-lg font-semibold text-gray-900">
          No files found
        </h3>
        <p className="mt-2 text-gray-600">Upload some files to get started!</p>
      </div>
    );
  }

  return (
    <>
      {error && (
        <Alert variant="destructive" className="mb-6">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* Selection Actions Bar */}
      {selectedFiles.size > 0 && (
        <div className="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-lg flex items-center justify-between">
          <span className="text-sm text-blue-800">
            {selectedFiles.size} file(s) selected
          </span>
          <div className="flex space-x-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setSelectedFiles(new Set())}
            >
              Clear Selection
            </Button>
            <Button
              variant="destructive"
              size="sm"
              onClick={deleteSelectedFiles}
              disabled={deleteLoading !== null}
            >
              <Trash2 className="w-4 h-4 mr-2" />
              Delete Selected
            </Button>
          </div>
        </div>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
        {files.map((file) => {
          const FileIcon = getFileIcon(file.mime_type);
          const thumbnailUrl = getThumbnailUrl(file);
          const isSelected = selectedFiles.has(file.filename);

          return (
            <Card
              key={file.filename}
              className={`group hover:shadow-lg transition-shadow cursor-pointer ${
                isSelected ? "ring-2 ring-blue-500 bg-blue-50" : ""
              }`}
              onClick={(e) => handleFileClick(file, e)}
              onContextMenu={(e) => handleContextMenu(e, file)}
            >
              <div className="aspect-square p-4 flex items-center justify-center border-b">
                {thumbnailUrl ? (
                  <img
                    src={thumbnailUrl}
                    alt={file.filename}
                    className="max-w-full max-h-full object-contain rounded"
                  />
                ) : (
                  <FileIcon className="w-12 h-12 text-gray-400" />
                )}
              </div>

              <div className="p-3">
                <h3
                  className="font-medium text-sm text-gray-900 truncate hover:text-blue-600"
                  title={file.filename}
                >
                  {file.filename}
                </h3>
                <p className="text-xs text-gray-500 mt-1">
                  {formatFileSize(file.size)}
                </p>
                <p className="text-xs text-gray-500">
                  {formatDate(file.uploaded_at)}
                </p>

                {/* Action buttons - only show on hover or when selected */}
                <div
                  className={`flex space-x-1 mt-3 transition-opacity ${
                    isSelected
                      ? "opacity-100"
                      : "opacity-0 group-hover:opacity-100"
                  }`}
                >
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      copyToClipboard(file.urls.original);
                    }}
                    title="Copy URL"
                  >
                    <Copy className="w-4 h-4" />
                  </Button>

                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedFile(file);
                    }}
                    title="View details"
                  >
                    <Eye className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            </Card>
          );
        })}
      </div>

      {/* Context Menu */}
      {contextMenu.visible && contextMenu.file && (
        <div
          ref={contextMenuRef}
          className="fixed bg-white border border-gray-200 rounded-lg shadow-lg py-2 z-50"
          style={{
            left: contextMenu.x,
            top: contextMenu.y,
          }}
        >
          <button
            className="w-full px-4 py-2 text-left hover:bg-gray-100 flex items-center space-x-2"
            onClick={() => {
              setSelectedFile(contextMenu.file);
              closeContextMenu();
            }}
          >
            <Eye className="w-4 h-4" />
            <span>View Details</span>
          </button>
          <button
            className="w-full px-4 py-2 text-left hover:bg-gray-100 flex items-center space-x-2"
            onClick={() => {
              if (contextMenu.file) {
                copyToClipboard(contextMenu.file.urls.original);
              }
              closeContextMenu();
            }}
          >
            <Copy className="w-4 h-4" />
            <span>Copy URL</span>
          </button>
          <button
            className="w-full px-4 py-2 text-left hover:bg-gray-100 flex items-center space-x-2"
            onClick={() => {
              if (contextMenu.file) {
                window.open(contextMenu.file.urls.original, "_blank");
              }
              closeContextMenu();
            }}
          >
            <Download className="w-4 h-4" />
            <span>Download</span>
          </button>
          <hr className="my-1" />
          <button
            className="w-full px-4 py-2 text-left hover:bg-red-50 text-red-600 flex items-center space-x-2"
            onClick={() => {
              if (contextMenu.file) {
                deleteFile(contextMenu.file.filename);
              }
              closeContextMenu();
            }}
            disabled={deleteLoading === contextMenu.file?.filename}
          >
            <Trash2 className="w-4 h-4" />
            <span>
              {deleteLoading === contextMenu.file?.filename
                ? "Deleting..."
                : "Delete"}
            </span>
          </button>
        </div>
      )}

      {/* File Details Modal */}
      <Dialog open={!!selectedFile} onOpenChange={() => setSelectedFile(null)}>
        <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
          {selectedFile && (
            <>
              <DialogHeader>
                <DialogTitle className="text-lg font-semibold truncate">
                  {selectedFile.filename}
                </DialogTitle>
              </DialogHeader>

              <div className="space-y-4">
                {/* File Preview */}
                <div className="flex justify-center">
                  {selectedFile.mime_type?.startsWith("image/") ? (
                    <img
                      src={selectedFile.urls.original}
                      alt={selectedFile.filename}
                      className="max-w-full max-h-96 object-contain rounded-lg"
                    />
                  ) : (
                    <div className="p-8 border rounded-lg bg-gray-50">
                      {(() => {
                        const FileIcon = getFileIcon(selectedFile.mime_type);
                        return (
                          <FileIcon className="w-16 h-16 text-gray-400 mx-auto" />
                        );
                      })()}
                      <p className="text-center text-gray-600 mt-2">
                        {selectedFile.mime_type || "Unknown type"}
                      </p>
                    </div>
                  )}
                </div>

                {/* File Info */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                  <div>
                    <strong>Filename:</strong> {selectedFile.filename}
                  </div>
                  <div>
                    <strong>Size:</strong> {formatFileSize(selectedFile.size)}
                  </div>
                  <div>
                    <strong>Type:</strong> {selectedFile.mime_type || "Unknown"}
                  </div>
                  <div>
                    <strong>Uploaded:</strong>{" "}
                    {formatDate(selectedFile.uploaded_at)}
                  </div>
                  {selectedFile.dimensions &&
                    selectedFile.dimensions.length === 2 && (
                      <div>
                        <strong>Dimensions:</strong>{" "}
                        {selectedFile.dimensions[0]} ×{" "}
                        {selectedFile.dimensions[1]}
                      </div>
                    )}
                </div>

                {/* Available URLs */}
                <div className="space-y-2">
                  <h4 className="font-semibold">Available URLs:</h4>
                  <div className="space-y-2">
                    <div className="flex items-center space-x-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() =>
                          copyToClipboard(selectedFile.urls.original)
                        }
                      >
                        <Copy className="w-4 h-4 mr-2" />
                        Copy Original URL
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() =>
                          window.open(selectedFile.urls.original, "_blank")
                        }
                      >
                        <ExternalLink className="w-4 h-4" />
                      </Button>
                    </div>

                    {selectedFile.urls.thumbnail && (
                      <div className="flex items-center space-x-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() =>
                            copyToClipboard(selectedFile.urls.thumbnail!)
                          }
                        >
                          <Copy className="w-4 h-4 mr-2" />
                          Copy Thumbnail URL
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() =>
                            window.open(selectedFile.urls.thumbnail!, "_blank")
                          }
                        >
                          <ExternalLink className="w-4 h-4" />
                        </Button>
                      </div>
                    )}

                    {selectedFile.urls.qoi && (
                      <div className="flex items-center space-x-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() =>
                            copyToClipboard(selectedFile.urls.qoi!)
                          }
                        >
                          <Copy className="w-4 h-4 mr-2" />
                          Copy QOI URL
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() =>
                            window.open(selectedFile.urls.qoi!, "_blank")
                          }
                        >
                          <ExternalLink className="w-4 h-4" />
                        </Button>
                      </div>
                    )}
                  </div>
                </div>

                {/* Actions */}
                <div className="flex justify-end space-x-2 pt-4 border-t">
                  <Button
                    variant="outline"
                    onClick={() =>
                      window.open(selectedFile.urls.original, "_blank")
                    }
                  >
                    <Download className="w-4 h-4 mr-2" />
                    Download
                  </Button>

                  <Button
                    variant="destructive"
                    onClick={() => deleteFile(selectedFile.filename)}
                    disabled={deleteLoading === selectedFile.filename}
                  >
                    <Trash2 className="w-4 h-4 mr-2" />
                    {deleteLoading === selectedFile.filename
                      ? "Deleting..."
                      : "Delete"}
                  </Button>
                </div>
              </div>
            </>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}
