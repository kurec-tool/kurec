'use client';

import HeaderBar from '@/components/layouts/menu/HeaderBar';
import { CssBaseline } from '@mui/material';
import { AppRouterCacheProvider } from '@mui/material-nextjs/v13-appRouter';
import { ThemeProvider } from '@mui/material/styles';
import { theme } from './theme';

const Providers = ({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) => {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline>
        <AppRouterCacheProvider>
          <HeaderBar />
          {children}
        </AppRouterCacheProvider>
      </CssBaseline>
    </ThemeProvider>
  );
};

export default Providers;
