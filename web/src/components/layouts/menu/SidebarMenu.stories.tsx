import type { Meta, StoryFn } from '@storybook/react';
import React from 'react';
import SidebarMenu from './SidebarMenu';

export default {
  title: 'Components/Layouts/Menu/SidebarMenu',
  component: SidebarMenu,
} as Meta;

const Template: StoryFn<typeof SidebarMenu> = (args) => (
  <SidebarMenu {...args} />
);

export const Default = Template.bind({});
Default.args = {
  // Add default props here
};

export const WithItems = Template.bind({});
WithItems.args = {
  // Add props with items here
};
