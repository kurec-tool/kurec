'use client';
import DefaultLayout from '@/component/Navigation/DefaultLayout';
import { instantMeiliSearch } from '@meilisearch/instant-meilisearch';
import { Typography } from '@mui/joy';
import { InstantSearch, SearchBox } from 'react-instantsearch';

const { searchClient } = instantMeiliSearch(
  // biome-ignore lint/style/noNonNullAssertion: <explanation>
  process.env.NEXT_PUBLIC_MEILISEARCH_URL!,
  process.env.NEXT_PUBLIC_MEILISEARCH_KEY,
);

function EpgSearch() {
  return (
    <InstantSearch indexName="" searchClient={searchClient}>
      <SearchBox />
    </InstantSearch>
  );
}

export default function Home() {
  return (
    <DefaultLayout>
      <Typography>Meilisearchテスト</Typography>
    </DefaultLayout>
  );
}
