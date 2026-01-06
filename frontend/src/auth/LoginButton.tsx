import { useAuth } from './useAuth';

export function LoginButton() {
  const { isAuthenticated, isLoading, isConfigured, user, login, logout } = useAuth();

  if (!isConfigured) {
    return (
      <span className="text-sm text-gray-400">
        Auth not configured
      </span>
    );
  }

  if (isLoading) {
    return (
      <span className="text-sm text-gray-500">
        Loading...
      </span>
    );
  }

  if (isAuthenticated && user) {
    return (
      <div className="flex items-center gap-3">
        <span className="text-sm text-gray-700">
          {user.name || user.email || 'User'}
        </span>
        <button
          onClick={logout}
          className="px-3 py-1.5 text-sm text-gray-600 hover:text-gray-800 border border-gray-300 rounded-md hover:bg-gray-50 transition"
        >
          Sign out
        </button>
      </div>
    );
  }

  return (
    <button
      onClick={login}
      className="px-4 py-2 text-sm font-medium text-white bg-blue-500 rounded-md hover:bg-blue-600 transition"
    >
      Sign in
    </button>
  );
}
