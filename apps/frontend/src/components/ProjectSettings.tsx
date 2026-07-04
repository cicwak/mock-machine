import AutorenewIcon from '@mui/icons-material/Autorenew';
import SaveIcon from '@mui/icons-material/Save';
import { Box, Button, FormControlLabel, Stack, Switch, TextField, Typography } from '@mui/material';

import type { Project } from '../types';

export function ProjectSettings({
  project,
  saving,
  defaultProxyEnabled,
  onDefaultProxyEnabledChange,
  defaultProxyUrl,
  onDefaultProxyUrlChange,
  onSaveSettings,
  onRotateKey
}: {
  project: Project | null;
  saving: boolean;
  defaultProxyEnabled: boolean;
  onDefaultProxyEnabledChange: (value: boolean) => void;
  defaultProxyUrl: string;
  onDefaultProxyUrlChange: (value: string) => void;
  onSaveSettings: () => void;
  onRotateKey: () => void;
}) {
  if (!project) {
    return (
      <Box className="emptyCell">
        <Typography color="text.secondary">No project selected</Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ p: 2.5 }}>
      <Stack spacing={2.5} maxWidth={720}>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextField label="Project name" value={project.name} fullWidth InputProps={{ readOnly: true }} />
          <TextField label="Project ID" value={project.id} fullWidth InputProps={{ readOnly: true }} />
        </Stack>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems={{ md: 'center' }}>
          <TextField
            label="Project key"
            value={project.key}
            fullWidth
            className="monoInput"
            InputProps={{ readOnly: true }}
          />
          <Button
            variant="outlined"
            startIcon={<AutorenewIcon />}
            onClick={onRotateKey}
            disabled={saving}
            sx={{ minWidth: 150 }}
          >
            Rotate key
          </Button>
        </Stack>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} alignItems={{ md: 'center' }}>
          <FormControlLabel
            control={
              <Switch
                checked={defaultProxyEnabled}
                onChange={(event) => onDefaultProxyEnabledChange(event.target.checked)}
              />
            }
            label="Default upstream"
            sx={{ minWidth: 190 }}
          />
          <TextField
            label="Default upstream URL"
            value={defaultProxyUrl}
            onChange={(event) => onDefaultProxyUrlChange(event.target.value)}
            fullWidth
            placeholder="https://api.example.com"
            className="monoInput"
          />
          <Button
            variant="contained"
            startIcon={<SaveIcon />}
            onClick={onSaveSettings}
            disabled={saving}
            sx={{ minWidth: 150 }}
          >
            Save
          </Button>
        </Stack>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
          <TextField
            label="Header"
            value={`X-Mock-Project: ${project.key}`}
            fullWidth
            className="monoInput"
            InputProps={{ readOnly: true }}
          />
          <TextField
            label="Host key"
            value={`${project.key}.mock-machine.example.com`}
            fullWidth
            className="monoInput"
            InputProps={{ readOnly: true }}
          />
        </Stack>
      </Stack>
    </Box>
  );
}
