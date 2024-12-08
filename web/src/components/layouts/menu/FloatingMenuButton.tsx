import KuRecCircleIcon from '@/components/icons/KuRecCircleIcon';
import MenuIcon from '@mui/icons-material/Menu';
import { Fab } from '@mui/material';
import React from 'react';

type FloatingMenuButtonProps = Readonly<{
  toggleDrawer: () => void;
  isOpen: boolean;
}>;

const FloatingMenuButton = ({
  isOpen,
  toggleDrawer,
}: FloatingMenuButtonProps) => {
  return (
    <Fab
      color="primary"
      onClick={toggleDrawer}
      sx={{
        position: 'fixed',
        bottom: 16,
        left: 16,
        width: '40px',
        height: '40px',
        display: isOpen ? 'none' : 'flex',
      }}
    >
      <KuRecCircleIcon size="large" sx={{ marginBottom: '-10px' }} />
    </Fab>
  );
};

export default FloatingMenuButton;
