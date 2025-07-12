import { BrowserRouter as Router, Routes, Route } from "react-router-dom";
import { AuthProvider, useAuth } from "./contexts/AuthContext";
import { LoginPage } from "./components/LoginPage";
import HomePage from "./pages/HomePage";
import UploadPage from "./pages/UploadPage";
import FilesPage from "./pages/FilesPage";
import { RefreshCw } from "lucide-react";
import "./index.css";

function AuthenticatedApp() {
  const { isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center">
          <RefreshCw className="mx-auto h-8 w-8 text-gray-400 animate-spin" />
          <p className="mt-4 text-gray-600">Loading...</p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return <LoginPage />;
  }

  return (
    <Routes>
      <Route path="/" element={<HomePage />} />
      <Route path="/upload" element={<UploadPage />} />
      <Route path="/files" element={<FilesPage />} />
    </Routes>
  );
}

function App() {
  return (
    <AuthProvider>
      <Router basename="/web">
        <div className="min-h-screen bg-background">
          <AuthenticatedApp />
        </div>
      </Router>
    </AuthProvider>
  );
}

export default App;
