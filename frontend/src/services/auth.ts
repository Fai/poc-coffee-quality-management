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
        const res = await api.post<{ access_token: string; user_id: string }>('/auth/login', { email, password });
        api.setToken(res.access_token);
        set({ token: res.access_token, isAuthenticated: true, user: { id: res.user_id, name: '', email, business_id: '' } });
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
