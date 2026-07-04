import AddIcon from '@mui/icons-material/Add';
import AutorenewIcon from '@mui/icons-material/Autorenew';
import CheckCircleOutlineIcon from '@mui/icons-material/CheckCircleOutline';
import CloseIcon from '@mui/icons-material/Close';
import EditIcon from '@mui/icons-material/Edit';
import ErrorOutlineIcon from '@mui/icons-material/ErrorOutline';
import FolderIcon from '@mui/icons-material/Folder';
import InboxIcon from '@mui/icons-material/Inbox';
import LanIcon from '@mui/icons-material/Lan';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import RouteIcon from '@mui/icons-material/Route';
import SaveIcon from '@mui/icons-material/Save';
import SettingsIcon from '@mui/icons-material/Settings';
import {
  Alert,
  Autocomplete,
  AppBar,
  Box,
  Button,
  Chip,
  CircularProgress,
  Container,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  FormControl,
  IconButton,
  InputLabel,
  MenuItem,
  Paper,
  Select,
  Stack,
  Tab,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Tabs,
  TextField,
  Toolbar,
  Tooltip,
  Typography
} from '@mui/material';
import { useCallback, useEffect, useMemo, useState, type ReactNode } from 'react';
import { io } from 'socket.io-client';

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ??
  (import.meta.env.DEV ? 'http://127.0.0.1:8080/mockadminapi' : '/mockadminapi');
const SOCKET_IO_URL = import.meta.env.VITE_SOCKET_IO_URL ?? socketIoOrigin(API_BASE_URL);
const UNKNOWN_REQUEST_CAPTURED_EVENT = 'unknown_request:captured';

const HTTP_METHODS = [
  'ACL',
  'BASELINE-CONTROL',
  'BIND',
  'CHECKIN',
  'CHECKOUT',
  'CONNECT',
  'COPY',
  'DELETE',
  'GET',
  'HEAD',
  'LABEL',
  'LINK',
  'LOCK',
  'MERGE',
  'MKACTIVITY',
  'MKCALENDAR',
  'MKCOL',
  'MKREDIRECTREF',
  'MKWORKSPACE',
  'MOVE',
  'OPTIONS',
  'ORDERPATCH',
  'PATCH',
  'POST',
  'PRI',
  'PROPFIND',
  'PROPPATCH',
  'PUT',
  'QUERY',
  'REBIND',
  'REPORT',
  'SEARCH',
  'TRACE',
  'UNBIND',
  'UNCHECKOUT',
  'UNLINK',
  'UNLOCK',
  'UPDATE',
  'UPDATEREDIRECTREF',
  'VERSION-CONTROL'
] as const;

const COMMON_HTTP_METHODS = [
  'CONNECT',
  'DELETE',
  'GET',
  'HEAD',
  'OPTIONS',
  'PATCH',
  'POST',
  'PUT',
  'QUERY',
  'TRACE'
] as const;

const HTTP_METHOD_OPTIONS = [
  ...COMMON_HTTP_METHODS.map((method) => ({ method, group: 'Common' })),
  ...HTTP_METHODS.filter((method) => !COMMON_HTTP_METHODS.includes(method as CommonHttpMethod)).map(
    (method) => ({ method, group: 'Other' })
  )
];

type UnknownRequestStatus = 'new' | 'ignored' | 'converted';
type ScenarioKind = 'success' | 'error' | 'timeout' | 'custom';
type RouteStatus = 'active' | 'disabled';
type ProfileKind = 'static' | 'dynamic';
type CommonHttpMethod = (typeof COMMON_HTTP_METHODS)[number];

interface Project {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

interface ProjectsResponse {
  items: Project[];
  active_project_id: string;
}

interface UnknownRequest {
  id: string;
  project_id: string;
  method: string;
  path: string;
  query: Record<string, string>;
  headers: Record<string, string>;
  body_base64: string | null;
  body_text: string | null;
  first_seen_at: string;
  last_seen_at: string;
  count: number;
  status: UnknownRequestStatus;
  converted_route_id: string | null;
}

interface MockRoute {
  id: string;
  project_id: string;
  method: string;
  path_pattern: string;
  name: string;
  tags: string[];
  status: RouteStatus;
  active_scenario_id: string | null;
  created_at: string;
  updated_at: string;
}

interface RouteProfile {
  id: string;
  route_id: string;
  name: string;
  profile_kind: ProfileKind;
  kind: ScenarioKind;
  proxy_url: string | null;
  status_code: number;
  response_headers: Record<string, string>;
  response_body: string | null;
  delay_ms: number;
  selection_rules: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

interface ListResponse<T> {
  items: T[];
}

interface ConvertForm {
  name: string;
  tags: string;
  scenarioName: string;
  profileKind: ProfileKind;
  kind: ScenarioKind;
  proxyUrl: string;
  statusCode: number;
  responseHeaders: string;
  responseBody: string;
  delayMs: number;
}

const emptyForm: ConvertForm = {
  name: '',
  tags: '',
  scenarioName: 'success',
  profileKind: 'static',
  kind: 'success',
  proxyUrl: '',
  statusCode: 200,
  responseHeaders: '{\n  "content-type": "application/json"\n}',
  responseBody: '{\n  "ok": true\n}',
  delayMs: 0
};

interface RouteForm {
  method: string;
  pathPattern: string;
  name: string;
  tags: string;
  status: RouteStatus;
}

const statusColors: Record<UnknownRequestStatus, 'default' | 'success' | 'warning'> = {
  new: 'warning',
  ignored: 'default',
  converted: 'success'
};

export default function App() {
  const [tab, setTab] = useState(0);
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedProjectId, setSelectedProjectId] = useState<string>('');
  const [projectDialogOpen, setProjectDialogOpen] = useState(false);
  const [projectName, setProjectName] = useState('');
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
              </Tabs>
              {loading && <CircularProgress size={22} />}
            </Box>

            {tab === 0 ? (
              <UnknownRequestsTable
                requests={unknownRequests}
                loading={loading}
                onConvert={openConvertDialog}
              />
            ) : (
              <RoutesTable routes={routes} loading={loading} onEdit={openRouteSettings} />
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
      <Dialog open={projectDialogOpen} onClose={() => setProjectDialogOpen(false)} fullWidth maxWidth="xs">
        <DialogTitle className="dialogTitle">
          <Typography variant="h6" component="div">
            New project
          </Typography>
          <IconButton onClick={() => setProjectDialogOpen(false)} aria-label="Close">
            <CloseIcon />
          </IconButton>
        </DialogTitle>
        <DialogContent dividers>
          <TextField
            autoFocus
            label="Project name"
            value={projectName}
            onChange={(event) => setProjectName(event.target.value)}
            fullWidth
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setProjectDialogOpen(false)}>Cancel</Button>
          <Button variant="contained" startIcon={<SaveIcon />} onClick={createProject} disabled={saving}>
            Create
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}

function Metric({ icon, label, value }: { icon: ReactNode; label: string; value: number }) {
  return (
    <Paper variant="outlined" className="metricPanel">
      <Box className="metricIcon">{icon}</Box>
      <Box>
        <Typography variant="body2" color="text.secondary">
          {label}
        </Typography>
        <Typography variant="h5" component="div" sx={{ fontWeight: 700 }}>
          {value}
        </Typography>
      </Box>
    </Paper>
  );
}

function UnknownRequestsTable({
  requests,
  loading,
  onConvert
}: {
  requests: UnknownRequest[];
  loading: boolean;
  onConvert: (request: UnknownRequest) => void;
}) {
  return (
    <TableContainer>
      <Table size="small" stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>Method</TableCell>
            <TableCell>Path</TableCell>
            <TableCell>Status</TableCell>
            <TableCell align="right">Count</TableCell>
            <TableCell>Last seen</TableCell>
            <TableCell>Body</TableCell>
            <TableCell align="right">Action</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {requests.map((request) => (
            <TableRow key={request.id} hover>
              <TableCell>
                <Chip size="small" label={request.method} className="methodChip" />
              </TableCell>
              <TableCell className="pathCell">{request.path}</TableCell>
              <TableCell>
                <Chip size="small" label={request.status} color={statusColors[request.status]} />
              </TableCell>
              <TableCell align="right">{request.count}</TableCell>
              <TableCell>{formatDate(request.last_seen_at)}</TableCell>
              <TableCell className="bodyCell">{request.body_text || request.body_base64 || ''}</TableCell>
              <TableCell align="right">
                <Button
                  size="small"
                  variant="contained"
                  startIcon={<SaveIcon />}
                  disabled={request.status === 'converted'}
                  onClick={() => onConvert(request)}
                >
                  Save
                </Button>
              </TableCell>
            </TableRow>
          ))}
          {!loading && requests.length === 0 && (
            <TableRow>
              <TableCell colSpan={7} align="center" className="emptyCell">
                No unknown requests
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

function RoutesTable({
  routes,
  loading,
  onEdit
}: {
  routes: MockRoute[];
  loading: boolean;
  onEdit: (route: MockRoute) => void;
}) {
  return (
    <TableContainer>
      <Table size="small" stickyHeader>
        <TableHead>
          <TableRow>
            <TableCell>Method</TableCell>
            <TableCell>Pattern</TableCell>
            <TableCell>Name</TableCell>
            <TableCell>Status</TableCell>
            <TableCell>Tags</TableCell>
            <TableCell>Updated</TableCell>
            <TableCell align="right">Action</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {routes.map((route) => (
            <TableRow key={route.id} hover>
              <TableCell>
                <Chip size="small" label={route.method} className="methodChip" />
              </TableCell>
              <TableCell className="pathCell">{route.path_pattern}</TableCell>
              <TableCell>{route.name}</TableCell>
              <TableCell>
                <Chip
                  size="small"
                  label={route.status}
                  color={route.status === 'active' ? 'success' : 'default'}
                />
              </TableCell>
              <TableCell>
                <Stack direction="row" spacing={0.75} flexWrap="wrap" useFlexGap>
                  {route.tags.map((tag) => (
                    <Chip key={tag} size="small" label={tag} variant="outlined" />
                  ))}
                </Stack>
              </TableCell>
              <TableCell>{formatDate(route.updated_at)}</TableCell>
              <TableCell align="right">
                <Button size="small" startIcon={<SettingsIcon />} onClick={() => onEdit(route)}>
                  Settings
                </Button>
              </TableCell>
            </TableRow>
          ))}
          {!loading && routes.length === 0 && (
            <TableRow>
              <TableCell colSpan={7} align="center" className="emptyCell">
                No routes
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

function ConvertDialog({
  request,
  form,
  saving,
  onChange,
  onClose,
  onSave
}: {
  request: UnknownRequest | null;
  form: ConvertForm;
  saving: boolean;
  onChange: (form: ConvertForm) => void;
  onClose: () => void;
  onSave: () => void;
}) {
  return (
    <Dialog open={Boolean(request)} onClose={onClose} fullWidth maxWidth="md">
      <DialogTitle className="dialogTitle">
        <Box>
          <Typography variant="h6" component="div">
            {request ? `${request.method} ${request.path}` : 'Save route'}
          </Typography>
          {request && (
            <Typography variant="body2" color="text.secondary">
              {request.id}
            </Typography>
          )}
        </Box>
        <IconButton onClick={onClose} aria-label="Close">
          <CloseIcon />
        </IconButton>
      </DialogTitle>
      <DialogContent dividers>
        <Stack spacing={2} sx={{ pt: 0.5 }}>
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextField
              label="Route name"
              value={form.name}
              onChange={(event) => onChange({ ...form, name: event.target.value })}
              fullWidth
            />
            <TextField
              label="Tags"
              value={form.tags}
              onChange={(event) => onChange({ ...form, tags: event.target.value })}
              fullWidth
            />
          </Stack>

          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextField
              label="Scenario"
              value={form.scenarioName}
              onChange={(event) => onChange({ ...form, scenarioName: event.target.value })}
              fullWidth
            />
            <FormControl fullWidth>
              <InputLabel id="profile-kind-label">Profile type</InputLabel>
              <Select
                labelId="profile-kind-label"
                label="Profile type"
                value={form.profileKind}
                onChange={(event) => onChange({ ...form, profileKind: event.target.value as ProfileKind })}
              >
                <MenuItem value="static">static</MenuItem>
                <MenuItem value="dynamic">dynamic</MenuItem>
              </Select>
            </FormControl>
            <FormControl fullWidth>
              <InputLabel id="scenario-kind-label">Kind</InputLabel>
              <Select
                labelId="scenario-kind-label"
                label="Kind"
                value={form.kind}
                onChange={(event) => onChange({ ...form, kind: event.target.value as ScenarioKind })}
              >
                <MenuItem value="success">success</MenuItem>
                <MenuItem value="error">error</MenuItem>
                <MenuItem value="timeout">timeout</MenuItem>
                <MenuItem value="custom">custom</MenuItem>
              </Select>
            </FormControl>
          </Stack>

          {form.profileKind === 'dynamic' && (
            <TextField
              label="Proxy URL"
              value={form.proxyUrl}
              onChange={(event) => onChange({ ...form, proxyUrl: event.target.value })}
              fullWidth
            />
          )}

          {form.profileKind === 'static' && (
            <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextField
              label="Status"
              type="number"
              value={form.statusCode}
              onChange={(event) => onChange({ ...form, statusCode: Number(event.target.value) })}
              fullWidth
              inputProps={{ min: 100, max: 599 }}
            />
            <TextField
              label="Delay ms"
              type="number"
              value={form.delayMs}
              onChange={(event) => onChange({ ...form, delayMs: Number(event.target.value) })}
              fullWidth
              inputProps={{ min: 0 }}
            />
            </Stack>
          )}

          {form.profileKind === 'static' && (
            <>
              <TextField
                label="Response headers"
                value={form.responseHeaders}
                onChange={(event) => onChange({ ...form, responseHeaders: event.target.value })}
                minRows={4}
                multiline
                fullWidth
                className="monoInput"
              />

              <TextField
                label="Response body"
                value={form.responseBody}
                onChange={(event) => onChange({ ...form, responseBody: event.target.value })}
                minRows={8}
                multiline
                fullWidth
                className="monoInput"
              />
            </>
          )}

          {form.profileKind === 'dynamic' && (
            <TextField
              label="Delay ms"
              type="number"
              value={form.delayMs}
              onChange={(event) => onChange({ ...form, delayMs: Number(event.target.value) })}
              fullWidth
              inputProps={{ min: 0 }}
            />
          )}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button variant="contained" startIcon={<SaveIcon />} onClick={onSave} disabled={saving}>
          Save
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function RouteSettingsDialog({
  route,
  profiles,
  routeForm,
  profileForm,
  editingProfileId,
  saving,
  onRouteFormChange,
  onProfileFormChange,
  onClose,
  onSaveRoute,
  onSaveProfile,
  onEditProfile,
  onActivateProfile
}: {
  route: MockRoute | null;
  profiles: RouteProfile[];
  routeForm: RouteForm;
  profileForm: ConvertForm;
  editingProfileId: string | null;
  saving: boolean;
  onRouteFormChange: (form: RouteForm) => void;
  onProfileFormChange: (form: ConvertForm) => void;
  onClose: () => void;
  onSaveRoute: () => void;
  onSaveProfile: () => void;
  onEditProfile: (profile: RouteProfile) => void;
  onActivateProfile: (profileId: string) => void;
}) {
  return (
    <Dialog open={Boolean(route)} onClose={onClose} fullWidth maxWidth="lg">
      <DialogTitle className="dialogTitle">
        <Box>
          <Typography variant="h6" component="div">
            {route ? `${route.method} ${route.path_pattern}` : 'Route settings'}
          </Typography>
          {route && (
            <Typography variant="body2" color="text.secondary">
              {route.id}
            </Typography>
          )}
        </Box>
        <IconButton onClick={onClose} aria-label="Close">
          <CloseIcon />
        </IconButton>
      </DialogTitle>
      <DialogContent dividers>
        <Stack spacing={2.5}>
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <Autocomplete
              freeSolo
              fullWidth
              selectOnFocus
              clearOnBlur={false}
              handleHomeEndKeys
              options={HTTP_METHOD_OPTIONS}
              groupBy={(option) => option.group}
              getOptionLabel={(option) => (typeof option === 'string' ? option : option.method)}
              value={routeForm.method}
              inputValue={routeForm.method}
              onChange={(_, value) =>
                onRouteFormChange({
                  ...routeForm,
                  method: typeof value === 'string' ? value : value?.method ?? ''
                })
              }
              onInputChange={(_, value) => onRouteFormChange({ ...routeForm, method: value })}
              renderInput={(params) => (
                <TextField
                  {...params}
                  label="Method"
                  onBlur={() =>
                    onRouteFormChange({ ...routeForm, method: normalizeHttpMethod(routeForm.method) })
                  }
                />
              )}
            />
            <TextField
              label="Path pattern"
              value={routeForm.pathPattern}
              onChange={(event) => onRouteFormChange({ ...routeForm, pathPattern: event.target.value })}
              fullWidth
              className="monoInput"
            />
          </Stack>
          <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
            <TextField
              label="Route name"
              value={routeForm.name}
              onChange={(event) => onRouteFormChange({ ...routeForm, name: event.target.value })}
              fullWidth
            />
            <TextField
              label="Tags"
              value={routeForm.tags}
              onChange={(event) => onRouteFormChange({ ...routeForm, tags: event.target.value })}
              fullWidth
            />
            <FormControl fullWidth>
              <InputLabel id="route-status-label">Status</InputLabel>
              <Select
                labelId="route-status-label"
                label="Status"
                value={routeForm.status}
                onChange={(event) => onRouteFormChange({ ...routeForm, status: event.target.value as RouteStatus })}
              >
                <MenuItem value="active">active</MenuItem>
                <MenuItem value="disabled">disabled</MenuItem>
              </Select>
            </FormControl>
          </Stack>
          <Box>
            <Button variant="contained" startIcon={<SaveIcon />} onClick={onSaveRoute} disabled={saving}>
              Save route
            </Button>
          </Box>

          <Divider />

          <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
            Profiles
          </Typography>
          <TableContainer component={Paper} variant="outlined">
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>Name</TableCell>
                  <TableCell>Type</TableCell>
                  <TableCell>Target / response</TableCell>
                  <TableCell align="right">Delay</TableCell>
                  <TableCell align="right">Action</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {profiles.map((profile) => (
                  <TableRow key={profile.id} hover selected={profile.id === route?.active_scenario_id}>
                    <TableCell>{profile.name}</TableCell>
                    <TableCell>
                      <Chip size="small" label={profile.profile_kind} color={profile.profile_kind === 'dynamic' ? 'primary' : 'default'} />
                    </TableCell>
                    <TableCell className="bodyCell">
                      {profile.profile_kind === 'dynamic'
                        ? profile.proxy_url
                        : `${profile.status_code} ${profile.response_body ?? ''}`}
                    </TableCell>
                    <TableCell align="right">{profile.delay_ms}</TableCell>
                    <TableCell align="right">
                      <Tooltip title="Edit profile">
                        <IconButton size="small" onClick={() => onEditProfile(profile)}>
                          <EditIcon fontSize="small" />
                        </IconButton>
                      </Tooltip>
                      <Tooltip title="Set active">
                        <span>
                          <IconButton
                            size="small"
                            onClick={() => onActivateProfile(profile.id)}
                            disabled={profile.id === route?.active_scenario_id || saving}
                          >
                            <PlayArrowIcon fontSize="small" />
                          </IconButton>
                        </span>
                      </Tooltip>
                    </TableCell>
                  </TableRow>
                ))}
                {profiles.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={5} align="center" className="emptyCell">
                      No profiles
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </TableContainer>

          <Stack spacing={2}>
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
              {editingProfileId ? 'Edit profile' : 'New profile'}
            </Typography>
            <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
              <TextField
                label="Profile name"
                value={profileForm.scenarioName}
                onChange={(event) => onProfileFormChange({ ...profileForm, scenarioName: event.target.value })}
                fullWidth
              />
              <FormControl fullWidth>
                <InputLabel id="settings-profile-kind-label">Profile type</InputLabel>
                <Select
                  labelId="settings-profile-kind-label"
                  label="Profile type"
                  value={profileForm.profileKind}
                  onChange={(event) => onProfileFormChange({ ...profileForm, profileKind: event.target.value as ProfileKind })}
                >
                  <MenuItem value="static">static</MenuItem>
                  <MenuItem value="dynamic">dynamic</MenuItem>
                </Select>
              </FormControl>
              <TextField
                label="Delay ms"
                type="number"
                value={profileForm.delayMs}
                onChange={(event) => onProfileFormChange({ ...profileForm, delayMs: Number(event.target.value) })}
                fullWidth
                inputProps={{ min: 0 }}
              />
            </Stack>
            {profileForm.profileKind === 'dynamic' ? (
              <TextField
                label="Proxy URL"
                value={profileForm.proxyUrl}
                onChange={(event) => onProfileFormChange({ ...profileForm, proxyUrl: event.target.value })}
                fullWidth
              />
            ) : (
              <>
                <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
                  <TextField
                    label="Status"
                    type="number"
                    value={profileForm.statusCode}
                    onChange={(event) => onProfileFormChange({ ...profileForm, statusCode: Number(event.target.value) })}
                    fullWidth
                    inputProps={{ min: 100, max: 599 }}
                  />
                  <FormControl fullWidth>
                    <InputLabel id="settings-scenario-kind-label">Kind</InputLabel>
                    <Select
                      labelId="settings-scenario-kind-label"
                      label="Kind"
                      value={profileForm.kind}
                      onChange={(event) => onProfileFormChange({ ...profileForm, kind: event.target.value as ScenarioKind })}
                    >
                      <MenuItem value="success">success</MenuItem>
                      <MenuItem value="error">error</MenuItem>
                      <MenuItem value="timeout">timeout</MenuItem>
                      <MenuItem value="custom">custom</MenuItem>
                    </Select>
                  </FormControl>
                </Stack>
                <TextField
                  label="Response headers"
                  value={profileForm.responseHeaders}
                  onChange={(event) => onProfileFormChange({ ...profileForm, responseHeaders: event.target.value })}
                  minRows={3}
                  multiline
                  fullWidth
                  className="monoInput"
                />
                <TextField
                  label="Response body"
                  value={profileForm.responseBody}
                  onChange={(event) => onProfileFormChange({ ...profileForm, responseBody: event.target.value })}
                  minRows={6}
                  multiline
                  fullWidth
                  className="monoInput"
                />
              </>
            )}
            <Box>
              <Button variant="outlined" startIcon={<AddIcon />} onClick={onSaveProfile} disabled={saving}>
                {editingProfileId ? 'Save profile' : 'Add profile'}
              </Button>
            </Box>
          </Stack>
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Close</Button>
      </DialogActions>
    </Dialog>
  );
}

async function apiGet<T>(path: string): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`);
  return parseResponse<T>(response);
}

async function apiPost<T>(path: string, body: unknown): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify(body)
  });

  return parseResponse<T>(response);
}

async function apiPut<T>(path: string, body: unknown): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    method: 'PUT',
    headers: {
      'content-type': 'application/json'
    },
    body: JSON.stringify(body)
  });

  return parseResponse<T>(response);
}

async function parseResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const payload = await response.json().catch(() => null);
    throw new Error(payload?.error ?? `${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<T>;
}

function profilePayload(form: ConvertForm) {
  const headers =
    form.profileKind === 'static' ? parseJsonObject(form.responseHeaders, 'Response headers') : {};

  return {
    name: form.scenarioName,
    profile_kind: form.profileKind,
    kind: form.kind,
    proxy_url: form.profileKind === 'dynamic' ? form.proxyUrl : undefined,
    status_code: form.statusCode,
    response_headers: headers,
    response_body: form.profileKind === 'static' ? form.responseBody : undefined,
    delay_ms: form.delayMs,
    selection_rules: {}
  };
}

function splitTags(value: string): string[] {
  return value
    .split(',')
    .map((tag) => tag.trim())
    .filter(Boolean);
}

function normalizeHttpMethod(value: string): string {
  return value.trim().toUpperCase();
}

function projectPath(path: string, projectId: string): string {
  const separator = path.includes('?') ? '&' : '?';
  return `${path}${separator}project_id=${encodeURIComponent(projectId)}`;
}

function mergeUnknownRequest(requests: UnknownRequest[], next: UnknownRequest): UnknownRequest[] {
  const byId = new Map(requests.map((request) => [request.id, request]));
  byId.set(next.id, next);

  return Array.from(byId.values()).sort(
    (left, right) => new Date(right.last_seen_at).getTime() - new Date(left.last_seen_at).getTime()
  );
}

function socketIoOrigin(apiBaseUrl: string): string {
  if (apiBaseUrl.startsWith('http://') || apiBaseUrl.startsWith('https://')) {
    return new URL(apiBaseUrl).origin;
  }

  return window.location.origin;
}

function parseJsonObject(value: string, label: string): Record<string, string> {
  const parsed = JSON.parse(value);
  if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') {
    throw new Error(`${label} must be a JSON object`);
  }

  return parsed;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : 'Request failed';
}

function formatDate(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short'
  }).format(new Date(value));
}
