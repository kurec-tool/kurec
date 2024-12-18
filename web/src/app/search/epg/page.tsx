'use client';
import DefaultLayout from '@/component/Navigation/DefaultLayout';
import {
  type InstantMeiliSearchInstance,
  instantMeiliSearch,
} from '@meilisearch/instant-meilisearch';
import { Card, List, ListItem, Typography } from '@mui/joy';
import useInfiniteScroll from 'react-infinite-scroll-hook';
import { InfiniteHits, SearchBox, useInfiniteHits } from 'react-instantsearch';
import { InstantSearchNext } from 'react-instantsearch-nextjs';
import 'instantsearch.css/themes/satellite.css';
import { useEffect, useRef, useState } from 'react';

function Hit({
  hit,
}: {
  hit: {
    program_id: string;
    タイトル: string;
    ジャンル: string[];
    番組情報: string;
    放送局: string;
    開始時刻: Date;
    終了時刻: Date;
  };
}) {
  return (
    <Card>
      <Typography level="title-lg">{hit.タイトル}</Typography>
      <Typography level="body-sm">
        {hit.放送局} {hit.開始時刻.toString()}～{hit.終了時刻.toString()}{' '}
        {hit.ジャンル.join(',')}
      </Typography>
      <Typography level="body-md">{hit.番組情報}</Typography>
      <Typography level="body-xs">{hit.program_id}</Typography>
    </Card>
  );
}

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
        <ListItem ref={sentryRef}>
          <Typography>Loading...</Typography>
        </ListItem>
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
          <DefaultLayout searchComponent={<SearchBox />}>
            {/* <InfiniteHits hitComponent={Hit} /> */}
            <CustomInfiniteHits />
          </DefaultLayout>
        </InstantSearchNext>
      )}
    </>
  );
}
