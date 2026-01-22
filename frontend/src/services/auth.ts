import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { api } from './api';

interface User {
  id: string;
  name: string;
  email: string;
  business_id: string;
}

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  setToken: (token: string) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      user: null,
      token: null,
      isAuthenticated: false,

      login: async (email: string, password: string) => {
        // Mock login for demo - accepts any email/password
        const mockToken = 'demo-token-' + Date.now();
        const mockUser = { id: 'demo-user', name: email.split('@')[0], email, business_id: 'demo-business' };
        api.setToken(mockToken);
        set({ token: mockToken, isAuthenticated: true, user: mockUser });
      },

      logout: () => {
        api.setToken(null);
        set({ user: null, token: null, isAuthenticated: false });
      },

      setToken: (token: string) => {
        api.setToken(token);
        set({ token, isAuthenticated: true });
      },
    }),
    { name: 'auth-storage', partialize: (state) => ({ token: state.token, user: state.user, isAuthenticated: state.isAuthenticated }) }
  )
);

// Initialize token on load
const stored = useAuthStore.getState();
if (stored.token) api.setToken(stored.token);
