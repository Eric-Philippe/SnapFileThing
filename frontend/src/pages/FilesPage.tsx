import { useState } from "react";
import { Navigation } from "@/components/Navigation";
import FileExplorer from "@/components/FileExplorer/FileExplorer";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { FolderOpen } from "lucide-react";

function FilesPage() {
  const [error, setError] = useState("");
  const [currentFolderId, setCurrentFolderId] = useState<string | undefined>();

  const handleFolderChange = (folderId?: string) => {
    setCurrentFolderId(folderId);
  };

  const handleError = (errorMessage: string) => {
    setError(errorMessage);
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <Navigation />

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="flex items-center justify-between mb-8">
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
              <FolderOpen className="w-5 h-5 text-white" />
            </div>
            <h1 className="text-3xl font-bold text-gray-900">File Explorer</h1>
          </div>
        </div>

        {/* Error Display */}
        {error && (
          <Alert variant="destructive" className="mb-6">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Desktop File Explorer */}
        <div className="h-[600px]">
          <FileExplorer
            currentFolderId={currentFolderId}
            onFolderChange={handleFolderChange}
            onError={handleError}
          />
        </div>
      </main>
    </div>
  );
}

export default FilesPage;
