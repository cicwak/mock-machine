import AutorenewIcon from '@mui/icons-material/Autorenew';
import CheckCircleOutlineIcon from '@mui/icons-material/CheckCircleOutline';
import CloseIcon from '@mui/icons-material/Close';
import ErrorOutlineIcon from '@mui/icons-material/ErrorOutline';
import InboxIcon from '@mui/icons-material/Inbox';
import LanIcon from '@mui/icons-material/Lan';
import RouteIcon from '@mui/icons-material/Route';
import SaveIcon from '@mui/icons-material/Save';
import SettingsIcon from '@mui/icons-material/Settings';
import {
  Alert,
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

const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ??
  (import.meta.env.DEV ? 'http://127.0.0.1:8080/mockadminapi' : '/mockadminapi');

type UnknownRequestStatus = 'new' | 'ignored' | 'converted';
type ScenarioKind = 'success' | 'error' | 'timeout' | 'custom';
type RouteStatus = 'active' | 'disabled';

interface UnknownRequest {
  id: string;
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
  method: string;
  path_pattern: string;
  name: string;
  tags: string[];
  status: RouteStatus;
  active_scenario_id: string | null;
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
  kind: ScenarioKind;
  statusCode: number;
  responseHeaders: string;
  responseBody: string;
  delayMs: number;
}

const emptyForm: ConvertForm = {
  name: '',
  tags: '',
  scenarioName: 'success',
  kind: 'success',
  statusCode: 200,
  responseHeaders: '{\n  "content-type": "application/json"\n}',
  responseBody: '{\n  "ok": true\n}',
  delayMs: 0
};

const statusColors: Record<UnknownRequestStatus, 'default' | 'success' | 'warning'> = {
  new: 'warning',
  ignored: 'default',
  converted: 'success'
};

export default function App() {
  const [tab, setTab] = useState(0);
  const [unknownRequests, setUnknownRequests] = useState<UnknownRequest[]>([]);
  const [routes, setRoutes] = useState<MockRoute[]>([]);
  const [selected, setSelected] = useState<UnknownRequest | null>(null);
  const [form, setForm] = useState<ConvertForm>(emptyForm);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const newUnknownCount = useMemo(
    () => unknownRequests.filter((request) => request.status === 'new').length,
    [unknownRequests]
  );

  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const [unknownResponse, routesResponse] = await Promise.all([
        apiGet<ListResponse<UnknownRequest>>('/unknown-requests'),
        apiGet<ListResponse<MockRoute>>('/routes')
      ]);

      setUnknownRequests(unknownResponse.items);
      setRoutes(routesResponse.items);
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadData();
  }, [loadData]);

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

  async function convertSelected() {
    if (!selected) {
      return;
    }

    setSaving(true);
    setError(null);

    try {
      const responseHeaders = parseJsonObject(form.responseHeaders, 'Response headers');
      await apiPost(`/unknown-requests/${selected.id}/convert`, {
        name: form.name || undefined,
        tags: form.tags
          .split(',')
          .map((tag) => tag.trim())
          .filter(Boolean),
        scenario: {
          name: form.scenarioName,
          kind: form.kind,
          status_code: form.statusCode,
          response_headers: responseHeaders,
          response_body: form.responseBody,
          delay_ms: form.delayMs,
          selection_rules: {}
        }
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
              <RoutesTable routes={routes} loading={loading} />
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

function RoutesTable({ routes, loading }: { routes: MockRoute[]; loading: boolean }) {
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
            </TableRow>
          ))}
          {!loading && routes.length === 0 && (
            <TableRow>
              <TableCell colSpan={6} align="center" className="emptyCell">
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

async function parseResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const payload = await response.json().catch(() => null);
    throw new Error(payload?.error ?? `${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<T>;
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
