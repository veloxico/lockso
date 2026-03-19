/**
 * Item type system — defines templates for different credential types.
 * Each type specifies which standard fields to show and what custom fields
 * to pre-populate. The type is stored as a hidden custom field `_itemType`.
 */

import type { CustomField } from "@/types/vault";

export interface ItemTypeField {
  name: string;
  /** i18n key for the label */
  labelKey: string;
  type: CustomField["type"];
  /** Placeholder i18n key */
  placeholderKey?: string;
}

export interface ItemTypeDef {
  code: string;
  /** i18n key for the label */
  labelKey: string;
  /** Lucide icon name (mapped in component) */
  icon: string;
  /** Icon background color class */
  iconBg: string;
  /** Icon color class */
  iconColor: string;
  /** Which standard fields to show */
  standardFields: {
    login?: boolean;
    password?: boolean;
    url?: boolean;
  };
  /** Pre-populated custom fields */
  customFields: ItemTypeField[];
  /** Is this a "featured" type (shown in top grid)? */
  featured: boolean;
}

export const ITEM_TYPES: ItemTypeDef[] = [
  // ── Featured (top grid) ──
  {
    code: "login",
    labelKey: "itemType.login",
    icon: "KeyRound",
    iconBg: "bg-teal-500/15",
    iconColor: "text-teal-500",
    standardFields: { login: true, password: true, url: true },
    customFields: [],
    featured: true,
  },
  {
    code: "secure_note",
    labelKey: "itemType.secureNote",
    icon: "StickyNote",
    iconBg: "bg-amber-500/15",
    iconColor: "text-amber-500",
    standardFields: {},
    customFields: [],
    featured: true,
  },
  {
    code: "credit_card",
    labelKey: "itemType.creditCard",
    icon: "CreditCard",
    iconBg: "bg-blue-500/15",
    iconColor: "text-blue-500",
    standardFields: {},
    customFields: [
      { name: "Card Number", labelKey: "itemType.cardNumber", type: "text" },
      { name: "Cardholder", labelKey: "itemType.cardholder", type: "text" },
      { name: "Expiry", labelKey: "itemType.expiry", type: "text", placeholderKey: "itemType.expiryPlaceholder" },
      { name: "CVV", labelKey: "itemType.cvv", type: "password" },
      { name: "PIN", labelKey: "itemType.pin", type: "password" },
    ],
    featured: true,
  },
  {
    code: "contact",
    labelKey: "itemType.contact",
    icon: "Contact",
    iconBg: "bg-emerald-500/15",
    iconColor: "text-emerald-500",
    standardFields: {},
    customFields: [
      { name: "First Name", labelKey: "itemType.firstName", type: "text" },
      { name: "Last Name", labelKey: "itemType.lastName", type: "text" },
      { name: "Email", labelKey: "itemType.email", type: "email" },
      { name: "Phone", labelKey: "itemType.phone", type: "text" },
      { name: "Company", labelKey: "itemType.company", type: "text" },
    ],
    featured: true,
  },
  {
    code: "password",
    labelKey: "itemType.password",
    icon: "Lock",
    iconBg: "bg-violet-500/15",
    iconColor: "text-violet-500",
    standardFields: { password: true },
    customFields: [],
    featured: true,
  },
  {
    code: "document",
    labelKey: "itemType.document",
    icon: "FileText",
    iconBg: "bg-blue-400/15",
    iconColor: "text-blue-400",
    standardFields: {},
    customFields: [
      { name: "Document Number", labelKey: "itemType.documentNumber", type: "text" },
      { name: "Issued By", labelKey: "itemType.issuedBy", type: "text" },
      { name: "Issue Date", labelKey: "itemType.issueDate", type: "text" },
      { name: "Expiry Date", labelKey: "itemType.expiryDate", type: "text" },
    ],
    featured: true,
  },

  // ── Extended list ──
  {
    code: "ssh_key",
    labelKey: "itemType.sshKey",
    icon: "Terminal",
    iconBg: "bg-gray-500/15",
    iconColor: "text-gray-500",
    standardFields: {},
    customFields: [
      { name: "Host", labelKey: "itemType.host", type: "text" },
      { name: "Port", labelKey: "itemType.port", type: "text" },
      { name: "Username", labelKey: "itemType.username", type: "text" },
      { name: "Private Key", labelKey: "itemType.privateKey", type: "password" },
      { name: "Passphrase", labelKey: "itemType.passphrase", type: "password" },
    ],
    featured: false,
  },
  {
    code: "api_credentials",
    labelKey: "itemType.apiCredentials",
    icon: "Code",
    iconBg: "bg-cyan-500/15",
    iconColor: "text-cyan-500",
    standardFields: { url: true },
    customFields: [
      { name: "API Key", labelKey: "itemType.apiKey", type: "password" },
      { name: "API Secret", labelKey: "itemType.apiSecret", type: "password" },
      { name: "Token", labelKey: "itemType.token", type: "password" },
    ],
    featured: false,
  },
  {
    code: "database",
    labelKey: "itemType.database",
    icon: "Database",
    iconBg: "bg-green-500/15",
    iconColor: "text-green-500",
    standardFields: { login: true, password: true },
    customFields: [
      { name: "Host", labelKey: "itemType.host", type: "text" },
      { name: "Port", labelKey: "itemType.port", type: "text" },
      { name: "Database Name", labelKey: "itemType.dbName", type: "text" },
      { name: "Connection String", labelKey: "itemType.connectionString", type: "password" },
    ],
    featured: false,
  },
  {
    code: "bank_account",
    labelKey: "itemType.bankAccount",
    icon: "Landmark",
    iconBg: "bg-yellow-500/15",
    iconColor: "text-yellow-500",
    standardFields: {},
    customFields: [
      { name: "Bank Name", labelKey: "itemType.bankName", type: "text" },
      { name: "Account Number", labelKey: "itemType.accountNumber", type: "text" },
      { name: "Routing Number", labelKey: "itemType.routingNumber", type: "text" },
      { name: "SWIFT/BIC", labelKey: "itemType.swift", type: "text" },
      { name: "IBAN", labelKey: "itemType.iban", type: "text" },
    ],
    featured: false,
  },
  {
    code: "wifi",
    labelKey: "itemType.wifi",
    icon: "Wifi",
    iconBg: "bg-slate-500/15",
    iconColor: "text-slate-500",
    standardFields: { password: true },
    customFields: [
      { name: "SSID", labelKey: "itemType.ssid", type: "text" },
      { name: "Security Type", labelKey: "itemType.securityType", type: "text" },
    ],
    featured: false,
  },
  {
    code: "drivers_license",
    labelKey: "itemType.driversLicense",
    icon: "Car",
    iconBg: "bg-pink-500/15",
    iconColor: "text-pink-500",
    standardFields: {},
    customFields: [
      { name: "Full Name", labelKey: "itemType.fullName", type: "text" },
      { name: "License Number", labelKey: "itemType.licenseNumber", type: "text" },
      { name: "Category", labelKey: "itemType.category", type: "text" },
      { name: "Issue Date", labelKey: "itemType.issueDate", type: "text" },
      { name: "Expiry Date", labelKey: "itemType.expiryDate", type: "text" },
    ],
    featured: false,
  },
  {
    code: "crypto_wallet",
    labelKey: "itemType.cryptoWallet",
    icon: "Bitcoin",
    iconBg: "bg-orange-500/15",
    iconColor: "text-orange-500",
    standardFields: {},
    customFields: [
      { name: "Wallet Address", labelKey: "itemType.walletAddress", type: "text" },
      { name: "Private Key", labelKey: "itemType.privateKey", type: "password" },
      { name: "Seed Phrase", labelKey: "itemType.seedPhrase", type: "password" },
      { name: "Network", labelKey: "itemType.network", type: "text" },
    ],
    featured: false,
  },
  {
    code: "software_license",
    labelKey: "itemType.softwareLicense",
    icon: "BadgeCheck",
    iconBg: "bg-blue-600/15",
    iconColor: "text-blue-600",
    standardFields: { url: true },
    customFields: [
      { name: "License Key", labelKey: "itemType.licenseKey", type: "password" },
      { name: "Version", labelKey: "itemType.version", type: "text" },
      { name: "Licensed To", labelKey: "itemType.licensedTo", type: "text" },
      { name: "Purchase Date", labelKey: "itemType.purchaseDate", type: "text" },
    ],
    featured: false,
  },
  {
    code: "medical_record",
    labelKey: "itemType.medicalRecord",
    icon: "HeartPulse",
    iconBg: "bg-red-400/15",
    iconColor: "text-red-400",
    standardFields: {},
    customFields: [
      { name: "Patient Name", labelKey: "itemType.patientName", type: "text" },
      { name: "Blood Type", labelKey: "itemType.bloodType", type: "text" },
      { name: "Allergies", labelKey: "itemType.allergies", type: "text" },
      { name: "Insurance Number", labelKey: "itemType.insuranceNumber", type: "text" },
    ],
    featured: false,
  },
  {
    code: "passport",
    labelKey: "itemType.passport",
    icon: "Globe",
    iconBg: "bg-sky-500/15",
    iconColor: "text-sky-500",
    standardFields: {},
    customFields: [
      { name: "Full Name", labelKey: "itemType.fullName", type: "text" },
      { name: "Passport Number", labelKey: "itemType.passportNumber", type: "text" },
      { name: "Nationality", labelKey: "itemType.nationality", type: "text" },
      { name: "Issue Date", labelKey: "itemType.issueDate", type: "text" },
      { name: "Expiry Date", labelKey: "itemType.expiryDate", type: "text" },
    ],
    featured: false,
  },
  {
    code: "server",
    labelKey: "itemType.server",
    icon: "Server",
    iconBg: "bg-indigo-500/15",
    iconColor: "text-indigo-500",
    standardFields: { login: true, password: true },
    customFields: [
      { name: "IP Address", labelKey: "itemType.ipAddress", type: "text" },
      { name: "Port", labelKey: "itemType.port", type: "text" },
      { name: "OS", labelKey: "itemType.os", type: "text" },
    ],
    featured: false,
  },
];

/** Lookup item type by code */
export function getItemType(code: string): ItemTypeDef | undefined {
  return ITEM_TYPES.find((t) => t.code === code);
}

/** Extract item type code from custom fields */
export function getItemTypeFromCustoms(customs: CustomField[]): string | undefined {
  const field = customs.find((c) => c.name === "_itemType");
  return field?.value;
}

/** Featured types (shown in top grid) */
export const FEATURED_TYPES = ITEM_TYPES.filter((t) => t.featured);

/** Extended types (shown in bottom list) */
export const EXTENDED_TYPES = ITEM_TYPES.filter((t) => !t.featured);
