import { useCallback, useState } from "react";
import { Upload, FileText, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { uploadApi } from "@/lib/api";
import { UploadResponse } from "@/types/api";

interface FileDropZoneProps {
  onUploadComplete?: (results: UploadResponse[]) => void;
  selectedFolderId?: string;
}

interface FileWithPreview {
  id: string;
  file: File;
  preview?: string;
  name: string;
  size: number;
  type: string;
}

export function FileDropZone({
  onUploadComplete,
  selectedFolderId,
}: FileDropZoneProps) {
  const [isDragOver, setIsDragOver] = useState(false);
  const [files, setFiles] = useState<FileWithPreview[]>([]);
  const [uploading, setUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [currentFileProgress, setCurrentFileProgress] = useState(0);
  const [currentFileName, setCurrentFileName] = useState("");
  const [error, setError] = useState("");
  const [uploadResults, setUploadResults] = useState<UploadResponse[]>([]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);

    const droppedFiles = Array.from(e.dataTransfer.files);
    addFiles(droppedFiles);
  }, []);

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files) {
        const selectedFiles = Array.from(e.target.files);
        addFiles(selectedFiles);
      }
    },
    []
  );

  const addFiles = (newFiles: File[]) => {
    const filesWithId: FileWithPreview[] = newFiles.map((file) => ({
      id: Math.random().toString(36).substr(2, 9),
      file: file,
      preview: file.type.startsWith("image/")
        ? URL.createObjectURL(file)
        : undefined,
      name: file.name,
      size: file.size,
      type: file.type,
    }));

    setFiles((prev) => [...prev, ...filesWithId]);
    setError("");
  };

  const removeFile = (id: string) => {
    setFiles((prev) => {
      const fileItem = prev.find((f) => f.id === id);
      if (fileItem?.preview) {
        URL.revokeObjectURL(fileItem.preview);
      }
      return prev.filter((f) => f.id !== id);
    });
  };

  const clearFiles = () => {
    files.forEach((fileItem) => {
      if (fileItem.preview) {
        URL.revokeObjectURL(fileItem.preview);
      }
    });
    setFiles([]);
    setUploadResults([]);
    setError("");
  };

  const uploadFiles = async () => {
    if (files.length === 0) return;

    setUploading(true);
    setError("");
    setUploadResults([]);

    try {
      const results = await uploadApi.uploadMultipleFiles(
        files.map((f) => f.file),
        selectedFolderId,
        (overall: number, current: number, fileName: string) => {
          setUploadProgress(overall);
          setCurrentFileProgress(current);
          setCurrentFileName(fileName);
        }
      );

      setUploadResults(results);
      onUploadComplete?.(results);
      clearFiles();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Upload failed");
    } finally {
      setUploading(false);
      setUploadProgress(0);
      setCurrentFileProgress(0);
      setCurrentFileName("");
    }
  };

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  return (
    <div className="space-y-6">
      {/* Drop Zone */}
      <Card
        className={`border-2 border-dashed transition-colors ${
          isDragOver
            ? "border-blue-400 bg-blue-50"
            : "border-gray-300 hover:border-gray-400"
        }`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        <div className="p-12 text-center">
          <Upload
            className={`mx-auto h-12 w-12 ${
              isDragOver ? "text-blue-500" : "text-gray-400"
            }`}
          />
          <h3 className="mt-4 text-lg font-semibold text-gray-900">
            Drop files here or click to select
          </h3>
          <p className="mt-2 text-sm text-gray-600">
            Maximum file size: 100MB per file
          </p>
          <input
            type="file"
            multiple
            onChange={handleFileSelect}
            className="hidden"
            id="file-upload"
            disabled={uploading}
          />
          <Button asChild className="mt-4" disabled={uploading}>
            <label htmlFor="file-upload" className="cursor-pointer">
              Choose Files
            </label>
          </Button>
        </div>
      </Card>

      {/* File Preview */}
      {files.length > 0 && (
        <Card className="p-6">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold">
              Files to Upload ({files.length})
            </h3>
            <Button variant="outline" onClick={clearFiles} disabled={uploading}>
              Clear All
            </Button>
          </div>

          <div className="space-y-3 max-h-60 overflow-y-auto">
            {files.map((fileItem) => (
              <div
                key={fileItem.id}
                className="flex items-center space-x-3 p-3 border rounded-lg"
              >
                {fileItem.preview ? (
                  <img
                    src={fileItem.preview}
                    alt={fileItem.name}
                    className="w-10 h-10 object-cover rounded"
                  />
                ) : (
                  <FileText className="w-10 h-10 text-gray-400" />
                )}
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-gray-900 truncate">
                    {fileItem.name}
                  </p>
                  <p className="text-sm text-gray-500">
                    {formatFileSize(fileItem.size)}
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => removeFile(fileItem.id)}
                  disabled={uploading}
                >
                  <X className="w-4 h-4" />
                </Button>
              </div>
            ))}
          </div>

          <div className="mt-4">
            <Button
              onClick={uploadFiles}
              disabled={uploading}
              className="w-full"
            >
              {uploading
                ? "Uploading..."
                : `Upload ${files.length} File${files.length > 1 ? "s" : ""}`}
            </Button>
          </div>
        </Card>
      )}

      {/* Upload Progress */}
      {uploading && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold mb-4">Uploading Files...</h3>
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-sm mb-2">
                <span>Overall Progress</span>
                <span>{Math.round(uploadProgress)}%</span>
              </div>
              <Progress value={uploadProgress} />
            </div>

            {currentFileName && (
              <div>
                <div className="flex justify-between text-sm mb-2">
                  <span className="truncate">{currentFileName}</span>
                  <span>{Math.round(currentFileProgress)}%</span>
                </div>
                <Progress value={currentFileProgress} />
              </div>
            )}
          </div>
        </Card>
      )}

      {/* Error Display */}
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* Upload Results */}
      {uploadResults.length > 0 && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold mb-4 text-green-700">
            Upload Completed! ({uploadResults.length} files)
          </h3>
          <div className="space-y-3 max-h-60 overflow-y-auto">
            {uploadResults.map((result, index) => (
              <div key={index} className="p-3 border rounded-lg bg-green-50">
                <div className="flex justify-between items-start">
                  <div>
                    <p className="font-medium text-gray-900">
                      {result.filename}
                    </p>
                    <p className="text-sm text-gray-600">
                      {formatFileSize(result.metadata.size)} •{" "}
                      {result.metadata.mime_type}
                    </p>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      navigator.clipboard.writeText(result.urls.original);
                    }}
                  >
                    Copy URL
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}
    </div>
  );
}
