import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type { LoginResponse, UserView } from "@/types/api";

interface AuthState {
  isAuthenticated: boolean;
  accessToken: string | null;
  refreshToken: string | null;
  accessTokenExpiresAt: number | null;
  refreshTokenExpiresAt: number | null;
  user: UserView | null;
  isTwoFactorAuthRequired: boolean;
  isMasterKeyRequired: boolean;

  setAuth: (response: LoginResponse) => void;
  confirmTwoFactor: () => void;
  requireTwoFactor: () => void;
  updateTokens: (
    accessToken: string,
    refreshToken: string,
    accessTokenExpiredAt: string,
    refreshTokenExpiredAt: string,
  ) => void;
  logout: () => void;
  getAccessToken: () => string | null;
}

/**
 * Auth store — tokens persisted in sessionStorage.
 *
 * sessionStorage is scoped to the browser tab and cleared when the tab closes,
 * providing a reasonable balance between security and UX for a password manager.
 * Unlike localStorage, sessionStorage is not shared across tabs and is cleared
 * on tab/browser close.
 */
export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      isAuthenticated: false,
      accessToken: null,
      refreshToken: null,
      accessTokenExpiresAt: null,
      refreshTokenExpiresAt: null,
      user: null,
      isTwoFactorAuthRequired: false,
      isMasterKeyRequired: false,

      setAuth: (response: LoginResponse) => {
        const fullyAuthenticated =
          !response.isTwoFactorAuthRequired && !response.isMasterKeyRequired;

        set({
          isAuthenticated: fullyAuthenticated,
          accessToken: response.accessToken,
          refreshToken: response.refreshToken,
          accessTokenExpiresAt: new Date(response.accessTokenExpiredAt).getTime(),
          refreshTokenExpiresAt: new Date(
            response.refreshTokenExpiredAt,
          ).getTime(),
          user: response.user,
          isTwoFactorAuthRequired: response.isTwoFactorAuthRequired,
          isMasterKeyRequired: response.isMasterKeyRequired,
        });
      },

      confirmTwoFactor: () =>
        set((state) => ({
          isTwoFactorAuthRequired: false,
          isAuthenticated: !state.isMasterKeyRequired,
        })),

      requireTwoFactor: () =>
        set({
          isTwoFactorAuthRequired: true,
          isAuthenticated: false,
        }),

      updateTokens: (
        accessToken: string,
        refreshToken: string,
        accessTokenExpiredAt: string,
        refreshTokenExpiredAt: string,
      ) =>
        set({
          accessToken,
          refreshToken,
          accessTokenExpiresAt: new Date(accessTokenExpiredAt).getTime(),
          refreshTokenExpiresAt: new Date(refreshTokenExpiredAt).getTime(),
        }),

      logout: () =>
        set({
          isAuthenticated: false,
          accessToken: null,
          refreshToken: null,
          accessTokenExpiresAt: null,
          refreshTokenExpiresAt: null,
          user: null,
          isTwoFactorAuthRequired: false,
          isMasterKeyRequired: false,
        }),

      getAccessToken: () => get().accessToken,
    }),
    {
      name: "lockso-auth",
      storage: createJSONStorage(() => sessionStorage),
      // Only persist tokens and auth state, not actions
      partialize: (state) => ({
        isAuthenticated: state.isAuthenticated,
        accessToken: state.accessToken,
        refreshToken: state.refreshToken,
        accessTokenExpiresAt: state.accessTokenExpiresAt,
        refreshTokenExpiresAt: state.refreshTokenExpiresAt,
        user: state.user,
        isTwoFactorAuthRequired: state.isTwoFactorAuthRequired,
        isMasterKeyRequired: state.isMasterKeyRequired,
      }),
    },
  ),
);
