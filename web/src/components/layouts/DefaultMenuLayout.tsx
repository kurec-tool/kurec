import { Box } from '@mui/material';
import React from 'react';
import FloatingMenuButton from './menu/FloatingMenuButton';
import HeaderBar from './menu/HeaderBar';
import SidebarMenu from './menu/SidebarMenu';

const DefaultMenuLayout = ({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) => {
  const [isOpen, setIsOpen] = React.useState(false);
  const toggleDrawer = React.useCallback(() => {
    setIsOpen((prevIsOpen) => !prevIsOpen);
  }, []);

  return (
    <Box display="flex" flexDirection="column" height="100vh">
      <HeaderBar />
      <Box display="flex" flexGrow={1} overflow="auto">
        <Box flexGrow={1} mt={2} ml={2}>
          <SidebarMenu isOpen={isOpen} toggleDrawer={toggleDrawer} />
          <FloatingMenuButton isOpen={isOpen} toggleDrawer={toggleDrawer} />
          {children}
        </Box>
      </Box>
    </Box>
  );
};

export default DefaultMenuLayout;
