import AddIcon from '@mui/icons-material/Add';
import CloseIcon from '@mui/icons-material/Close';
import EditIcon from '@mui/icons-material/Edit';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import SaveIcon from '@mui/icons-material/Save';
import {
  Autocomplete,
  Box,
  Button,
  Chip,
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
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TextField,
  Tooltip,
  Typography
} from '@mui/material';

import { HTTP_METHOD_OPTIONS } from '../config';
import { normalizeHttpMethod } from '../lib/forms';
import type { ConvertForm, MockRoute, ProfileKind, RouteForm, RouteProfile, RouteStatus, ScenarioKind } from '../types';

export function RouteSettingsDialog({
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
