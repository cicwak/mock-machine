import CheckCircleOutlineIcon from '@mui/icons-material/CheckCircleOutline';
import LanIcon from '@mui/icons-material/Lan';
import StorageIcon from '@mui/icons-material/Storage';
import {
  Alert,
  AppBar,
  Box,
  Chip,
  Container,
  Divider,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Paper,
  Stack,
  Toolbar,
  Typography
} from '@mui/material';

const services = [
  'nginx routes /mockadmin, /mockadminapi and public mock paths',
  'backend exposes /mockadminapi/health',
  'PostgreSQL starts with the initial mock schema',
  'Redis is ready for active route and scenario cache',
  'MinIO creates a private bucket and app read/write user'
];

export default function App() {
  return (
    <Box className="appShell">
      <AppBar position="static" color="default" elevation={0} className="topBar">
        <Toolbar>
          <LanIcon color="primary" />
          <Typography variant="h6" component="h1" sx={{ ml: 1.5, fontWeight: 700 }}>
            Mock Machine
          </Typography>
          <Chip label="local compose" size="small" sx={{ ml: 'auto' }} />
        </Toolbar>
      </AppBar>

      <Container maxWidth="md" sx={{ py: 4 }}>
        <Stack spacing={3}>
          <Alert severity="success" icon={<CheckCircleOutlineIcon />}>
            The monorepo shell is ready. Backend, frontend, PostgreSQL, Redis, MinIO and nginx are wired for local development.
          </Alert>

          <Paper variant="outlined" sx={{ p: 3 }}>
            <Stack direction="row" spacing={1.5} alignItems="center">
              <StorageIcon color="primary" />
              <Box>
                <Typography variant="h5" component="h2" sx={{ fontWeight: 700 }}>
                  Infrastructure status
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  This placeholder will become the admin dashboard for routes, scenarios and unknown requests.
                </Typography>
              </Box>
            </Stack>

            <Divider sx={{ my: 2 }} />

            <List disablePadding>
              {services.map((service) => (
                <ListItem key={service} disableGutters>
                  <ListItemIcon sx={{ minWidth: 34 }}>
                    <CheckCircleOutlineIcon color="success" fontSize="small" />
                  </ListItemIcon>
                  <ListItemText primary={service} />
                </ListItem>
              ))}
            </List>
          </Paper>
        </Stack>
      </Container>
    </Box>
  );
}
