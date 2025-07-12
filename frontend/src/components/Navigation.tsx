import { Link, useLocation, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/contexts/AuthContext";
import {
  Camera,
  Upload,
  FolderOpen,
  LogOut,
  Home,
  Menu,
  X,
} from "lucide-react";
import { useState } from "react";

export function Navigation() {
  const { logout } = useAuth();
  const location = useLocation();
  const navigate = useNavigate();
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  const isActive = (path: string) => location.pathname === path;

  const handleLogout = async () => {
    try {
      await logout();
    } catch {
      // Even if logout fails, we should still redirect
    }
    // Use navigate to respect the basename configuration
    navigate("/");
  };

  const toggleMobileMenu = () => {
    setIsMobileMenuOpen(!isMobileMenuOpen);
  };

  const closeMobileMenu = () => {
    setIsMobileMenuOpen(false);
  };

  return (
    <nav className="bg-white border-b border-gray-200 shadow-sm">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          {/* Logo and brand */}
          <div className="flex items-center">
            <Link
              to="/"
              className="flex items-center space-x-2"
              onClick={closeMobileMenu}
            >
              <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
                <Camera className="w-5 h-5 text-white" />
              </div>
              <span className="text-xl font-bold text-gray-900">
                SnapFileThing
              </span>
            </Link>
          </div>

          {/* Desktop navigation */}
          <div className="hidden md:flex items-center space-x-4">
            <Link to="/">
              <Button
                variant={isActive("/") ? "default" : "ghost"}
                className="flex items-center space-x-2"
              >
                <Home className="w-4 h-4" />
                <span>Home</span>
              </Button>
            </Link>

            <Link to="/upload">
              <Button
                variant={isActive("/upload") ? "default" : "ghost"}
                className="flex items-center space-x-2"
              >
                <Upload className="w-4 h-4" />
                <span>Upload</span>
              </Button>
            </Link>

            <Link to="/files">
              <Button
                variant={isActive("/files") ? "default" : "ghost"}
                className="flex items-center space-x-2"
              >
                <FolderOpen className="w-4 h-4" />
                <span>Files</span>
              </Button>
            </Link>

            <Button
              variant="ghost"
              onClick={handleLogout}
              className="flex items-center space-x-2 text-gray-600 hover:text-gray-900"
            >
              <LogOut className="w-4 h-4" />
              <span>Logout</span>
            </Button>
          </div>

          {/* Mobile menu button */}
          <div className="md:hidden flex items-center">
            <Button
              variant="ghost"
              onClick={toggleMobileMenu}
              className="p-2"
              aria-label="Toggle mobile menu"
            >
              {isMobileMenuOpen ? (
                <X className="w-6 h-6" />
              ) : (
                <Menu className="w-6 h-6" />
              )}
            </Button>
          </div>
        </div>

        {/* Mobile navigation menu */}
        {isMobileMenuOpen && (
          <div className="md:hidden border-t border-gray-200 py-3">
            <div className="flex flex-col space-y-2">
              <Link to="/" onClick={closeMobileMenu}>
                <Button
                  variant={isActive("/") ? "default" : "ghost"}
                  className="w-full justify-start flex items-center space-x-2"
                >
                  <Home className="w-4 h-4" />
                  <span>Home</span>
                </Button>
              </Link>

              <Link to="/upload" onClick={closeMobileMenu}>
                <Button
                  variant={isActive("/upload") ? "default" : "ghost"}
                  className="w-full justify-start flex items-center space-x-2"
                >
                  <Upload className="w-4 h-4" />
                  <span>Upload</span>
                </Button>
              </Link>

              <Link to="/files" onClick={closeMobileMenu}>
                <Button
                  variant={isActive("/files") ? "default" : "ghost"}
                  className="w-full justify-start flex items-center space-x-2"
                >
                  <FolderOpen className="w-4 h-4" />
                  <span>Files</span>
                </Button>
              </Link>

              <Button
                variant="ghost"
                onClick={() => {
                  handleLogout();
                  closeMobileMenu();
                }}
                className="w-full justify-start flex items-center space-x-2 text-gray-600 hover:text-gray-900"
              >
                <LogOut className="w-4 h-4" />
                <span>Logout</span>
              </Button>
            </div>
          </div>
        )}
      </div>
    </nav>
  );
}
