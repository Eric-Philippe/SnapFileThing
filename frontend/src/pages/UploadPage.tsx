import { useState } from "react";
import { Navigation } from "@/components/Navigation";
import { FileDropZone } from "@/components/FileDropZone";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Upload, FolderOpen } from "lucide-react";
import { Link } from "react-router-dom";

function UploadPage() {
  const [error, setError] = useState("");

  const handleError = (errorMessage: string) => {
    setError(errorMessage);
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <Navigation />

      <main className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="text-center mb-8">
          <div className="mx-auto w-16 h-16 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center mb-4">
            <Upload className="w-8 h-8 text-white" />
          </div>
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            Upload Files
          </h1>
          <p className="text-lg text-gray-600">
            Upload images and files with automatic processing and thumbnail
            generation
          </p>
        </div>

        {/* Error Display */}
        {error && (
          <Alert variant="destructive" className="mb-6">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Quick Upload Notice */}
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-medium text-blue-800">
                Need to upload to a specific folder?
              </h3>
              <p className="text-sm text-blue-600 mt-1">
                Use the File Explorer to navigate to any folder and upload files
                directly there.
              </p>
            </div>
            <Link to="/files">
              <Button variant="outline" size="sm" className="ml-4">
                <FolderOpen className="w-4 h-4 mr-2" />
                Open File Explorer
              </Button>
            </Link>
          </div>
        </div>

        {/* File Drop Zone */}
        <div className="mb-8">
          <FileDropZone selectedFolderId={undefined} />
        </div>

        <div className="mt-12 bg-white rounded-lg p-6 shadow-sm border">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">
            Supported Features
          </h2>
          <div className="grid md:grid-cols-2 gap-6">
            <div>
              <h3 className="font-semibold text-gray-900 mb-2">
                Image Processing
              </h3>
              <ul className="text-sm text-gray-600 space-y-1">
                <li>• Automatic QOI conversion for better compression</li>
                <li>• Thumbnail generation (~200px)</li>
                <li>• Preserves original format</li>
                <li>• Supports JPEG, PNG, WebP, and more</li>
              </ul>
            </div>

            <div>
              <h3 className="font-semibold text-gray-900 mb-2">
                File Management
              </h3>
              <ul className="text-sm text-gray-600 space-y-1">
                <li>• Maximum file size: 100MB</li>
                <li>• Direct URL access</li>
                <li>• Secure storage</li>
                <li>• Bulk upload support</li>
              </ul>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export default UploadPage;
