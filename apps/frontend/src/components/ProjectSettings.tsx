import AutorenewIcon from '@mui/icons-material/Autorenew';
import { Box, Button, Stack, TextField, Typography } from '@mui/material';

import type { Project } from '../types';

export function ProjectSettings({
  project,
  saving,
  onRotateKey
}: {
  project: Project | null;
  saving: boolean;
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
