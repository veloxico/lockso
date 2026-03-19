import { api } from "./client";

export interface WebAuthnCredentialView {
  id: string;
  credentialId: string;
  deviceName: string;
  backedUp: boolean;
  createdAt: string;
  lastUsedAt: string | null;
}

export const webauthnApi = {
  beginRegistration: () =>
    api.post<PublicKeyCredentialCreationOptionsJSON>("/webauthn/register/begin"),

  finishRegistration: (response: unknown) =>
    api.post<WebAuthnCredentialView>("/webauthn/register/finish", response),

  beginAuthentication: () =>
    api.post<PublicKeyCredentialRequestOptionsJSON>("/webauthn/authenticate/begin"),

  finishAuthentication: (response: unknown) =>
    api.post<{ verified: boolean }>("/webauthn/authenticate/finish", response),

  listCredentials: () =>
    api.get<WebAuthnCredentialView[]>("/webauthn/credentials"),

  deleteCredential: (id: string) =>
    api.delete<{ message: string }>(`/webauthn/credentials/${id}`),

  renameCredential: (id: string, name: string) =>
    api.put<{ message: string }>(`/webauthn/credentials/${id}/name`, { name }),
};

// ─── Browser WebAuthn helpers ───────────────────────────────────────────────

/** Convert base64url string to ArrayBuffer */
function base64urlToBuffer(base64url: string): ArrayBuffer {
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");
  const padding = "=".repeat((4 - (base64.length % 4)) % 4);
  const binary = atob(base64 + padding);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

/** Convert ArrayBuffer to base64url string */
function bufferToBase64url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]!);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

// These types match the WebAuthn JSON serialization format from the server
interface PublicKeyCredentialCreationOptionsJSON {
  challenge: string;
  rp: { name: string; id: string };
  user: { id: string; name: string; displayName: string };
  pubKeyCredParams: { type: string; alg: number }[];
  timeout: number;
  authenticatorSelection: {
    authenticatorAttachment?: string;
    residentKey: string;
    requireResidentKey: boolean;
    userVerification: string;
  };
  attestation: string;
  excludeCredentials: { type: string; id: string; transports?: string[] }[];
}

interface PublicKeyCredentialRequestOptionsJSON {
  challenge: string;
  timeout: number;
  rpId: string;
  allowCredentials: { type: string; id: string; transports?: string[] }[];
  userVerification: string;
}

/**
 * Register a new WebAuthn credential.
 * Handles the full ceremony: server options → browser create → server verify.
 */
export async function registerCredential(deviceName: string): Promise<WebAuthnCredentialView> {
  const options = await webauthnApi.beginRegistration();

  const publicKeyOptions: PublicKeyCredentialCreationOptions = {
    challenge: base64urlToBuffer(options.challenge),
    rp: options.rp,
    user: {
      id: base64urlToBuffer(options.user.id),
      name: options.user.name,
      displayName: options.user.displayName,
    },
    pubKeyCredParams: options.pubKeyCredParams.map((p) => ({
      type: p.type as PublicKeyCredentialType,
      alg: p.alg,
    })),
    timeout: options.timeout,
    authenticatorSelection: {
      authenticatorAttachment: options.authenticatorSelection.authenticatorAttachment as AuthenticatorAttachment | undefined,
      residentKey: options.authenticatorSelection.residentKey as ResidentKeyRequirement,
      requireResidentKey: options.authenticatorSelection.requireResidentKey,
      userVerification: options.authenticatorSelection.userVerification as UserVerificationRequirement,
    },
    attestation: options.attestation as AttestationConveyancePreference,
    excludeCredentials: options.excludeCredentials.map((c) => ({
      type: c.type as PublicKeyCredentialType,
      id: base64urlToBuffer(c.id),
      transports: c.transports as AuthenticatorTransport[] | undefined,
    })),
  };

  const credential = (await navigator.credentials.create({
    publicKey: publicKeyOptions,
  })) as PublicKeyCredential | null;

  if (!credential) {
    throw new Error("Registration cancelled");
  }

  const attestation = credential.response as AuthenticatorAttestationResponse;

  const result = await webauthnApi.finishRegistration({
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    deviceName,
    response: {
      clientDataJson: bufferToBase64url(attestation.clientDataJSON),
      attestationObject: bufferToBase64url(attestation.attestationObject),
      transports: attestation.getTransports?.() ?? [],
    },
  });

  return result;
}

/**
 * Authenticate with a WebAuthn credential.
 * Handles the full ceremony: server options → browser get → server verify.
 */
export async function authenticateCredential(): Promise<boolean> {
  const options = await webauthnApi.beginAuthentication();

  const publicKeyOptions: PublicKeyCredentialRequestOptions = {
    challenge: base64urlToBuffer(options.challenge),
    timeout: options.timeout,
    rpId: options.rpId,
    allowCredentials: options.allowCredentials.map((c) => ({
      type: c.type as PublicKeyCredentialType,
      id: base64urlToBuffer(c.id),
      transports: c.transports as AuthenticatorTransport[] | undefined,
    })),
    userVerification: options.userVerification as UserVerificationRequirement,
  };

  const assertion = (await navigator.credentials.get({
    publicKey: publicKeyOptions,
  })) as PublicKeyCredential | null;

  if (!assertion) {
    throw new Error("Authentication cancelled");
  }

  const assertionResponse = assertion.response as AuthenticatorAssertionResponse;

  const result = await webauthnApi.finishAuthentication({
    id: assertion.id,
    rawId: bufferToBase64url(assertion.rawId),
    type: assertion.type,
    response: {
      clientDataJson: bufferToBase64url(assertionResponse.clientDataJSON),
      authenticatorData: bufferToBase64url(assertionResponse.authenticatorData),
      signature: bufferToBase64url(assertionResponse.signature),
      userHandle: assertionResponse.userHandle
        ? bufferToBase64url(assertionResponse.userHandle)
        : undefined,
    },
  });

  return result.verified;
}
