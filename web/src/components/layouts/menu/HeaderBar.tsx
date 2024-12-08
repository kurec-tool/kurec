'use client';
import { AppBar, Button, Toolbar, Typography, styled } from '@mui/material';
import type React from 'react';
import KuRecIcon from '../../icons/KuRecIcon';

const ThemedTypography = styled(Typography)(({ theme }) => ({
  color: theme.palette.mode === 'dark' ? 'black' : 'white',
  fontSize: '24px',
}));

const HeaderBar: React.FC = () => {
  return (
    <AppBar position="static">
      <Toolbar>
        <KuRecIcon size="small" />
        <ThemedTypography
          variant="h6"
          style={{ flexGrow: 1, marginLeft: '10px', marginBottom: '5px' }}
          sx={{}}
        >
          KuRec
        </ThemedTypography>
      </Toolbar>
    </AppBar>
  );
};

export default HeaderBar;
