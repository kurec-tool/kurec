import CssBaseline from '@mui/joy/CssBaseline';
import InitColorSchemeScript from '@mui/joy/InitColorSchemeScript';
import { CssVarsProvider } from '@mui/joy/styles';
import type { Preview, StoryContext } from '@storybook/react';
// biome-ignore lint/style/useImportType: <explanation>
import React from 'react';
import { theme } from '../src/app/theme';
import '@fontsource/inter';

const withThemeProvider = (Story: React.FC, context: StoryContext) => {
  return (
    <>
      <CssBaseline />
      <InitColorSchemeScript defaultMode="system" />
      <CssVarsProvider
        // colorSchemeSelector="media"
        defaultMode="system"
        theme={theme}
      >
        <Story />
      </CssVarsProvider>
    </>
  );
};
const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  decorators: [withThemeProvider],
};

export default preview;
