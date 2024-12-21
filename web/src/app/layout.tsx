import CssBaseline from '@mui/joy/CssBaseline';
import InitColorSchemeScript from '@mui/joy/InitColorSchemeScript';
import { CssVarsProvider } from '@mui/joy/styles';
import type { Metadata } from 'next';
import '@fontsource/inter';
import { theme } from './theme';

export const metadata: Metadata = {
  title: 'KuRec',
  description: 'KuRec - きゅーれっく',
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="ja" suppressHydrationWarning={true}>
      <head>
        <link rel="icon" href="/favicon.png" />
        {/* favicon.icoは書かずに置いておく */}
      </head>
      <body>
        <CssBaseline />
        <InitColorSchemeScript defaultMode="system" />
        <CssVarsProvider
          // colorSchemeSelector="media"
          defaultMode="system"
          theme={theme}
        >
          {children}
        </CssVarsProvider>
      </body>
    </html>
  );
}
