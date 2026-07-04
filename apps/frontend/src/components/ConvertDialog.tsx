import CloseIcon from '@mui/icons-material/Close';
import SaveIcon from '@mui/icons-material/Save';
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControl,
  IconButton,
  InputLabel,
  MenuItem,
  Select,
  Stack,
  TextField,
  Typography
} from '@mui/material';

import type { ConvertForm, ProfileKind, ProxyUrlMode, ScenarioKind, UnknownRequest } from '../types';

export function ConvertDialog({
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
            <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
              <FormControl fullWidth>
                <InputLabel id="convert-proxy-url-mode-label">URL mode</InputLabel>
                <Select
                  labelId="convert-proxy-url-mode-label"
                  label="URL mode"
                  value={form.proxyUrlMode}
                  onChange={(event) => onChange({ ...form, proxyUrlMode: event.target.value as ProxyUrlMode })}
                >
                  <MenuItem value="prefix">prefix</MenuItem>
                  <MenuItem value="static">static</MenuItem>
                </Select>
              </FormControl>
              <TextField
                label="Proxy URL"
                value={form.proxyUrl}
                onChange={(event) => onChange({ ...form, proxyUrl: event.target.value })}
                fullWidth
              />
            </Stack>
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
