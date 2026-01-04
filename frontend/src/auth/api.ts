const API_URL = import.meta.env.VITE_API_URL || '';

// Get access token from storage
function getAccessToken(): string | null {
  const token = localStorage.getItem('auth_access_token');
  if (!token) return null;
  
  // Check expiration
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    if (Date.now() >= payload.exp * 1000) {
      localStorage.removeItem('auth_access_token');
      return null;
    }
    return token;
  } catch {
    return null;
  }
}

interface RequestOptions extends RequestInit {
  requireAuth?: boolean;
}

/**
 * Fetch wrapper that automatically adds auth headers when available
 * 
 * @param endpoint - API endpoint (e.g., '/items')
 * @param options - Fetch options plus optional requireAuth flag
 * @returns Fetch response
 * 
 * Usage:
 * ```ts
 * // Public endpoint
 * const response = await api('/health');
 * 
 * // Protected endpoint (will add auth header if logged in)
 * const response = await api('/items', { method: 'POST', body: JSON.stringify(data) });
 * 
 * // Require auth (will throw if not logged in)
 * const response = await api('/profile', { requireAuth: true });
 * ```
 */
export async function api(endpoint: string, options: RequestOptions = {}): Promise<Response> {
  const { requireAuth = false, ...fetchOptions } = options;
  
  const headers = new Headers(fetchOptions.headers);
  
  // Set content type for JSON if body is present and not already set
  if (fetchOptions.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  // Add auth header if token is available
  const token = getAccessToken();
  if (token) {
    headers.set('Authorization', `Bearer ${token}`);
  } else if (requireAuth) {
    throw new Error('Authentication required');
  }

  const response = await fetch(`${API_URL}${endpoint}`, {
    ...fetchOptions,
    headers,
  });

  // Handle 401 - could trigger re-login here
  if (response.status === 401) {
    // Optionally redirect to login
    // window.location.href = '/login';
    console.warn('Unauthorized request to:', endpoint);
  }

  return response;
}

/**
 * Typed API response helper
 */
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

/**
 * Fetch and parse JSON response
 */
export async function apiJson<T>(endpoint: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
  const response = await api(endpoint, options);
  return response.json();
}
