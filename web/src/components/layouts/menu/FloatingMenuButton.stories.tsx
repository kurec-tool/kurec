import type { StoryFn } from '@storybook/react';
import React from 'react';
import FloatingMenuButton from './FloatingMenuButton';

export default {
  title: 'Components/Layouts/Menu/FloatingMenuButton',
  component: FloatingMenuButton,
};

const Template: StoryFn<typeof FloatingMenuButton> = (args) => (
  <div style={{ position: 'fixed', bottom: '20px', left: '20px' }}>
    <FloatingMenuButton {...args} />
  </div>
);

export const Default = Template.bind({});
Default.args = {
  // Add any default props for FloatingMenuButton here
  toggleDrawer: () => console.log('toggleDrawer'),
};
