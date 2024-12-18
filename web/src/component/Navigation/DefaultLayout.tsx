import Menu from '@mui/icons-material/Menu';
import {
  Box,
  Drawer,
  IconButton,
  ModalClose,
  Typography,
  styled,
} from '@mui/joy';
import React from 'react';
import SideMenu from './SideMenu';

import { Search } from '@mui/icons-material';
import { Cherry_Bomb_One } from 'next/font/google';
import Image from 'next/image';
import KuRecIconImage from './KuRec-icon-transparent.webp';

// ヘッダーバー要素

const LogoIcon = styled(Image)(() => ({
  margin: '8px',
  width: '28px',
  height: '28px',
  display: 'inline',
}));

const LogoTextFont = Cherry_Bomb_One({
  subsets: ['latin'],
  weight: ['400'],
  display: 'block',
  variable: '--font-logo-text',
  fallback: ['sans-serif'],
});

const LogoText = styled(Typography)(({ theme }) => ({
  ...LogoTextFont.style,
  letterSpacing: '3px',
  margin: '8px',
  marginBottom: '7px',
  fontSize: '25px',
  color: theme.palette.mode === 'dark' ? '#0a0a0a' : '#f0f0f0',
  backgroundColor: theme.palette.primary[300],
  display: 'inline',
}));

const HeaderBarBox = styled(Box)(({ theme }) => ({
  backgroundColor: theme.palette.primary[300],
}));

// サイドバー要素

// まだ無し

// レイアウト本体

const DefaultLayout: React.FC<{
  children: React.ReactNode;
  searchComponent?: JSX.Element;
}> = ({ children, searchComponent }) => {
  const [opened, setOpened] = React.useState(false);

  return (
    <Box sx={{ height: '100vh', overflow: 'auto' }}>
      <Drawer open={opened} onClose={() => setOpened(false)} size="sm">
        <ModalClose />
        <h1>さいどばー</h1>
      </Drawer>
      <HeaderBarBox
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'flex-start',
          position: 'sticky',
          top: 0,
          zIndex: 1100,
        }}
      >
        <IconButton
          variant="outlined"
          color="primary"
          sx={{ marginLeft: '5px' }}
          onClick={() => {
            setOpened(true);
          }}
        >
          <Menu />
        </IconButton>
        {/* ロゴ要らないのでは説: <LogoIcon src={KuRecIconImage} alt="KuRec アイコン" /> */}
        <LogoText>KuRec</LogoText>
        {searchComponent}
      </HeaderBarBox>
      <Box padding={1}>{children}</Box>
    </Box>
  );
};
export default DefaultLayout;
