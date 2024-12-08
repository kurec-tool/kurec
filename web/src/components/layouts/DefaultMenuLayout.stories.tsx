import { Box, Paper } from '@mui/material';
import type { StoryFn } from '@storybook/react';
import React from 'react';
import DefaultMenuLayout from './DefaultMenuLayout';

export default {
  title: 'Components/Layouts/DefaultMenuLayout',
  component: DefaultMenuLayout,
  parameters: {
    layout: 'fullscreen',
  },
};

const Template: StoryFn<typeof DefaultMenuLayout> = () => (
  <DefaultMenuLayout>
    <Box>ダミー！</Box>
  </DefaultMenuLayout>
);

export const Default = Template.bind({});
Default.args = {
  // Add default props here
};
