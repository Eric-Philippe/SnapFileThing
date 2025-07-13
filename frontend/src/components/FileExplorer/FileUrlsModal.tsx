import React, { useState } from "react";
import { ExplorerItem } from "./FileExplorer";

interface FileUrlsModalProps {
  file: ExplorerItem;
  onClose: () => void;
}

const FileUrlsModal: React.FC<FileUrlsModalProps> = ({ file, onClose }) => {
  if (!file || file.type !== "file" || !file.urls) return null;
  const { original, thumbnail, qoi } = file.urls;
  const [copied, setCopied] = useState<string | null>(null);

  const handleCopy = async (value: string, key: string) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(key);
      setTimeout(() => setCopied((prev) => (prev === key ? null : prev)), 1200);
    } catch {}
  };

  const CopyButton = ({ value, label }: { value: string; label: string }) => (
    <button
      type="button"
      className="ml-2 px-2 py-1 text-xs rounded bg-gray-200 hover:bg-gray-300 focus:outline-none"
      title={`Copy ${label} URL`}
      onClick={() => handleCopy(value, label)}
    >
      {copied === label ? (
        <span role="img" aria-label="Copied" className="text-green-600">
          ✔️
        </span>
      ) : (
        <span role="img" aria-label="Copy">
          📋
        </span>
      )}
    </button>
  );

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-40">
      <div className="bg-white rounded-lg shadow-lg p-6 min-w-[550px] max-w-full">
        <h2 className="text-lg font-semibold mb-4">File URLs</h2>
        <div className="space-y-3">
          <div className="flex items-center">
            <span className="font-medium">Original:</span>
            <input
              className="w-full border rounded px-2 py-1 mt-1 text-xs bg-gray-50 ml-2"
              value={original}
              readOnly
              onFocus={(e) => e.target.select()}
            />
            <CopyButton value={original} label="original" />
          </div>
          {thumbnail && (
            <div className="flex items-center">
              <span className="font-medium">Thumbnail:</span>
              <input
                className="w-full border rounded px-2 py-1 mt-1 text-xs bg-gray-50 ml-2"
                value={thumbnail}
                readOnly
                onFocus={(e) => e.target.select()}
              />
              <CopyButton value={thumbnail} label="thumbnail" />
            </div>
          )}
          {qoi && (
            <div className="flex items-center">
              <span className="font-medium">QOI:</span>
              <input
                className="w-full border rounded px-2 py-1 mt-1 text-xs bg-gray-50 ml-2"
                value={qoi}
                readOnly
                onFocus={(e) => e.target.select()}
              />
              <CopyButton value={qoi} label="qoi" />
            </div>
          )}
        </div>
        <button
          className="mt-6 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 float-right"
          onClick={onClose}
        >
          Close
        </button>
      </div>
    </div>
  );
};

export default FileUrlsModal;
