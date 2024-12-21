'use client';
import DefaultLayout from '@/component/Navigation/DefaultLayout';
import {
  type InstantMeiliSearchInstance,
  instantMeiliSearch,
} from '@meilisearch/instant-meilisearch';
import {
  Box,
  Card,
  Chip,
  Input,
  List,
  ListItem,
  Stack,
  Typography,
} from '@mui/joy';
import useInfiniteScroll from 'react-infinite-scroll-hook';
import {
  Highlight,
  RefinementList,
  useInfiniteHits,
  useInstantSearch,
  useSearchBox,
} from 'react-instantsearch';
import { InstantSearchNext } from 'react-instantsearch-nextjs';
import 'instantsearch.css/themes/satellite.css';
import AddIcon from '@mui/icons-material/Add';
import SearchIcon from '@mui/icons-material/Search';
import { useCallback, useEffect, useState } from 'react';

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
      {items.map((item) => {
        const startDateTime = new Date(item.開始時刻);
        const endDateTime = new Date(item.終了時刻);
        console.log(
          `[${item.開始日時}]`,
          startDateTime,
          item.終了日時,
          endDateTime,
        );
        const startDate = `${startDateTime.getMonth() + 1}/${startDateTime.getDate()}`;
        const startTime = `${startDateTime.getHours()}:${startDateTime.getMinutes()}`;
        const endTime = `${endDateTime.getHours()}:${endDateTime.getMinutes()}`;

        return (
          <ListItem key={item.program_id}>
            <Card sx={{ width: '100%' }}>
              <Stack direction="row" spacing={2}>
                {item.ogp_url_hash && (
                  <Box>
                    <img
                      src={`http://localhost:3000/api/ogp/${item.ogp_url_hash}`}
                      alt="公式サイト画像"
                      style={{ maxWidth: '150px', display: 'inline' }}
                    />
                  </Box>
                )}
                <Box>
                  <Typography level="title-lg">
                    <Highlight attribute="タイトル" hit={item} />
                  </Typography>
                  <Typography level="body-sm">
                    {item.放送局} {startDate} {startTime}～{endTime}{' '}
                    {item.ジャンル.join(', ')}
                  </Typography>
                </Box>
              </Stack>
              <Typography level="body-md">
                <Highlight attribute="番組情報" hit={item} />
              </Typography>
              {/* TODO: なんか良い感じに表示する */}
              {/* <Typography level="body-sm">
                {(item.その他情報 as string).split('\n').map((line) => (
                  <p key={line}>{line}</p>
                ))}
              </Typography> */}
              <Typography level="body-xs">{item.program_id}</Typography>
            </Card>
          </ListItem>
        );
      })}
      {(loading || hasNextPage) && (
        <ListItem ref={sentryRef}>Now Loading...</ListItem>
      )}
    </List>
  );
};

function CustomSearchBox({
  inputValue,
  setInputValue,
}: { inputValue: string; setInputValue: (v: string) => void }) {
  const { clear, query, refine } = useSearchBox();
  const status = useInstantSearch();

  const handleAdd = useCallback(() => {
    console.log('追加！:', inputValue);
  }, [inputValue]);

  return (
    <Box>
      <Input
        placeholder="検索..."
        startDecorator={<SearchIcon />}
        endDecorator={
          inputValue && (
            <Chip onClick={handleAdd} startDecorator={<AddIcon />}>
              ルールに追加
            </Chip>
          )
        }
        autoFocus={true}
        onChange={(e) => {
          setInputValue(e.target.value);
          refine(e.target.value);
        }}
        value={inputValue}
      />
    </Box>
  );
}

function GenreFilter() {
  return (
    <Box>
      <Typography level="title-md">ジャンル</Typography>
      <RefinementList attribute="ジャンル" limit={255} searchable={true} />
    </Box>
  );
}

export default function EpgSearch() {
  const [searchClient, setSearchClient] =
    useState<InstantMeiliSearchInstance>();
  const [inputValue, setInputValue] = useState('');

  useEffect(() => {
    const { searchClient, meiliSearchInstance, setMeiliSearchParams } =
      instantMeiliSearch(
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
          <DefaultLayout
            searchComponent={
              <CustomSearchBox
                inputValue={inputValue}
                setInputValue={setInputValue}
              />
            }
          >
            <Box>
              <GenreFilter />
            </Box>
            <Box>
              <CustomInfiniteHits />
            </Box>
          </DefaultLayout>
        </InstantSearchNext>
      )}
    </>
  );
}
