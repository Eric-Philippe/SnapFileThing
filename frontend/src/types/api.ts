export interface ImportResponse {
  success: boolean;
  message: string;
}
export interface FileMetadata {
  size: number;
  mime_type: string;
  uploaded_at: string;
  width?: number;
  height?: number;
}

export interface FileUrls {
  original: string;
  qoi?: string;
  thumbnail?: string;
}

export interface UploadResponse {
  success: boolean;
  filename: string;
  urls: FileUrls;
  metadata: FileMetadata;
}

export interface FileInfo {
  filename: string;
  size: number;
  mime_type: string;
  uploaded_at: string;
  is_image: boolean;
  urls: FileUrls;
  dimensions?: [number, number];
  folder_id?: string;
}

export interface FolderInfo {
  id: string;
  name: string;
  parent_id?: string;
  created_at: string;
  file_count: number;
  folder_count: number;
  size: number;
}

export interface CreateFolderRequest {
  name: string;
  parent_id?: string;
}

export interface MoveFileRequest {
  folder_id?: string;
}

export interface MoveFolderRequest {
  parent_id?: string;
}

export interface FileListResponse {
  files: FileInfo[];
  folders: FolderInfo[];
  current_folder?: FolderInfo;
  breadcrumbs: FolderInfo[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface FolderListResponse {
  folders: FolderInfo[];
  current_folder?: FolderInfo;
  breadcrumbs: FolderInfo[];
}

export interface ErrorResponse {
  error: string;
  message?: string;
}
