import type { Meta, StoryFn } from '@storybook/react';
import React from 'react';
import KuRecCircleIcon from './KuRecCircleIcon';

export default {
  title: 'Components/Icons/KuRecCircleIcon',
  component: KuRecCircleIcon,
} as Meta;

const Template: StoryFn = (args) => <KuRecCircleIcon size={args.size} />;

export const Small = Template.bind({});
Small.args = {
  size: 'small',
};

export const Large = Template.bind({});
Large.args = {
  size: 'large',
};
