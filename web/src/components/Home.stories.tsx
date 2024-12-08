import Providers from '@/app/providers';
import type { Meta, StoryFn } from '@storybook/react';
import React from 'react';
import Home from '../app/page';

export default {
  title: 'Example/Home',
  component: Home,
  parameters: {
    layout: 'fullscreen',
  },
} as Meta<typeof Home>;

const Template: StoryFn<typeof Home> = () => (
  <Providers>
    <Home />
  </Providers>
);

export const Default = Template.bind({});
Default.args = {
  // Add default args here
};
