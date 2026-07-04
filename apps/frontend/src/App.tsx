import AddIcon from '@mui/icons-material/Add';
import AutorenewIcon from '@mui/icons-material/Autorenew';
import CheckCircleOutlineIcon from '@mui/icons-material/CheckCircleOutline';
import ErrorOutlineIcon from '@mui/icons-material/ErrorOutline';
import FolderIcon from '@mui/icons-material/Folder';
import InboxIcon from '@mui/icons-material/Inbox';
import LanIcon from '@mui/icons-material/Lan';
import RouteIcon from '@mui/icons-material/Route';
import SettingsIcon from '@mui/icons-material/Settings';
import {
  Alert,
  AppBar,
  Box,
  Chip,
  CircularProgress,
  Container,
  FormControl,
  IconButton,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  Stack,
  Tab,
  Tabs,
  Toolbar,
  Tooltip,
  Typography
} from '@mui/material';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { io } from 'socket.io-client';

import { ConvertDialog } from './components/ConvertDialog';
import { Metric } from './components/Metric';
import { ProjectDialog } from './components/ProjectDialog';
import { ProjectSettings } from './components/ProjectSettings';
import { RoutesTable } from './components/RoutesTable';
import { RouteSettingsDialog } from './components/RouteSettingsDialog';
import { UnknownRequestsTable } from './components/UnknownRequestsTable';
import { SOCKET_IO_URL, UNKNOWN_REQUEST_CAPTURED_EVENT } from './config';
import { apiGet, apiPost, apiPut } from './lib/api';
import {
  errorMessage,
  mergeUnknownRequest,
  normalizeHttpMethod,
  parseJsonObject,
  profilePayload,
  projectPath,
  splitTags
} from './lib/forms';
import {
  emptyForm,
  type ConvertForm,
  type ListResponse,
  type MockRoute,
  type Project,
  type ProjectsResponse,
  type RouteForm,
  type RouteProfile,
  type UnknownRequest
} from './types';

export default function App() {
  const [tab, setTab] = useState(0);
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState<string>('');
  const [projectDialogOpen, setProjectDialogOpen] = useState(false);
  const [projectName, setProjectName] = useState('');
  const [defaultProxyEnabled, setDefaultProxyEnabled] = useState(false);
  const [defaultProxyUrl, setDefaultProxyUrl] = useState('');
  const [unknownRequests, setUnknownRequests] = useState<UnknownRequest[]>([]);
  const [routes, setRoutes] = useState<MockRoute[]>([]);
  const [selected, setSelected] = useState<UnknownRequest | null>(null);
  const [editingRoute, setEditingRoute] = useState<MockRoute | null>(null);
  const [routeProfiles, setRouteProfiles] = useState<RouteProfile[]>([]);
  const [routeForm, setRouteForm] = useState<RouteForm>({
    method: 'GET',
    pathPattern: '/',
    name: '',
    tags: '',
    status: 'active'
  });
  const [profileForm, setProfileForm] = useState<ConvertForm>(emptyForm);
  const [editingProfileId, setEditingProfileId] = useState<string | null>(null);
  const [form, setForm] = useState<ConvertForm>(emptyForm);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [realtimeConnected, setRealtimeConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const newUnknownCount = useMemo(
    () => unknownRequests.filter((request) => request.status === 'new').length,
    [unknownRequests]
  );
  const selectedProject = useMemo(
    () => projects.find((project) => project.id === selectedProjectId) ?? null,
    [projects, selectedProjectId]
  );

  const loadProjects = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await apiGet<ProjectsResponse>('/projects');
      setProjects(response.items);
      setSelectedProjectId((current) => current || response.active_project_id || response.items[0]?.id || '');
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setLoading(false);
    }
  }, []);

  const loadData = useCallback(async () => {
    if (!selectedProjectId) {
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const [unknownResponse, routesResponse] = await Promise.all([
        apiGet<ListResponse<UnknownRequest>>(projectPath('/unknown-requests', selectedProjectId)),
        apiGet<ListResponse<MockRoute>>(projectPath('/routes', selectedProjectId))
      ]);

      setUnknownRequests(unknownResponse.items);
      setRoutes(routesResponse.items);
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setLoading(false);
    }
  }, [selectedProjectId]);

  useEffect(() => {
    void loadProjects();
  }, [loadProjects]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  useEffect(() => {
    setDefaultProxyEnabled(selectedProject?.default_proxy_enabled ?? false);
    setDefaultProxyUrl(selectedProject?.default_proxy_url ?? '');
  }, [selectedProject?.id, selectedProject?.default_proxy_enabled, selectedProject?.default_proxy_url]);

  useEffect(() => {
    const socket = io(SOCKET_IO_URL, {
      path: '/socket.io',
      transports: ['websocket', 'polling']
    });

    socket.on('connect', () => setRealtimeConnected(true));
    socket.on('disconnect', () => setRealtimeConnected(false));
    socket.on(UNKNOWN_REQUEST_CAPTURED_EVENT, (request: UnknownRequest) => {
      if (request.project_id !== selectedProjectId) {
        return;
      }
      setUnknownRequests((current) => mergeUnknownRequest(current, request));
    });

    return () => {
      socket.disconnect();
    };
  }, [selectedProjectId]);

  async function selectProject(projectId: string) {
    setSelectedProjectId(projectId);
    setSelected(null);
    setEditingRoute(null);
    setError(null);
    setNotice(null);
    try {
      await apiPut(`/projects/${projectId}/active`, {});
    } catch (requestError) {
      setError(errorMessage(requestError));
    }
  }

  async function createProject() {
    if (!projectName.trim()) {
      setError('Project name cannot be empty');
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const project = await apiPost<Project>('/projects', { name: projectName.trim() });
      setProjects((current) => [...current, project].sort((left, right) => left.name.localeCompare(right.name)));
      setSelectedProjectId(project.id);
      setProjectName('');
      setProjectDialogOpen(false);
      setNotice('Project created');
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setSaving(false);
    }
  }

  async function rotateSelectedProjectKey() {
    if (!selectedProjectId) {
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const project = await apiPut<Project>(`/projects/${selectedProjectId}/key`, {});
      setProjects((current) =>
        current
          .map((item) => (item.id === project.id ? project : item))
          .sort((left, right) => left.name.localeCompare(right.name))
      );
      setNotice('Project key rotated');
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setSaving(false);
    }
  }

  async function saveSelectedProjectSettings() {
    if (!selectedProjectId) {
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const project = await apiPut<Project>(`/projects/${selectedProjectId}/settings`, {
        default_proxy_enabled: defaultProxyEnabled,
        default_proxy_url: defaultProxyUrl.trim() || null
      });
      setProjects((current) =>
        current
          .map((item) => (item.id === project.id ? project : item))
          .sort((left, right) => left.name.localeCompare(right.name))
      );
      setNotice('Project settings saved');
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setSaving(false);
    }
  }

  function openConvertDialog(request: UnknownRequest) {
    setSelected(request);
    setForm({
      ...emptyForm,
      name: `${request.method} ${request.path}`,
      responseBody: request.body_text || emptyForm.responseBody
    });
    setError(null);
    setNotice(null);
  }

  async function openRouteSettings(route: MockRoute) {
    setEditingRoute(route);
    setRouteForm({
      method: route.method,
      pathPattern: route.path_pattern,
      name: route.name,
      tags: route.tags.join(', '),
      status: route.status
    });
    setProfileForm(emptyForm);
    setEditingProfileId(null);
    setError(null);
    setNotice(null);

    try {
      const response = await apiGet<ListResponse<RouteProfile>>(`/routes/${route.id}/profiles`);
      setRouteProfiles(response.items);
    } catch (requestError) {
      setError(errorMessage(requestError));
    }
  }

  async function saveRouteSettings() {
    if (!editingRoute) {
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const route = await apiPut<MockRoute>(projectPath(`/routes/${editingRoute.id}`, selectedProjectId), {
        method: normalizeHttpMethod(routeForm.method),
        path_pattern: routeForm.pathPattern,
        name: routeForm.name,
        tags: splitTags(routeForm.tags),
        status: routeForm.status,
        active_scenario_id: editingRoute.active_scenario_id
      });
      setEditingRoute(route);
      setNotice('Route settings saved');
      await loadData();
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setSaving(false);
    }
  }

  async function saveProfile() {
    if (!editingRoute) {
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const payload = profilePayload(profileForm);
      if (editingProfileId) {
        await apiPut(`/routes/${editingRoute.id}/profiles/${editingProfileId}`, payload);
      } else {
        await apiPost(`/routes/${editingRoute.id}/profiles`, payload);
      }
      const response = await apiGet<ListResponse<RouteProfile>>(`/routes/${editingRoute.id}/profiles`);
      setRouteProfiles(response.items);
      setProfileForm(emptyForm);
      setEditingProfileId(null);
      setNotice('Profile saved');
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setSaving(false);
    }
  }

  async function activateProfile(profileId: string) {
    if (!editingRoute) {
      return;
    }

    setSaving(true);
    setError(null);
    try {
      const route = await apiPut<MockRoute>(`/routes/${editingRoute.id}/active-profile/${profileId}`, {});
      setEditingRoute(route);
      setNotice('Active profile switched');
      await loadData();
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setSaving(false);
    }
  }

  function editProfile(profile: RouteProfile) {
    setEditingProfileId(profile.id);
    setProfileForm({
      name: '',
      tags: '',
      scenarioName: profile.name,
      profileKind: profile.profile_kind,
      kind: profile.kind,
      proxyUrl: profile.proxy_url ?? '',
      proxyUrlMode: profile.proxy_url_mode ?? 'prefix',
      statusCode: profile.status_code,
      responseHeaders: JSON.stringify(profile.response_headers, null, 2),
      responseBody: profile.response_body ?? '',
      delayMs: profile.delay_ms
    });
  }

  async function convertSelected() {
    if (!selected) {
      return;
    }

    setSaving(true);
    setError(null);

    try {
      const responseHeaders = parseJsonObject(form.responseHeaders, 'Response headers');
      await apiPost(projectPath(`/unknown-requests/${selected.id}/convert`, selectedProjectId), {
        name: form.name || undefined,
        tags: splitTags(form.tags),
        scenario: profilePayload({ ...form, responseHeaders: JSON.stringify(responseHeaders, null, 2) })
      });

      setSelected(null);
      setNotice('Route saved');
      await loadData();
      setTab(1);
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Box className="appShell">
      <AppBar position="static" color="default" elevation={0} className="topBar">
        <Toolbar>
          <LanIcon color="primary" />
          <Typography variant="h6" component="h1" sx={{ ml: 1.5, fontWeight: 700 }}>
            Mock Machine
          </Typography>
          <FolderIcon fontSize="small" sx={{ ml: 2, mr: 1, color: 'text.secondary' }} />
          <FormControl size="small" sx={{ minWidth: 220 }} disabled={projects.length === 0}>
            <InputLabel id="project-select-label">Project</InputLabel>
            <Select
              labelId="project-select-label"
              label="Project"
              value={selectedProjectId}
              onChange={(event) => void selectProject(event.target.value)}
            >
              {projects.map((project) => (
                <MenuItem key={project.id} value={project.id}>
                  {project.name}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
          <Tooltip title="New project">
            <span>
              <IconButton sx={{ ml: 0.5 }} onClick={() => setProjectDialogOpen(true)}>
                <AddIcon />
              </IconButton>
            </span>
          </Tooltip>
          <Chip
            size="small"
            label={realtimeConnected ? 'Realtime' : 'Offline'}
            color={realtimeConnected ? 'success' : 'default'}
            variant={realtimeConnected ? 'filled' : 'outlined'}
            sx={{ ml: 2 }}
          />
          <Tooltip title="Refresh">
            <span>
              <IconButton sx={{ ml: 'auto' }} onClick={loadData} disabled={loading}>
                <AutorenewIcon />
              </IconButton>
            </span>
          </Tooltip>
        </Toolbar>
      </AppBar>

      <Container maxWidth="xl" sx={{ py: 3 }}>
        <Stack spacing={2.5}>
          {error && (
            <Alert severity="error" icon={<ErrorOutlineIcon />} onClose={() => setError(null)}>
              {error}
            </Alert>
          )}

          {notice && (
            <Alert severity="success" icon={<CheckCircleOutlineIcon />} onClose={() => setNotice(null)}>
              {notice}
            </Alert>
          )}

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <Metric icon={<InboxIcon />} label="Unknown" value={newUnknownCount} />
            <Metric icon={<RouteIcon />} label="Routes" value={routes.length} />
            <Metric
              icon={<SettingsIcon />}
              label="Converted"
              value={unknownRequests.filter((request) => request.status === 'converted').length}
            />
          </Stack>

          <Paper variant="outlined" className="workspacePanel">
            <Box className="panelHeader">
              <Tabs value={tab} onChange={(_, nextTab) => setTab(nextTab)} aria-label="admin sections">
                <Tab icon={<InboxIcon />} iconPosition="start" label="Unknown" />
                <Tab icon={<RouteIcon />} iconPosition="start" label="Routes" />
                <Tab icon={<SettingsIcon />} iconPosition="start" label="Project" />
              </Tabs>
              {loading && <CircularProgress size={22} />}
            </Box>

            {tab === 0 ? (
              <UnknownRequestsTable
                requests={unknownRequests}
                loading={loading}
                onConvert={openConvertDialog}
              />
            ) : tab === 1 ? (
              <RoutesTable routes={routes} loading={loading} onEdit={openRouteSettings} />
            ) : (
              <ProjectSettings
                project={selectedProject}
                saving={saving}
                defaultProxyEnabled={defaultProxyEnabled}
                onDefaultProxyEnabledChange={setDefaultProxyEnabled}
                defaultProxyUrl={defaultProxyUrl}
                onDefaultProxyUrlChange={setDefaultProxyUrl}
                onSaveSettings={saveSelectedProjectSettings}
                onRotateKey={rotateSelectedProjectKey}
              />
            )}
          </Paper>
        </Stack>
      </Container>

      <ConvertDialog
        request={selected}
        form={form}
        saving={saving}
        onChange={setForm}
        onClose={() => setSelected(null)}
        onSave={convertSelected}
      />
      <RouteSettingsDialog
        route={editingRoute}
        profiles={routeProfiles}
        routeForm={routeForm}
        profileForm={profileForm}
        editingProfileId={editingProfileId}
        saving={saving}
        onRouteFormChange={setRouteForm}
        onProfileFormChange={setProfileForm}
        onClose={() => setEditingRoute(null)}
        onSaveRoute={saveRouteSettings}
        onSaveProfile={saveProfile}
        onEditProfile={editProfile}
        onActivateProfile={activateProfile}
      />
      <ProjectDialog
        open={projectDialogOpen}
        name={projectName}
        saving={saving}
        onNameChange={setProjectName}
        onClose={() => setProjectDialogOpen(false)}
        onCreate={createProject}
      />
    </Box>
  );
}
