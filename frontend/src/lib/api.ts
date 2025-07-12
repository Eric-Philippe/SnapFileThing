import {
  FileListResponse,
  UploadResponse,
  ErrorResponse,
  FolderListResponse,
  CreateFolderRequest,
  MoveFileRequest,
  MoveFolderRequest,
} from "../types/api";

// Token management
interface TokenData {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
}

interface LoginResponse extends TokenData {
  // Login response contains token data
}

// Helper function to get the correct base path for redirects
function getBasePath(): string {
  // This gets the base path from the document.baseURI or falls back to "/"
  const base = document.querySelector("base")?.href || document.baseURI;
  const url = new URL(base);
  return url.pathname;
}

// Helper function to redirect to login page respecting basename
function redirectToLogin(): void {
  window.location.href = getBasePath();
}

class TokenManager {
  private static readonly ACCESS_TOKEN_KEY = "access_token";
  private static readonly REFRESH_TOKEN_KEY = "refresh_token";
  private static readonly TOKEN_EXPIRY_KEY = "token_expiry";

  static setTokens(tokenData: TokenData): void {
    const expiryTime = Date.now() + tokenData.expires_in * 1000;

    localStorage.setItem(this.ACCESS_TOKEN_KEY, tokenData.access_token);
    localStorage.setItem(this.REFRESH_TOKEN_KEY, tokenData.refresh_token);
    localStorage.setItem(this.TOKEN_EXPIRY_KEY, expiryTime.toString());
  }

  static getAccessToken(): string | null {
    return localStorage.getItem(this.ACCESS_TOKEN_KEY);
  }

  static getRefreshToken(): string | null {
    return localStorage.getItem(this.REFRESH_TOKEN_KEY);
  }

  static isTokenExpired(): boolean {
    const expiryTime = localStorage.getItem(this.TOKEN_EXPIRY_KEY);
    if (!expiryTime) return true;

    // Consider token expired if it expires in the next 5 minutes
    return Date.now() > parseInt(expiryTime) - 5 * 60 * 1000;
  }

  static clearTokens(): void {
    localStorage.removeItem(this.ACCESS_TOKEN_KEY);
    localStorage.removeItem(this.REFRESH_TOKEN_KEY);
    localStorage.removeItem(this.TOKEN_EXPIRY_KEY);
  }

  static hasTokens(): boolean {
    return !!this.getAccessToken() && !!this.getRefreshToken();
  }
}

class ApiError extends Error {
  status: number;

  constructor(message: string, status: number) {
    super(message);
    this.status = status;
    this.name = "ApiError";
  }
}

const API_BASE = "/api";

// Helper function to get auth headers
function getAuthHeaders(): Record<string, string> {
  const token = TokenManager.getAccessToken();
  return token ? { Authorization: `Bearer ${token}` } : {};
}

// Helper function to handle API responses
async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    if (response.status === 401) {
      // Try to refresh token
      const refreshed = await authApi.tryRefreshToken();
      if (!refreshed) {
        // Redirect to login or handle auth error
        TokenManager.clearTokens();
        redirectToLogin();
        throw new ApiError("Unauthorized", 401);
      }
      // If refresh was successful, the original request should be retried by the caller
      throw new ApiError("Token expired", 401);
    }

    let errorMessage = "An error occurred";
    try {
      const errorData: ErrorResponse = await response.json();
      errorMessage = errorData.message || errorData.error || errorMessage;
    } catch {
      errorMessage = response.statusText || errorMessage;
    }

    throw new ApiError(errorMessage, response.status);
  }

  return response.json();
}

// Enhanced fetch function with automatic token refresh
async function authenticatedFetch(
  url: string,
  options: RequestInit = {}
): Promise<Response> {
  // Check if token needs refresh before making request
  if (TokenManager.hasTokens() && TokenManager.isTokenExpired()) {
    await authApi.tryRefreshToken();
  }

  const headers = {
    "Content-Type": "application/json",
    ...getAuthHeaders(),
    ...options.headers,
  };

  let response = await fetch(url, {
    ...options,
    headers,
    credentials: "include",
  });

  // If unauthorized, try refreshing token once
  if (response.status === 401 && TokenManager.hasTokens()) {
    const refreshed = await authApi.tryRefreshToken();
    if (refreshed) {
      // Retry the original request with new token
      const newHeaders = {
        ...headers,
        ...getAuthHeaders(),
      };
      response = await fetch(url, {
        ...options,
        headers: newHeaders,
        credentials: "include",
      });
    }
  }

  return response;
}

// Auth functions
export const authApi = {
  // Check if user is authenticated
  async checkAuth(): Promise<boolean> {
    try {
      const response = await fetch(`${API_BASE}/auth/verify`, {
        headers: getAuthHeaders(),
        credentials: "include",
      });

      if (response.ok) {
        const data = await response.json();
        return data.valid === true;
      }
      return false;
    } catch {
      return false;
    }
  },

  // Login with username/password
  async login(username: string, password: string): Promise<boolean> {
    try {
      const response = await fetch(`${API_BASE}/auth/login`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ username, password }),
        credentials: "include",
      });

      if (response.ok) {
        const tokenData: LoginResponse = await response.json();
        TokenManager.setTokens(tokenData);
        return true;
      }
      return false;
    } catch {
      return false;
    }
  },

  // Logout - revoke tokens on server and clear local storage
  async logout(): Promise<void> {
    try {
      // Call logout endpoint to blacklist the current token
      await fetch(`${API_BASE}/auth/logout`, {
        method: "POST",
        headers: getAuthHeaders(),
        credentials: "include",
      });
    } catch {
      // Clear tokens on error
      TokenManager.clearTokens();
    } finally {
      // Always clear local tokens regardless of server response
      TokenManager.clearTokens();
      // Redirect to login page
      redirectToLogin();
    }
  },

  // Try to refresh the access token
  async tryRefreshToken(): Promise<boolean> {
    const refreshToken = TokenManager.getRefreshToken();
    if (!refreshToken) {
      return false;
    }

    try {
      const response = await fetch(`${API_BASE}/auth/refresh`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ refresh_token: refreshToken }),
        credentials: "include",
      });

      if (response.ok) {
        const tokenData: LoginResponse = await response.json();
        TokenManager.setTokens(tokenData);
        return true;
      } else {
        // Refresh failed, clear tokens
        TokenManager.clearTokens();
        return false;
      }
    } catch {
      TokenManager.clearTokens();
      return false;
    }
  },
};

// File upload functions
export const uploadApi = {
  async uploadFile(
    file: File,
    folderId?: string,
    onProgress?: (progress: number) => void
  ): Promise<UploadResponse> {
    const formData = new FormData();
    formData.append("file", file);
    if (folderId) {
      formData.append("folder_id", folderId);
    }

    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();

      if (onProgress) {
        xhr.upload.addEventListener("progress", (e) => {
          if (e.lengthComputable) {
            const progress = (e.loaded / e.total) * 100;
            onProgress(progress);
          }
        });
      }

      xhr.addEventListener("load", () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          try {
            const response = JSON.parse(xhr.responseText);
            resolve(response);
          } catch (error) {
            reject(new ApiError("Invalid response format", xhr.status));
          }
        } else {
          let errorMessage = "Upload failed";
          try {
            const errorData = JSON.parse(xhr.responseText);
            errorMessage = errorData.message || errorData.error || errorMessage;
          } catch {
            errorMessage = xhr.statusText || errorMessage;
          }
          reject(new ApiError(errorMessage, xhr.status));
        }
      });

      xhr.addEventListener("error", () => {
        reject(new ApiError("Network error", 0));
      });

      xhr.withCredentials = true;
      xhr.open("POST", `${API_BASE}/upload`);

      // Set auth headers (must be after open() but before send())
      const token = TokenManager.getAccessToken();
      if (token) {
        xhr.setRequestHeader("Authorization", `Bearer ${token}`);
      }

      xhr.send(formData);
    });
  },

  async uploadMultipleFiles(
    files: File[],
    folderId?: string,
    onProgress?: (overall: number, current: number, currentFile: string) => void
  ): Promise<UploadResponse[]> {
    const results: UploadResponse[] = [];

    for (let i = 0; i < files.length; i++) {
      const file = files[i];

      try {
        const result = await this.uploadFile(file, folderId, (fileProgress) => {
          const overallProgress =
            (i / files.length) * 100 + fileProgress / files.length;
          onProgress?.(overallProgress, fileProgress, file.name);
        });
        results.push(result);
      } catch (error) {
        // Continue with other files even if one fails
        throw error; // Re-throw to let the caller handle it
      }
    }

    return results;
  },
};

// File management functions
export const filesApi = {
  // Export all files as ZIP
  async exportAllFiles(
    originalsOnly = false,
    folderId?: string
  ): Promise<void> {
    const params = new URLSearchParams();
    if (originalsOnly) params.append("originals_only", "true");
    if (folderId) params.append("folder_id", folderId);

    const response = await authenticatedFetch(
      `${API_BASE}/files/export?${params.toString()}`,
      {
        method: "GET",
      }
    );

    if (!response.ok) {
      await handleResponse(response); // will throw
      return;
    }

    // Get filename from Content-Disposition header
    const disposition = response.headers.get("Content-Disposition");
    let filename = "export.zip";
    if (disposition) {
      const match = disposition.match(/filename="([^"]+)"/);
      if (match) filename = match[1];
    }

    const blob = await response.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    window.URL.revokeObjectURL(url);
  },
  async listFiles(
    page = 0,
    perPage = 20,
    folderId?: string
  ): Promise<FileListResponse> {
    const params = new URLSearchParams({
      page: page.toString(),
      per_page: perPage.toString(),
    });

    if (folderId) {
      params.append("folder_id", folderId);
    }

    const response = await authenticatedFetch(
      `${API_BASE}/files?${params.toString()}`
    );
    return handleResponse<FileListResponse>(response);
  },

  async deleteFile(filename: string): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE}/files/${encodeURIComponent(filename)}`,
      {
        method: "DELETE",
      }
    );
    await handleResponse(response);
  },

  async moveFile(filename: string, request: MoveFileRequest): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE}/files/${encodeURIComponent(filename)}/move`,
      {
        method: "PUT",
        body: JSON.stringify(request),
      }
    );
    await handleResponse(response);
  },
};

// Folder management functions
export const foldersApi = {
  async listFolders(folderId?: string): Promise<FolderListResponse> {
    const params = new URLSearchParams();
    if (folderId) {
      params.append("folder_id", folderId);
    }

    const response = await authenticatedFetch(
      `${API_BASE}/folders?${params.toString()}`
    );
    return handleResponse<FolderListResponse>(response);
  },

  async createFolder(request: CreateFolderRequest): Promise<void> {
    const response = await authenticatedFetch(`${API_BASE}/folders`, {
      method: "POST",
      body: JSON.stringify(request),
    });
    await handleResponse(response);
  },

  async deleteFolder(folderId: string): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE}/folders/${encodeURIComponent(folderId)}`,
      {
        method: "DELETE",
      }
    );
    await handleResponse(response);
  },

  async moveFolder(
    folderId: string,
    request: MoveFolderRequest
  ): Promise<void> {
    const response = await authenticatedFetch(
      `${API_BASE}/folders/${encodeURIComponent(folderId)}/move`,
      {
        method: "PUT",
        body: JSON.stringify(request),
      }
    );
    await handleResponse(response);
  },
};

// Health check
export const healthApi = {
  async check(): Promise<{ status: string }> {
    const response = await fetch(`${API_BASE}/health`);
    return handleResponse(response);
  },
};

export { ApiError };
