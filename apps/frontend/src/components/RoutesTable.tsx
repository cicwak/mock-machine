import SettingsIcon from '@mui/icons-material/Settings';
import { Button, Chip, Stack, Table, TableBody, TableCell, TableContainer, TableHead, TableRow } from '@mui/material';

import { routeStatusColors } from '../config';
import { formatDate } from '../lib/forms';
import type { MockRoute } from '../types';

export function RoutesTable({
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
                  color={routeStatusColors[route.status]}
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
