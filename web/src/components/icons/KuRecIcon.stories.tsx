import type { Meta, StoryFn } from '@storybook/react';
import React from 'react';
import KuRecIcon from './KuRecIcon';

export default {
  title: 'Components/Icons/KuRecIcon',
  component: KuRecIcon,
} as Meta;

const Template: StoryFn = (args) => <KuRecIcon size={args.size} />;

export const Small = Template.bind({});
Small.args = {
  size: 'small',
};

export const Large = Template.bind({});
Large.args = {
  size: 'large',
};
