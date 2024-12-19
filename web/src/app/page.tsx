'use client';
import DefaultLayout from '@/component/Navigation/DefaultLayout';
import { Box, Typography, styled } from '@mui/joy';

const HomeBox = styled(Box)(() => ({
  backgroundImage: 'url(/KuRec-logo.webp)',
  backgroundSize: 'contain',
  backgroundRepeat: 'no-repeat',
  backgroundPosition: 'center',
  width: '100%',
  minHeight: 'calc(100vh - 48px - 16px)', // Assuming the header height is 64px
  margin: '0',
  padding: '8px',
}));

export default function Home() {
  return (
    <DefaultLayout>
      <HomeBox>
        <Typography level="title-lg">KuRec - nantoka kantoka</Typography>
        <Typography level="body-md">
          ここに適当な長い日本語のダミーテキストが入ります。これはテスト用のテキストであり、実際のコンテンツではありません。ダミーテキストは、デザインやレイアウトの確認のために使用されます。このテキストは、文章の長さやフォーマットを確認するためのものであり、特定の意味を持ちません。ダミーテキストを使用することで、実際のコンテンツが入る前にデザインのバランスや視覚的な要素を調整することができます。このようなテキストは、しばしば「フィラー」や「プレースホルダー」としても知られています。
        </Typography>
      </HomeBox>
    </DefaultLayout>
  );
}
