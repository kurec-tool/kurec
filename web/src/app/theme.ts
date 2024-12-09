'use client';
import { extendTheme } from '@mui/joy';

export const theme = extendTheme({
  //   fontFamily: {
  // body: DynaPuff_400.variable,
  // display: DynaPuff_400.variable,
  //   },
  colorSchemes: {
    light: {
      palette: {
        primary: {
          '50': '#effcfb',
          '100': '#d7f6f5',
          '200': '#b4edec',
          '300': '#97e6e6',
          '400': '#44cacc',
          '500': '#29adb1',
          '600': '#258c95',
          '700': '#24727a',
          '800': '#255d65',
          '900': '#234e56',
        },
      },
    },
    dark: {
      palette: {
        primary: {
          '50': '#effcfb',
          '100': '#d7f6f5',
          '200': '#b4edec',
          '300': '#97e6e6',
          '400': '#44cacc',
          '500': '#29adb1',
          '600': '#258c95',
          '700': '#24727a',
          '800': '#255d65',
          '900': '#234e56',
        },
      },
    },
  },
});
