"use client";

import type { FormEvent } from "react";
import { useEffect, useMemo, useRef, useState } from "react";
import { Pencil, Plus, Trash2, X, Folder, Settings as SettingsIcon, FolderOpen, Globe, MapPin } from "lucide-react";
import { AppShell } from "../../components/app-shell";
import { Alert, AlertDescription, AlertTitle } from "../../components/ui/alert";
import { Badge } from "../../components/ui/badge";
import { Button } from "../../components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui/card";
import { Input } from "../../components/ui/input";
import { Label } from "../../components/ui/label";
import { Spinner } from "../../components/ui/spinner";
import {
  createDevSession,
  getFolderSettings,
  updateFolderSettings,
  getSystemSettings,
  updateSystemSettings,
  detectLocation,
  type AllowedFolder,
  type FolderSettings,
  type SystemSettings
} from "../../lib/api";
import { ensureSessionToken } from "../../lib/session";

const emptyFolderSettings: FolderSettings = {
  folderConfigs: []
};

const defaultSystemSettings: SystemSettings = {
  timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC",
  location: undefined
};

type SettingsTab = "providers" | "folders" | "system";

export default function SettingsPage() {
  const [sessionToken, setSessionToken] = useState("");
  const [activeTab, setActiveTab] = useState<SettingsTab>("system");
  const [folderSettings, setFolderSettings] = useState<FolderSettings>(emptyFolderSettings);
  const [systemSettings, setSystemSettings] = useState<SystemSettings>(defaultSystemSettings);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [detecting, setDetecting] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [editingFolderId, setEditingFolderId] = useState<string | null>(null);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  const [folderPath, setFolderPath] = useState("");
  const [folderName, setFolderName] = useState("");
  const [isBase, setIsBase] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    ensureSessionToken(createDevSession)
      .then((token) => setSessionToken(token))
      .catch(() => setError("Failed to create development session."));
  }, []);

  useEffect(() => {
    if (!sessionToken) {
      return;
    }

    loadAllSettings();
  }, [sessionToken]);

  async function loadAllSettings() {
    if (!sessionToken) return;
    
    setLoading(true);
    try {
      const [folders, system] = await Promise.all([
        getFolderSettings(sessionToken),
        getSystemSettings(sessionToken)
      ]);
      setFolderSettings(folders);
      setSystemSettings(system);
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : "Failed to load settings");
    } finally {
      setLoading(false);
    }
  }

  const folderCards = useMemo(
    () => folderSettings.folderConfigs,
    [folderSettings.folderConfigs]
  );

  function resetForm() {
    setEditingFolderId(null);
    setFolderPath("");
    setFolderName("");
    setIsBase(false);
  }

  function openCreateModal() {
    setError("");
    setMessage("");
    resetForm();
    setModalOpen(true);
  }

  function openEditModal(folder: AllowedFolder) {
    setError("");
    setMessage("");
    setEditingFolderId(folder.id);
    setFolderPath(folder.path);
    setFolderName(folder.name);
    setIsBase(folder.isBase);
    setModalOpen(true);
  }

  function handleSelectFolder() {
    if (fileInputRef.current) {
      fileInputRef.current.click();
    }
  }

  function handleFolderChange(event: React.ChangeEvent<HTMLInputElement>) {
    const files = event.target.files;
    if (files && files.length > 0) {
      const file = files[0];
      const pathParts = file.webkitRelativePath.split('/');
      if (pathParts.length > 0) {
        const folderName = pathParts[0];
        setFolderName(folderName);
        
        const fullPath = file.webkitRelativePath;
        const pathWithoutFileName = fullPath.substring(0, fullPath.lastIndexOf('/'));
        setFolderPath(pathWithoutFileName || folderName);
      }
    }
  }

  async function saveFolderSettings(nextConfigs: AllowedFolder[]) {
    setSaving(true);
    try {
      const saved = await updateFolderSettings({
        sessionToken,
        settings: {
          folderConfigs: nextConfigs
        }
      });
      setFolderSettings(saved);
      setMessage("Folder settings saved.");
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Failed to save folder settings");
    } finally {
      setSaving(false);
    }
  }

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!folderPath.trim() || !folderName.trim()) {
      return;
    }

    setError("");
    setMessage("");

    const folderPayload: AllowedFolder = {
      id: editingFolderId ?? crypto.randomUUID(),
      path: folderPath.trim(),
      name: folderName.trim(),
      isBase,
      createdAt: Date.now()
    };

    let nextConfigs: AllowedFolder[];
    
    if (editingFolderId) {
      nextConfigs = folderSettings.folderConfigs.map((folder) =>
        folder.id === editingFolderId ? folderPayload : folder
      );
    } else {
      nextConfigs = [folderPayload, ...folderSettings.folderConfigs];
    }

    if (isBase) {
      nextConfigs = nextConfigs.map((folder) => ({
        ...folder,
        isBase: folder.id === folderPayload.id
      }));
    }

    await saveFolderSettings(nextConfigs);
    setModalOpen(false);
    resetForm();
  }

  async function onDelete(folderId: string) {
    setError("");
    setMessage("");
    const nextConfigs = folderSettings.folderConfigs.filter((folder) => folder.id !== folderId);
    await saveFolderSettings(nextConfigs);
  }

  async function handleSaveSystemSettings() {
    if (!sessionToken) return;
    setSaving(true);
    setError("");
    setMessage("");
    try {
      const saved = await updateSystemSettings({
        sessionToken,
        settings: systemSettings
      });
      setSystemSettings(saved);
      setMessage("System settings saved.");
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Failed to save system settings");
    } finally {
      setSaving(false);
    }
  }

  async function handleDetectLocation() {
    if (!sessionToken) return;
    setDetecting(true);
    setError("");
    try {
      const location = await detectLocation(sessionToken);
      if (location) {
        setSystemSettings({
          timezone: location.timezone,
          location: {
            city: location.city,
            country: location.country,
            countryCode: location.countryCode,
            latitude: location.latitude,
            longitude: location.longitude
          }
        });
        setMessage(`Location detected: ${location.city}, ${location.country}`);
      } else {
        setError("Could not detect location. Please enter manually.");
      }
    } catch (detectError) {
      setError(detectError instanceof Error ? detectError.message : "Failed to detect location");
    } finally {
      setDetecting(false);
    }
  }

  return (
    <AppShell active="settings" title="Settings">
      <section className="space-y-4">
        <div className="flex items-center gap-2 border-b border-slate-200">
          <button
            type="button"
            onClick={() => setActiveTab("system")}
            className={`flex items-center gap-2 px-4 py-3 text-sm font-medium transition ${
              activeTab === "system"
                ? "border-b-2 border-blue-600 text-blue-600"
                : "text-slate-500 hover:text-slate-700"
            }`}
          >
            <Globe className="h-4 w-4" />
            System
          </button>
          <button
            type="button"
            onClick={() => setActiveTab("folders")}
            className={`flex items-center gap-2 px-4 py-3 text-sm font-medium transition ${
              activeTab === "folders"
                ? "border-b-2 border-blue-600 text-blue-600"
                : "text-slate-500 hover:text-slate-700"
            }`}
          >
            <Folder className="h-4 w-4" />
            Folders
          </button>
          <button
            type="button"
            onClick={() => window.location.href = "/providers"}
            className={`flex items-center gap-2 px-4 py-3 text-sm font-medium transition ${
              activeTab === "providers"
                ? "border-b-2 border-blue-600 text-blue-600"
                : "text-slate-500 hover:text-slate-700"
            }`}
          >
            <SettingsIcon className="h-4 w-4" />
            Providers
          </button>
        </div>

        {activeTab === "system" && (
          <Card>
            <CardHeader>
              <CardTitle>System Settings</CardTitle>
              <CardDescription>
                Configure timezone and location for the system. These settings help the AI understand context better.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="timezone">Timezone</Label>
                  <Input
                    id="timezone"
                    value={systemSettings.timezone}
                    onChange={(e) => setSystemSettings({ ...systemSettings, timezone: e.target.value })}
                    placeholder="e.g., Asia/Shanghai"
                  />
                  <p className="text-xs text-slate-500">
                    Current system timezone: {Intl.DateTimeFormat().resolvedOptions().timeZone}
                  </p>
                </div>

                <div className="grid gap-2">
                  <div className="flex items-center justify-between">
                    <Label>Location</Label>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={handleDetectLocation}
                      disabled={!sessionToken || detecting}
                    >
                      {detecting ? <Spinner className="h-4 w-4" /> : <MapPin className="h-4 w-4" />}
                      {detecting ? "Detecting..." : "Auto Detect"}
                    </Button>
                  </div>
                  
                  {systemSettings.location ? (
                    <div className="rounded-lg border border-slate-200 bg-slate-50 p-4 space-y-2">
                      <div className="flex items-center gap-2">
                        <MapPin className="h-4 w-4 text-slate-500" />
                        <span className="font-medium">{systemSettings.location.city}, {systemSettings.location.country}</span>
                      </div>
                      <div className="text-sm text-slate-500">
                        {systemSettings.location.latitude?.toFixed(4)}, {systemSettings.location.longitude?.toFixed(4)}
                      </div>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => setSystemSettings({ ...systemSettings, location: undefined })}
                      >
                        Clear location
                      </Button>
                    </div>
                  ) : (
                    <p className="text-sm text-slate-500">
                      No location set. Click "Auto Detect" to detect your location by IP, or it will be used from system context.
                    </p>
                  )}
                </div>
              </div>

              <div className="flex justify-end">
                <Button
                  type="button"
                  onClick={handleSaveSystemSettings}
                  disabled={!sessionToken || saving}
                >
                  {saving ? <Spinner className="h-4 w-4" /> : null}
                  {saving ? "Saving..." : "Save Settings"}
                </Button>
              </div>
            </CardContent>
          </Card>
        )}

        {activeTab === "folders" && (
          <>
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-lg font-semibold text-slate-900">Allowed Folders</h2>
                <p className="text-sm text-slate-500">
                  Configure which folders the AI agent can access. One folder must be set as base for default operations.
                </p>
              </div>
              <Button type="button" onClick={openCreateModal} disabled={!sessionToken || saving}>
                <Plus className="h-4 w-4" />
                Add Folder
              </Button>
            </div>

            {loading ? (
              <Card>
                <CardContent className="py-6">
                  <p className="text-sm text-slate-500">Loading folder settings...</p>
                </CardContent>
              </Card>
            ) : folderCards.length === 0 ? (
              <Card>
                <CardContent className="py-6">
                  <p className="text-sm text-slate-500">
                    No folders configured yet. Add at least one folder to enable file operations.
                  </p>
                </CardContent>
              </Card>
            ) : (
              <div className="grid gap-4 lg:grid-cols-2">
                {folderCards.map((folder) => (
                  <Card key={folder.id} className="min-h-[180px] border-slate-200">
                    <CardHeader>
                      <div className="flex items-start justify-between gap-3">
                        <div className="space-y-2">
                          <div className="flex items-center gap-2">
                            <Folder className="h-4 w-4 text-slate-600" />
                            <CardTitle className="text-base">{folder.name}</CardTitle>
                            {folder.isBase && (
                              <Badge variant="default">Base</Badge>
                            )}
                          </div>
                          <CardDescription className="text-xs">
                            {folder.isBase
                              ? "Default folder for file operations"
                              : "Allowed folder for operations"}
                          </CardDescription>
                        </div>

                        <div className="flex items-center gap-1">
                          <button
                            type="button"
                            onClick={() => openEditModal(folder)}
                            className="rounded-md p-1.5 text-slate-500 transition hover:bg-slate-100 hover:text-slate-700"
                            aria-label={`Edit ${folder.name}`}
                          >
                            <Pencil className="h-4 w-4" />
                          </button>
                          <button
                            type="button"
                            onClick={() => onDelete(folder.id)}
                            className="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-50 hover:text-rose-600"
                            aria-label={`Delete ${folder.name}`}
                          >
                            <Trash2 className="h-4 w-4" />
                          </button>
                        </div>
                      </div>
                    </CardHeader>

                    <CardContent className="space-y-3">
                      <div className="space-y-1">
                        <p className="text-xs font-medium uppercase tracking-wide text-slate-500">Path</p>
                        <p className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700 break-all">
                          {folder.path}
                        </p>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </>
        )}

        {message ? (
          <Alert variant="success">
            <AlertTitle>Saved</AlertTitle>
            <AlertDescription>{message}</AlertDescription>
          </Alert>
        ) : null}

        {error ? (
          <Alert variant="destructive">
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}
      </section>

      {modalOpen ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-900/40 p-4">
          <div className="w-full max-w-xl rounded-xl border border-slate-200 bg-white shadow-xl">
            <div className="flex items-center justify-between border-b border-slate-200 px-5 py-4">
              <div>
                <h2 className="text-base font-semibold text-slate-900">
                  {editingFolderId ? "Edit Folder" : "Add Folder"}
                </h2>
                <p className="text-sm text-slate-500">
                  Configure folder access for AI agents. Set one as base for default operations.
                </p>
              </div>
              <button
                type="button"
                className="rounded-lg p-1 text-slate-500 transition hover:bg-slate-100"
                onClick={() => {
                  setModalOpen(false);
                  resetForm();
                }}
                aria-label="Close folder modal"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <form onSubmit={onSubmit} className="grid gap-4 px-5 py-4">
              <div className="grid gap-2">
                <Label htmlFor="folder-name">Display Name</Label>
                <Input
                  id="folder-name"
                  value={folderName}
                  onChange={(event) => setFolderName(event.target.value)}
                  placeholder="My Workspace"
                />
              </div>

              <div className="grid gap-2">
                <Label>Folder Path</Label>
                <div className="flex gap-2">
                  <Input
                    value={folderPath}
                    onChange={(event) => setFolderPath(event.target.value)}
                    placeholder="Click 'Browse' to select a folder"
                    className="flex-1"
                  />
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleSelectFolder}
                  >
                    <FolderOpen className="h-4 w-4 mr-2" />
                    Browse
                  </Button>
                </div>
                <input
                  ref={fileInputRef}
                  type="file"
                  onChange={handleFolderChange}
                  style={{ display: 'none' }}
                  // @ts-expect-error webkitdirectory is not in the type definition
                  webkitdirectory=""
                  directory=""
                />
                <p className="text-xs text-slate-500">
                  Click 'Browse' to select a folder or enter the path manually.
                </p>
              </div>

              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="is-base"
                  checked={isBase}
                  onChange={(event) => setIsBase(event.target.checked)}
                  className="h-4 w-4 rounded border-slate-300 text-blue-600 focus:ring-blue-500"
                />
                <Label htmlFor="is-base" className="text-sm">
                  Set as base folder (default for operations)
                </Label>
              </div>

              <div className="flex items-center justify-end gap-2 border-t border-slate-200 pt-3">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => {
                    setModalOpen(false);
                    resetForm();
                  }}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={
                    saving ||
                    !sessionToken ||
                    !folderPath.trim() ||
                    !folderName.trim()
                  }
                >
                  {saving ? <Spinner className="h-4 w-4" /> : null}
                  {saving ? "Saving..." : editingFolderId ? "Save Changes" : "Add Folder"}
                </Button>
              </div>
            </form>
          </div>
        </div>
      ) : null}
    </AppShell>
  );
}