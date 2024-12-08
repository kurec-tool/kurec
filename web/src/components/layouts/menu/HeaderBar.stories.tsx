import type { Meta, StoryFn } from '@storybook/react';
import React from 'react';
import HeaderBar from './HeaderBar';

const meta: Meta = {
  title: 'Components/Layouts/HeaderBar',
  component: HeaderBar,
  args: {},
  parameters: {
    layout: 'fullscreen',
  },
};
export default meta;

const Template: StoryFn<typeof HeaderBar> = (args) => <HeaderBar {...args} />;

export const Default = Template.bind({});
Default.args = {
  // Add default args here
};
