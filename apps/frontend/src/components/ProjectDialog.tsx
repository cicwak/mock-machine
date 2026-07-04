import CloseIcon from '@mui/icons-material/Close';
import SaveIcon from '@mui/icons-material/Save';
import { Button, Dialog, DialogActions, DialogContent, DialogTitle, IconButton, TextField, Typography } from '@mui/material';

interface ProjectDialogProps {
  open: boolean;
  name: string;
  saving: boolean;
  onNameChange: (name: string) => void;
  onClose: () => void;
  onCreate: () => void;
}

export function ProjectDialog({ open, name, saving, onNameChange, onClose, onCreate }: ProjectDialogProps) {
  return (
    <Dialog open={open} onClose={onClose} fullWidth maxWidth="xs">
      <DialogTitle className="dialogTitle">
        <Typography variant="h6" component="div">
          New project
        </Typography>
        <IconButton onClick={onClose} aria-label="Close">
          <CloseIcon />
        </IconButton>
      </DialogTitle>
      <DialogContent dividers>
        <TextField
          autoFocus
          label="Project name"
          value={name}
          onChange={(event) => onNameChange(event.target.value)}
          fullWidth
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button variant="contained" startIcon={<SaveIcon />} onClick={onCreate} disabled={saving}>
          Create
        </Button>
      </DialogActions>
    </Dialog>
  );
}
