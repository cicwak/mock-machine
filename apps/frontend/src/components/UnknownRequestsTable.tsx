import SaveIcon from '@mui/icons-material/Save';
import { Button, Chip, Table, TableBody, TableCell, TableContainer, TableHead, TableRow } from '@mui/material';

import { unknownStatusColors } from '../config';
import { formatDate } from '../lib/forms';
import type { UnknownRequest } from '../types';

export function UnknownRequestsTable({
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
                <Chip size="small" label={request.status} color={unknownStatusColors[request.status]} />
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
