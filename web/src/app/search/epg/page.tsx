'use client';
import DefaultLayout from '@/component/Navigation/DefaultLayout';
import {
  type InstantMeiliSearchInstance,
  instantMeiliSearch,
} from '@meilisearch/instant-meilisearch';
import { Card, Typography } from '@mui/joy';
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
  // スクロールイベントで次のページをロード
  useEffect(() => {
    const handleScroll = () => {
      const nearBottom =
        window.innerHeight + window.scrollY >= document.body.offsetHeight - 100;

      console.log('loading more?', nearBottom, isLastPage);
      if (nearBottom && !isLastPage) {
        showMore();
      }
    };

    console.log('add event listener');
    document.body.addEventListener('scroll', handleScroll);

    return () => {
      console.log('remove event listener');
      document.body.removeEventListener('scroll', handleScroll);
    };
  }, [isLastPage, showMore]);

  return (
    <div>
      {items.map((hit) => (
        <Card key={hit.program_id}>
          <Typography level="title-lg">{hit.タイトル}</Typography>
          <Typography level="body-sm">
            {hit.放送局} {hit.開始時刻.toString()}～{hit.終了時刻.toString()}{' '}
            {hit.ジャンル.join(',')}
          </Typography>
          <Typography level="body-md">{hit.番組情報}</Typography>
          <Typography level="body-xs">{hit.program_id}</Typography>
        </Card>
      ))}
      {isLastPage && <p>No more results</p>}
    </div>
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
            <InfiniteHits hitComponent={Hit} />
            {/* 上手く行かなかったので一旦コメントアウト: <CustomInfiniteHits /> */}
          </DefaultLayout>
        </InstantSearchNext>
      )}
    </>
  );
}
