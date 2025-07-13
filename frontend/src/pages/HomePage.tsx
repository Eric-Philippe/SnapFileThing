import { useState } from "react";
import { Link } from "react-router-dom";
import { Navigation } from "@/components/Navigation";
import { FileDropZone } from "@/components/FileDropZone";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { UploadResponse } from "@/types/api";
import { Upload, FolderOpen, Camera, Zap, Shield, Globe } from "lucide-react";
import { Footer } from "@/components/Footer";

function HomePage() {
  const [recentUploads, setRecentUploads] = useState<UploadResponse[]>([]);

  const handleUploadComplete = (results: UploadResponse[]) => {
    setRecentUploads((prev) => [...results, ...prev].slice(0, 5)); // Keep only last 5
  };

  return (
    <div className="min-h-screen bg-gray-50">
      <Navigation />

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Hero Section */}
        <div className="text-center mb-12">
          <div className="mx-auto w-20 h-20 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center mb-6">
            <Camera className="w-10 h-10 text-white" />
          </div>
          <h1 className="text-4xl font-bold text-gray-900 mb-4">
            Welcome to SnapFileThing
          </h1>
          <p className="text-xl text-gray-600 max-w-2xl mx-auto">
            Lightweight file hosting with automatic image conversion,
            thumbnails, and direct URLs. Perfect for screenshots, photos, and
            any file sharing needs.
          </p>
        </div>

        {/* Quick Actions */}
        <div className="grid md:grid-cols-2 gap-6 mb-12">
          <Card className="p-6 hover:shadow-lg transition-shadow">
            <div className="flex items-center space-x-4 mb-4">
              <div className="w-12 h-10 md:w-12 md:h-12 bg-blue-100 rounded-lg flex items-center justify-center">
                <Upload className="w-4 h-4 md:w-6 md:h-6 text-blue-600" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-gray-900">
                  Quick Upload
                </h3>
                <p className="text-gray-600">
                  Drop files below or go to the dedicated upload page
                </p>
              </div>
            </div>
            <Link to="/upload">
              <Button className="w-full">Go to Upload Page</Button>
            </Link>
          </Card>

          <Card className="p-6 hover:shadow-lg transition-shadow">
            <div className="flex items-center space-x-4 mb-4">
              <div className="w-12 h-10 md:w-12 md:h-12 bg-green-100 rounded-lg flex items-center justify-center">
                <FolderOpen className="w-4 h-4 md:w-6 md:h-6 text-green-600" />
              </div>
              <div>
                <h3 className="text-lg font-semibold text-gray-900">
                  Browse Files
                </h3>
                <p className="text-gray-600">
                  View, manage, and get direct URLs for your files
                </p>
              </div>
            </div>
            <Link to="/files">
              <Button variant="outline" className="w-full">
                Browse Files
              </Button>
            </Link>
          </Card>
        </div>

        {/* Features */}
        <div className="mb-12">
          <h2 className="text-2xl font-bold text-gray-900 text-center mb-8">
            Features
          </h2>
          <div className="grid md:grid-cols-3 gap-6">
            <div className="text-center">
              <div className="w-16 h-16 bg-purple-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <Zap className="w-8 h-8 text-purple-600" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                Fast Processing
              </h3>
              <p className="text-gray-600">
                Automatic QOI conversion and thumbnail generation for images
              </p>
            </div>

            <div className="text-center">
              <div className="w-16 h-16 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <Globe className="w-8 h-8 text-blue-600" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                Direct URLs
              </h3>
              <p className="text-gray-600">
                Get direct links to your files for easy sharing and embedding
              </p>
            </div>

            <div className="text-center">
              <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <Shield className="w-8 h-8 text-green-600" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900 mb-2">
                Secure
              </h3>
              <p className="text-gray-600">
                Protected with authentication and rate limiting
              </p>
            </div>
          </div>
        </div>

        {/* Drop Zone */}
        <div className="mb-12">
          <h2 className="text-2xl font-bold text-gray-900 mb-6">
            Quick Upload
          </h2>
          <FileDropZone onUploadComplete={handleUploadComplete} />
        </div>

        {/* Recent Uploads */}
        {recentUploads.length > 0 && (
          <div>
            <h2 className="text-2xl font-bold text-gray-900 mb-6">
              Recent Uploads
            </h2>
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
              {recentUploads.map((upload, index) => (
                <Card key={index} className="p-4">
                  <div className="flex items-center space-x-3">
                    {upload.metadata.mime_type.startsWith("image/") &&
                    upload.urls.thumbnail ? (
                      <img
                        src={upload.urls.thumbnail}
                        alt={upload.filename}
                        className="w-12 h-12 object-cover rounded"
                      />
                    ) : (
                      <div className="w-12 h-12 bg-gray-100 rounded flex items-center justify-center">
                        <Upload className="w-6 h-6 text-gray-400" />
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <p className="font-medium text-gray-900 truncate">
                        {upload.filename}
                      </p>
                      <p className="text-sm text-gray-500">
                        {(upload.metadata.size / 1024 / 1024).toFixed(2)} MB
                      </p>
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() =>
                        navigator.clipboard.writeText(upload.urls.original)
                      }
                    >
                      Copy URL
                    </Button>
                  </div>
                </Card>
              ))}
            </div>
          </div>
        )}
      </main>

      <Footer />
    </div>
  );
}

export default HomePage;
