'use client';
import DefaultLayout from '@/component/Navigation/DefaultLayout';
import { instantMeiliSearch } from '@meilisearch/instant-meilisearch';
import { Typography } from '@mui/joy';
import { SearchBox } from 'react-instantsearch';
import { InstantSearchNext } from 'react-instantsearch-nextjs';
import 'instantsearch.css/themes/satellite.css';

const { searchClient } = instantMeiliSearch(
  // biome-ignore lint/style/noNonNullAssertion: <explanation>
  process.env.NEXT_PUBLIC_MEILISEARCH_URL!,
  process.env.NEXT_PUBLIC_MEILISEARCH_KEY,
);

export default function Home() {
  return (
    <DefaultLayout>
      <Typography>Meilisearchテスト</Typography>
      <InstantSearchNext indexName="" searchClient={searchClient}>
        <SearchBox />
      </InstantSearchNext>
    </DefaultLayout>
  );
}
