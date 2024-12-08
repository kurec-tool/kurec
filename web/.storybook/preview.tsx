import { CssBaseline, ThemeProvider } from '@mui/material';
import type { Preview, StoryContext } from '@storybook/react';
// biome-ignore lint/style/useImportType: なぜかtypeにすると上手くいかない
import React from 'react';
import { theme } from '../src/app/theme';

import '../src/app/globals.css';

const withThemeProvider = (Story: React.FC, context: StoryContext) => {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Story />
    </ThemeProvider>
  );
};

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/,
      },
    },
  },
  decorators: [withThemeProvider],
};

export default preview;
