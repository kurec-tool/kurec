import { Drawer, ModalClose } from '@mui/joy';

type SideMenuProps = {
  opened: boolean;
  handleClosed: () => void;
};

export default function SideMenu(props: SideMenuProps) {
  return (
    <Drawer open={props.opened} onClose={props.handleClosed} size="sm">
      <ModalClose />
      <h1>さいどばー</h1>
    </Drawer>
  );
}
