import type { Meta, StoryFn } from '@storybook/react';
import KuRecLogo from './KuRecLogo';

export default {
  title: 'Components/Images/KuRecLogo',
  component: KuRecLogo,
} as Meta;

const Template: StoryFn<typeof KuRecLogo> = () => <KuRecLogo />;

export const Default = Template.bind({});
Default.args = {};
