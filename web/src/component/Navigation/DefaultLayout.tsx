import Menu from '@mui/icons-material/Menu';
import {
  Box,
  Drawer,
  IconButton,
  List,
  ListItem,
  MenuItem,
  MenuList,
  ModalClose,
  Typography,
  styled,
} from '@mui/joy';
import React from 'react';
import SideMenu from './SideMenu';

import { Search } from '@mui/icons-material';
import { Cherry_Bomb_One } from 'next/font/google';
import Image from 'next/image';
import { useRouter } from 'next/navigation';
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
  marginTop: '4px',
  fontSize: '25px',
  color: '#fac8b0',
  backgroundColor: theme.palette.primary[300],
  display: 'inline',
  textShadow: '2px 2px 4px rgba(20, 20, 20, 0.5)',
}));

const HeaderBarBox = styled(Box)(({ theme }) => ({
  backgroundColor: theme.palette.primary[300],
  height: '48px',
}));

const Sidebar = styled(Box)(({ theme }) => ({
  padding: '12px',
}));

// サイドバー要素

// まだ無し

// レイアウト本体

const DefaultLayout: React.FC<{
  children: React.ReactNode;
  searchComponent?: JSX.Element;
}> = ({ children, searchComponent }) => {
  const router = useRouter();
  const [opened, setOpened] = React.useState(false);

  return (
    <Box sx={{ height: '100vh', overflow: 'auto' }}>
      <Drawer open={opened} onClose={() => setOpened(false)} size="sm">
        <ModalClose />
        <Sidebar>
          <Typography level="title-md">メニュー</Typography>
          <List>
            <ListItem onClick={() => router.push('/')}>ホーム</ListItem>
            <ListItem onClick={() => router.push('/search/epg')}>
              番組表検索
            </ListItem>
          </List>
        </Sidebar>
      </Drawer>
      <HeaderBarBox
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'flex-start',
          position: 'sticky',
          top: 0,
          zIndex: 1100,
          paddingRight: '12px',
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
      {children}
    </Box>
  );
};
export default DefaultLayout;
