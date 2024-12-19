'use client';
import DefaultLayout from '@/component/Navigation/DefaultLayout';
import {
  type InstantMeiliSearchInstance,
  instantMeiliSearch,
} from '@meilisearch/instant-meilisearch';
import { Card, List, ListItem, Typography } from '@mui/joy';
import useInfiniteScroll from 'react-infinite-scroll-hook';
import { SearchBox, useInfiniteHits } from 'react-instantsearch';
import { InstantSearchNext } from 'react-instantsearch-nextjs';
import 'instantsearch.css/themes/satellite.css';
import { useEffect, useState } from 'react';

const CustomInfiniteHits = () => {
  const { items, isLastPage, showMore } = useInfiniteHits();
  const [hasNextPage, setHasNextPage] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setHasNextPage(!isLastPage);
  }, [isLastPage]);

  const [sentryRef] = useInfiniteScroll({
    loading,
    hasNextPage,
    onLoadMore: showMore,
    disabled: false,
    rootMargin: '0px 0px 100px 0px',
  });

  return (
    <List>
      {items.length === 0 && !loading && (
        <ListItem>該当する番組が見つかりませんでした</ListItem>
      )}
      {items.map((item) => (
        <ListItem key={item.program_id}>
          <Card>
            <Typography level="title-lg">{item.タイトル}</Typography>
            <Typography level="body-sm">
              {item.放送局} {item.開始時刻.toString()}～
              {item.終了時刻.toString()} {item.ジャンル.join(',')}
            </Typography>
            <Typography level="body-md">{item.番組情報}</Typography>
            <Typography level="body-xs">{item.program_id}</Typography>
          </Card>
        </ListItem>
      ))}
      {(loading || hasNextPage) && (
        <ListItem ref={sentryRef}>Now Loading...</ListItem>
      )}
    </List>
  );
};

export default function Home() {
  const [searchClient, setSearchClient] =
    useState<InstantMeiliSearchInstance>();

  useEffect(() => {
    const { searchClient } = instantMeiliSearch(
      // biome-ignore lint/style/noNonNullAssertion: <explanation>
      process.env.NEXT_PUBLIC_MEILISEARCH_URL!,
      process.env.NEXT_PUBLIC_MEILISEARCH_KEY,
    );
    setSearchClient(searchClient);
  }, []);
  return (
    <>
      {searchClient && (
        <InstantSearchNext indexName="kurec-epg" searchClient={searchClient}>
          <DefaultLayout searchComponent={<SearchBox autoFocus />}>
            {/* <InfiniteHits hitComponent={Hit} /> */}
            <CustomInfiniteHits />
          </DefaultLayout>
        </InstantSearchNext>
      )}
    </>
  );
}
