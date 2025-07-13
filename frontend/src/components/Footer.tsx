import { useEffect, useState } from "react";
import { healthApi } from "@/lib/api";
import { Github } from "lucide-react";

const GITHUB_URL = "https://github.com/Eric-Philippe/SnapFileThing";

export function Footer() {
  const [version, setVersion] = useState<string>("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    healthApi
      .check()
      .then((data) => {
        setVersion(data.version || "");
      })
      .catch(() => {
        setVersion("");
      })
      .finally(() => setLoading(false));
  }, []);

  return (
    <footer className="bg-white border-t border-gray-200 shadow-sm mt-12">
      <div className="max-w-7xl mx-auto px-4 py-4 flex flex-col md:flex-row items-center justify-between text-gray-600 text-sm">
        <div className="flex items-center space-x-2 mb-2 md:mb-0">
          <span>SnapFileThing</span>
          <span className="mx-2">|</span>
          <span>
            Version: {loading ? "..." : version ? version : "unknown"}
          </span>
        </div>
        <a
          href={GITHUB_URL}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center space-x-1 hover:text-blue-600 transition-colors"
        >
          <Github className="w-4 h-4" />
          <span>Source on GitHub</span>
        </a>
      </div>
    </footer>
  );
}
