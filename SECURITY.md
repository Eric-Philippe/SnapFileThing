# Security Policy

## Supported Versions

Currently, only the latest version of SnapFileThing is supported with security updates.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it by creating an issue in the repository or contacting the maintainers directly.

## Security Features

SnapFileThing implements several security measures:

### File Upload Security

- **File Type Validation**: Magic number validation to prevent malicious file uploads
- **File Size Limits**: Configurable maximum file size to prevent resource exhaustion
- **Filename Sanitization**: Automatic sanitization to prevent directory traversal attacks
- **MIME Type Validation**: Server-side MIME type validation based on file content

### Authentication & Authorization

- **Basic Authentication**: Secure Basic Auth implementation for public mode
- **Constant-Time Comparison**: Protection against timing attacks in credential validation
- **Protected Endpoints**: Authentication required for sensitive operations in public mode

### Network Security

- **CORS Protection**: Configurable CORS policies
- **Dual Port Architecture**: Separation of authenticated and public endpoints
- **Request Logging**: Comprehensive logging for security monitoring

### File System Security

- **Secure File Storage**: Files stored with proper permissions
- **Path Traversal Prevention**: Validation to prevent directory traversal attacks
- **Temporary File Cleanup**: Automatic cleanup of temporary files

### Additional Security Measures

- **Error Information Disclosure**: Limited error information in responses
- **Input Validation**: Comprehensive validation of all user inputs
- **Memory Safety**: Rust's memory safety guarantees prevent common vulnerabilities

## Configuration Security

### Production Deployment

- Change default admin credentials before public deployment
- Use strong passwords (minimum 12 characters, mixed case, numbers, symbols)
- Configure appropriate CORS policies
- Use HTTPS in production (reverse proxy recommended)
- Regularly update dependencies

### Environment Variables

Sensitive configuration can be set via environment variables:

- `ADMIN_PASSWORD`: Admin password for public mode
- `AUTH_MODE`: Set to "public" for production deployments requiring authentication

## Best Practices

1. **Regular Updates**: Keep SnapFileThing and its dependencies updated
2. **Monitoring**: Monitor upload activity and file access patterns
3. **Backup**: Regularly backup uploaded files
4. **Access Control**: Limit network access to the service appropriately
5. **Reverse Proxy**: Use a reverse proxy (nginx, Apache) for HTTPS and additional security
