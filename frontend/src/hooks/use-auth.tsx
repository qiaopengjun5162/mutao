"use client";

import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

interface AuthState {
  token: string | null;
  userId: string | null;
  username: string | null;
}

interface AuthContextType extends AuthState {
  login: (token: string, userId: string, username: string) => void;
  logout: () => void;
  isLoggedIn: boolean;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [auth, setAuth] = useState<AuthState>({
    token: null,
    userId: null,
    username: null,
  });

  useEffect(() => {
    const token = localStorage.getItem("token");
    const userId = localStorage.getItem("userId");
    const username = localStorage.getItem("username");
    if (token && userId && username) {
      setAuth({ token, userId, username });
    }
  }, []);

  const login = (token: string, userId: string, username: string) => {
    localStorage.setItem("token", token);
    localStorage.setItem("userId", userId);
    localStorage.setItem("username", username);
    setAuth({ token, userId, username });
  };

  const logout = () => {
    localStorage.removeItem("token");
    localStorage.removeItem("userId");
    localStorage.removeItem("username");
    setAuth({ token: null, userId: null, username: null });
  };

  return (
    <AuthContext.Provider value={{ ...auth, login, logout, isLoggedIn: !!auth.token }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}
