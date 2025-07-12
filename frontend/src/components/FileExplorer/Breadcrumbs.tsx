import React from "react";
import { ChevronRightIcon } from "@heroicons/react/24/outline";
import { FolderInfo } from "../../types/api";

interface BreadcrumbsProps {
  breadcrumbs: FolderInfo[];
  onClick: (folderId?: string) => void;
}

export default function Breadcrumbs({
  breadcrumbs,
  onClick,
}: BreadcrumbsProps) {
  return (
    <nav className="flex items-center space-x-2 p-4 text-sm text-gray-600 bg-gray-50">
      <button
        onClick={() => onClick()}
        className="hover:text-gray-900 font-medium"
      >
        Root
      </button>
      {breadcrumbs.map((folder) => (
        <React.Fragment key={folder.id}>
          <ChevronRightIcon className="h-4 w-4" />
          <button
            onClick={() => onClick(folder.id)}
            className="hover:text-gray-900"
          >
            {folder.name}
          </button>
        </React.Fragment>
      ))}
    </nav>
  );
}
