import { createContext, useContext, useState, useEffect, ReactNode, useCallback } from 'react';

// Auth configuration - loaded from environment
const AUTH_CONFIG = {
  userPoolId: import.meta.env.VITE_COGNITO_USER_POOL_ID || '',
  clientId: import.meta.env.VITE_COGNITO_CLIENT_ID || '',
  domain: import.meta.env.VITE_COGNITO_DOMAIN || '',
  redirectUri: window.location.origin + '/callback',
  logoutUri: window.location.origin,
};

// Check if auth is configured
const isAuthConfigured = () => {
  return AUTH_CONFIG.userPoolId && AUTH_CONFIG.clientId && AUTH_CONFIG.domain;
};

interface User {
  id: string;
  email?: string;
  name?: string;
}

interface AuthContextType {
  user: User | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  isConfigured: boolean;
  login: () => void;
  logout: () => void;
  getAccessToken: () => string | null;
}

export const AuthContext = createContext<AuthContextType | undefined>(undefined);

// Token storage keys
const ACCESS_TOKEN_KEY = 'auth_access_token';
const ID_TOKEN_KEY = 'auth_id_token';
const REFRESH_TOKEN_KEY = 'auth_refresh_token';

// Parse JWT token (without verification - that happens server-side)
function parseJwt(token: string): Record<string, unknown> | null {
  try {
    const base64Url = token.split('.')[1];
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split('')
        .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
        .join('')
    );
    return JSON.parse(jsonPayload);
  } catch {
    return null;
  }
}

// Check if token is expired
function isTokenExpired(token: string): boolean {
  const payload = parseJwt(token);
  if (!payload || typeof payload.exp !== 'number') return true;
  return Date.now() >= payload.exp * 1000;
}

// Extract user from ID token
function getUserFromToken(idToken: string): User | null {
  const payload = parseJwt(idToken);
  if (!payload) return null;
  
  return {
    id: payload.sub as string,
    email: payload.email as string | undefined,
    name: payload.name as string | undefined,
  };
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Initialize auth state from stored tokens
  useEffect(() => {
    if (!isAuthConfigured()) {
      setIsLoading(false);
      return;
    }

    const idToken = localStorage.getItem(ID_TOKEN_KEY);
    const accessToken = localStorage.getItem(ACCESS_TOKEN_KEY);

    if (idToken && accessToken && !isTokenExpired(accessToken)) {
      const user = getUserFromToken(idToken);
      setUser(user);
    } else {
      // Clear expired tokens
      localStorage.removeItem(ACCESS_TOKEN_KEY);
      localStorage.removeItem(ID_TOKEN_KEY);
      localStorage.removeItem(REFRESH_TOKEN_KEY);
    }

    setIsLoading(false);
  }, []);

  // Handle OAuth callback
  useEffect(() => {
    if (!isAuthConfigured()) return;

    const handleCallback = async () => {
      const params = new URLSearchParams(window.location.search);
      const code = params.get('code');

      if (code && window.location.pathname === '/callback') {
        setIsLoading(true);
        try {
          // Exchange code for tokens
          const tokenEndpoint = `${AUTH_CONFIG.domain}/oauth2/token`;
          const response = await fetch(tokenEndpoint, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/x-www-form-urlencoded',
            },
            body: new URLSearchParams({
              grant_type: 'authorization_code',
              client_id: AUTH_CONFIG.clientId,
              code,
              redirect_uri: AUTH_CONFIG.redirectUri,
            }),
          });

          if (response.ok) {
            const tokens = await response.json();
            
            localStorage.setItem(ACCESS_TOKEN_KEY, tokens.access_token);
            localStorage.setItem(ID_TOKEN_KEY, tokens.id_token);
            if (tokens.refresh_token) {
              localStorage.setItem(REFRESH_TOKEN_KEY, tokens.refresh_token);
            }

            const user = getUserFromToken(tokens.id_token);
            setUser(user);

            // Clean up URL
            window.history.replaceState({}, document.title, '/');
          } else {
            console.error('Token exchange failed:', await response.text());
          }
        } catch (error) {
          console.error('Auth callback error:', error);
        }
        setIsLoading(false);
      }
    };

    handleCallback();
  }, []);

  const login = useCallback(() => {
    if (!isAuthConfigured()) {
      console.warn('Auth not configured');
      return;
    }

    const authUrl = new URL(`${AUTH_CONFIG.domain}/login`);
    authUrl.searchParams.set('client_id', AUTH_CONFIG.clientId);
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('scope', 'email openid profile');
    authUrl.searchParams.set('redirect_uri', AUTH_CONFIG.redirectUri);

    window.location.href = authUrl.toString();
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem(ACCESS_TOKEN_KEY);
    localStorage.removeItem(ID_TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    setUser(null);

    if (!isAuthConfigured()) return;

    const logoutUrl = new URL(`${AUTH_CONFIG.domain}/logout`);
    logoutUrl.searchParams.set('client_id', AUTH_CONFIG.clientId);
    logoutUrl.searchParams.set('logout_uri', AUTH_CONFIG.logoutUri);

    window.location.href = logoutUrl.toString();
  }, []);

  const getAccessToken = useCallback(() => {
    const token = localStorage.getItem(ACCESS_TOKEN_KEY);
    if (token && !isTokenExpired(token)) {
      return token;
    }
    return null;
  }, []);

  return (
    <AuthContext.Provider
      value={{
        user,
        isLoading,
        isAuthenticated: !!user,
        isConfigured: isAuthConfigured(),
        login,
        logout,
        getAccessToken,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

