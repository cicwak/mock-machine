import { Box, Paper, Typography } from '@mui/material';
import type { ReactNode } from 'react';

export function Metric({ icon, label, value }: { icon: ReactNode; label: string; value: number }) {
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
