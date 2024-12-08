import InboxIcon from '@mui/icons-material/Inbox';
import MailIcon from '@mui/icons-material/Mail';
import MenuOpenIcon from '@mui/icons-material/MenuOpen';
import {
  Button,
  Drawer,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  useMediaQuery,
} from '@mui/material';

type AppDrawerProps = Readonly<{
  isOpen: boolean;
  toggleDrawer: () => void;
}>;

const SidebarMenu = ({ isOpen, toggleDrawer }: AppDrawerProps) => {
  const isMobile = useMediaQuery('(max-width: 600px)');
  return (
    <Drawer
      variant={isMobile ? 'temporary' : 'permanent'}
      anchor={isMobile ? 'bottom' : 'left'}
      open={isOpen}
      onClose={toggleDrawer}
      sx={{
        '& .MuiDrawer-paper': {
          width: isMobile ? '100%' : 60,
          backgroundColor: '#fff',
          backdropFilter: 'blur(10px)', // グラスモーフィズム
        },
      }}
    >
      <List>
        {!isMobile && (
          <ListItem key="open-close" disablePadding>
            <Button fullWidth={!isOpen} onClick={toggleDrawer}>
              <ListItemIcon>
                <MenuOpenIcon />
              </ListItemIcon>
              {isOpen && <ListItemText primary="閉じる" />}
            </Button>
          </ListItem>
        )}
        {['Inbox', 'Starred', 'Send email', 'Drafts'].map((text, index) => (
          <ListItem key={text} disablePadding>
            <Button fullWidth={!isOpen}>
              <ListItemIcon>
                {index % 2 === 0 ? <InboxIcon /> : <MailIcon />}
              </ListItemIcon>
              {isOpen && <ListItemText primary={text} />}
            </Button>
          </ListItem>
        ))}
      </List>
    </Drawer>
  );
};

export default SidebarMenu;
