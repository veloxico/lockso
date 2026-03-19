import { useState, useEffect, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Trash2, Eye, EyeOff, ChevronDown, ChevronUp } from "lucide-react";
import {
  Dialog,
  DialogHeader,
  DialogTitle,
  DialogContent,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { PasswordGenerator } from "./password-generator";
import { BreachIndicator } from "./breach-indicator";
import { ItemTypePicker } from "./item-type-picker";
import { VAULT_COLORS } from "@/lib/colors";
import { cn } from "@/lib/utils";
import { itemApi } from "@/api/vaults";
import { toApiError } from "@/lib/api-error";
import { getItemType, type ItemTypeDef } from "@/lib/item-types";
import type { ItemView, CustomField } from "@/types/vault";

interface ItemFormDialogProps {
  open: boolean;
  mode: "create" | "edit";
  vaultId: string;
  folderId?: string | null;
  /** For edit mode */
  item?: ItemView | null;
  onClose: () => void;
  onSuccess: () => void;
}

const CUSTOM_FIELD_TYPES: CustomField["type"][] = [
  "text",
  "password",
  "url",
  "email",
  "totp",
];

export function ItemFormDialog({
  open,
  mode,
  vaultId,
  folderId,
  item,
  onClose,
  onSuccess,
}: ItemFormDialogProps) {
  const { t } = useTranslation();

  // Type picker state (create mode only)
  const [showTypePicker, setShowTypePicker] = useState(false);
  const [selectedType, setSelectedType] = useState<ItemTypeDef | null>(null);

  // Form fields
  const [name, setName] = useState("");
  const [login, setLogin] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [url, setUrl] = useState("");
  const [description, setDescription] = useState("");
  const [tags, setTags] = useState("");
  const [colorCode, setColorCode] = useState(0);
  const [customs, setCustoms] = useState<CustomField[]>([]);
  const [showGenerator, setShowGenerator] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  // When dialog opens, decide whether to show type picker
  useEffect(() => {
    if (!open) return;

    if (mode === "edit" && item) {
      // Edit mode — populate from item
      setName(item.name);
      setLogin(item.login);
      setPassword(item.password);
      setUrl(item.url);
      setDescription(item.description);
      setTags(item.tags.join(", "));
      setColorCode(item.colorCode);
      setCustoms(item.customs.filter((c) => c.name !== "_itemType").map((c) => ({ ...c })));
      setShowTypePicker(false);

      // Restore type from hidden field
      const typeField = item.customs.find((c) => c.name === "_itemType");
      setSelectedType(typeField ? getItemType(typeField.value) ?? null : null);
    } else {
      // Create mode — show type picker first
      setName("");
      setLogin("");
      setPassword("");
      setUrl("");
      setDescription("");
      setTags("");
      setColorCode(0);
      setCustoms([]);
      setSelectedType(null);
      setShowTypePicker(true);
    }
    setShowPassword(false);
    setShowGenerator(false);
    setError("");
  }, [open, mode, item]);

  const handleTypeSelected = (typeDef: ItemTypeDef) => {
    setSelectedType(typeDef);
    setShowTypePicker(false);

    // Pre-populate custom fields from template
    const templateCustoms: CustomField[] = typeDef.customFields.map((f) => ({
      name: f.name,
      value: "",
      type: f.type,
    }));
    setCustoms(templateCustoms);
  };

  // Determine which standard fields to show
  const showLogin = selectedType ? !!selectedType.standardFields.login : true;
  const showPasswordField = selectedType ? !!selectedType.standardFields.password : true;
  const showUrl = selectedType ? !!selectedType.standardFields.url : true;

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) {
      setError(t("item.errorNameRequired"));
      return;
    }

    setLoading(true);
    setError("");

    const parsedTags = tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);

    // Build customs: user-visible fields + hidden _itemType
    const visibleCustoms = customs.filter((c) => c.name.trim() && c.value.trim());
    const allCustoms: CustomField[] = [
      ...visibleCustoms,
      ...(selectedType
        ? [{ name: "_itemType", value: selectedType.code, type: "text" as const }]
        : []),
    ];

    try {
      if (mode === "create") {
        await itemApi.create({
          vaultId,
          folderId: folderId || undefined,
          name: trimmedName,
          login: login.trim() || undefined,
          password: password || undefined,
          url: url.trim() || undefined,
          description: description.trim() || undefined,
          tags: parsedTags.length > 0 ? parsedTags : undefined,
          customs: allCustoms.length > 0 ? allCustoms : undefined,
          colorCode,
        });
      } else if (item) {
        // Always send customs array on edit (even empty) so backend clears removed fields
        await itemApi.update(item.id, {
          name: trimmedName,
          login: login.trim() ?? "",
          password: password ?? "",
          url: url.trim() ?? "",
          description: description.trim() ?? "",
          tags: parsedTags,
          customs: allCustoms,
          colorCode,
          folderId: folderId || undefined,
        });
      }
      onSuccess();
      onClose();
    } catch (err) {
      const apiErr = toApiError(err);
      setError(apiErr.message || t("item.errorSaveFailed"));
    } finally {
      setLoading(false);
    }
  };

  const addCustomField = () => {
    setCustoms([...customs, { name: "", value: "", type: "text" }]);
  };

  const removeCustomField = (index: number) => {
    setCustoms(customs.filter((_, i) => i !== index));
  };

  const updateCustomField = (
    index: number,
    field: Partial<CustomField>,
  ) => {
    setCustoms(
      customs.map((c, i) => (i === index ? { ...c, ...field } : c)),
    );
  };

  // If showing type picker, render it instead of the form
  if (showTypePicker && mode === "create") {
    return (
      <ItemTypePicker
        open={open}
        onClose={onClose}
        onSelect={handleTypeSelected}
      />
    );
  }

  return (
    <Dialog open={open} onClose={onClose}>
      <form onSubmit={handleSubmit}>
        <DialogHeader>
          <DialogTitle>
            {mode === "create" ? t("item.createTitle") : t("item.editTitle")}
            {selectedType && (
              <span className="ml-2 text-sm font-normal text-muted-foreground">
                — {t(selectedType.labelKey)}
              </span>
            )}
          </DialogTitle>
        </DialogHeader>

        <DialogContent>
          <div className="max-h-[60vh] overflow-y-auto space-y-4 pr-1">
            {/* Name */}
            <div className="space-y-2">
              <Label htmlFor="item-name">{t("item.nameLabel")}</Label>
              <Input
                id="item-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder={t("item.namePlaceholder")}
                maxLength={255}
                autoFocus
              />
            </div>

            {/* Login (conditional) */}
            {showLogin && (
              <div className="space-y-2">
                <Label htmlFor="item-login">
                  {t("item.loginLabel")}{" "}
                  <span className="text-muted-foreground font-normal">({t("common.optional")})</span>
                </Label>
                <Input
                  id="item-login"
                  value={login}
                  onChange={(e) => setLogin(e.target.value)}
                  placeholder={t("item.loginPlaceholder")}
                  maxLength={255}
                />
              </div>
            )}

            {/* Password (conditional) */}
            {showPasswordField && (
              <div className="space-y-2">
                <Label htmlFor="item-password">
                  {t("item.passwordLabel")}{" "}
                  <span className="text-muted-foreground font-normal">({t("common.optional")})</span>
                </Label>
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <Input
                      id="item-password"
                      type={showPassword ? "text" : "password"}
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      placeholder={t("item.passwordPlaceholder")}
                      maxLength={256}
                    />
                    <button
                      type="button"
                      onClick={() => setShowPassword(!showPassword)}
                      className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                    >
                      {showPassword ? (
                        <EyeOff className="h-4 w-4" />
                      ) : (
                        <Eye className="h-4 w-4" />
                      )}
                    </button>
                  </div>
                </div>

                <BreachIndicator password={password} />

                {/* Generator toggle */}
                <button
                  type="button"
                  onClick={() => setShowGenerator(!showGenerator)}
                  className="flex items-center gap-1 text-xs text-primary hover:underline"
                >
                  {showGenerator ? (
                    <ChevronUp className="h-3 w-3" />
                  ) : (
                    <ChevronDown className="h-3 w-3" />
                  )}
                  {t("item.generator")}
                </button>

                {showGenerator && (
                  <PasswordGenerator
                    onUse={(pw) => {
                      setPassword(pw);
                    }}
                  />
                )}
              </div>
            )}

            {/* URL (conditional) */}
            {showUrl && (
              <div className="space-y-2">
                <Label htmlFor="item-url">
                  {t("item.urlLabel")}{" "}
                  <span className="text-muted-foreground font-normal">({t("common.optional")})</span>
                </Label>
                <Input
                  id="item-url"
                  type="url"
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  placeholder="https://example.com"
                  maxLength={2048}
                />
              </div>
            )}

            {/* Type-specific custom fields (with labels from type template) */}
            {customs.length > 0 && selectedType && (
              <div className="space-y-3">
                <Label className="text-sm font-medium">{t("item.typeFields")}</Label>
                {customs.map((field, i) => {
                  const template = selectedType.customFields[i];
                  const isPassword = field.type === "password";
                  return (
                    <div key={i} className="space-y-1">
                      <Label className="text-xs text-muted-foreground">
                        {template ? t(template.labelKey) : field.name}
                      </Label>
                      <div className="flex gap-2">
                        <Input
                          value={field.value}
                          type={isPassword ? "password" : "text"}
                          onChange={(e) =>
                            updateCustomField(i, { value: e.target.value })
                          }
                          placeholder={template?.placeholderKey ? t(template.placeholderKey) : ""}
                          maxLength={5000}
                        />
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => removeCustomField(i)}
                          className="shrink-0"
                        >
                          <Trash2 className="h-4 w-4 text-destructive" />
                        </Button>
                      </div>
                    </div>
                  );
                })}
              </div>
            )}

            {/* Description */}
            <div className="space-y-2">
              <Label htmlFor="item-desc">
                {t("item.descriptionLabel")}{" "}
                <span className="text-muted-foreground font-normal">({t("common.optional")})</span>
              </Label>
              <Textarea
                id="item-desc"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder={t("item.descriptionPlaceholder")}
                maxLength={5000}
                rows={3}
              />
            </div>

            {/* Tags */}
            <div className="space-y-2">
              <Label htmlFor="item-tags">
                {t("item.tagsLabel")}{" "}
                <span className="text-muted-foreground font-normal">({t("common.optional")})</span>
              </Label>
              <Input
                id="item-tags"
                value={tags}
                onChange={(e) => setTags(e.target.value)}
                placeholder={t("item.tagsPlaceholder")}
              />
              <p className="text-xs text-muted-foreground">{t("item.tagsHint")}</p>
            </div>

            {/* Color */}
            <div className="space-y-2">
              <Label>{t("vault.colorLabel")}</Label>
              <div className="flex flex-wrap gap-2">
                {VAULT_COLORS.map((c, i) => (
                  <button
                    key={i}
                    type="button"
                    onClick={() => setColorCode(i)}
                    className={cn(
                      "h-7 w-7 rounded-full transition-all",
                      c.bg,
                      colorCode === i
                        ? `ring-2 ${c.ring} ring-offset-2 ring-offset-background`
                        : "hover:scale-110",
                    )}
                    title={c.label}
                  />
                ))}
              </div>
            </div>

            {/* Additional custom fields (free-form) */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <Label>{t("item.customFields")}</Label>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={addCustomField}
                >
                  <Plus className="h-3.5 w-3.5" />
                  {t("item.addField")}
                </Button>
              </div>

              {/* Show only non-template custom fields for free-form editing */}
              {customs
                .map((field, i) => ({ field, i }))
                .filter(({ i }) => !selectedType || i >= (selectedType?.customFields.length ?? 0))
                .map(({ field, i }) => (
                  <div key={i} className="flex gap-2 items-start">
                    <div className="flex-1 space-y-1">
                      <Input
                        value={field.name}
                        onChange={(e) =>
                          updateCustomField(i, { name: e.target.value })
                        }
                        placeholder={t("item.fieldName")}
                        maxLength={100}
                      />
                      <Input
                        value={field.value}
                        type={field.type === "password" ? "password" : "text"}
                        onChange={(e) =>
                          updateCustomField(i, { value: e.target.value })
                        }
                        placeholder={t("item.fieldValue")}
                        maxLength={5000}
                      />
                    </div>
                    <select
                      value={field.type}
                      onChange={(e) =>
                        updateCustomField(i, {
                          type: e.target.value as CustomField["type"],
                        })
                      }
                      className="mt-0.5 h-10 rounded-md border border-input bg-background px-2 text-sm"
                    >
                      {CUSTOM_FIELD_TYPES.map((ft) => (
                        <option key={ft} value={ft}>
                          {ft}
                        </option>
                      ))}
                    </select>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => removeCustomField(i)}
                      className="mt-0.5 shrink-0"
                    >
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </div>
                ))}
            </div>

            {error && <p className="text-sm text-destructive">{error}</p>}
          </div>
        </DialogContent>

        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={loading}>
            {t("vault.cancel")}
          </Button>
          <Button type="submit" disabled={loading || !name.trim()}>
            {loading && <Spinner size="sm" />}
            {mode === "create" ? t("item.create") : t("item.save")}
          </Button>
        </DialogFooter>
      </form>
    </Dialog>
  );
}
