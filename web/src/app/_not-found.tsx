import { Box, Typography } from '@mui/material';
import type React from 'react';

const NotFound: React.FC = () => {
  return (
    <Box>
      <Typography variant="h1">404 - Not Found</Typography>
      <Typography>見つからないよ～</Typography>
    </Box>
  );
};

export default NotFound;
