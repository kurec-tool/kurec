'use client';
import DefaultLayout from '@/component/Navigation/DefaultLayout';
import type { InstantMeiliSearchInstance } from '@meilisearch/instant-meilisearch';
import {
  Box,
  Button,
  Card,
  Chip,
  Dropdown,
  Input,
  List,
  ListItem,
  Menu,
  MenuButton,
  MenuItem,
  Stack,
  Typography,
} from '@mui/joy';
import useInfiniteScroll from 'react-infinite-scroll-hook';
import {
  Highlight,
  type RefinementListProps,
  useInfiniteHits,
  useInstantSearch,
  useRefinementList,
  useSearchBox,
} from 'react-instantsearch';
import { InstantSearchNext } from 'react-instantsearch-nextjs';
import 'instantsearch.css/themes/satellite.css';
import { AddCircle } from '@mui/icons-material';
import AddIcon from '@mui/icons-material/Add';
import ErrorIcon from '@mui/icons-material/Error';
import SearchIcon from '@mui/icons-material/Search';
import { useEffect, useState } from 'react';
import { useDebounce } from 'react-use';

const debounceTimeout = 200;

const CustomInfiniteHits = (props: {
  onAddIndividual: (programId: number) => void;
  onIgnoreIndividual: (programId: number) => void;
}) => {
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
        const startDate = `${startDateTime.getMonth() + 1}/${startDateTime.getDate()}`;
        const startTime = `${startDateTime.getHours().toString().padStart(2, '0')}:${startDateTime.getMinutes().toString().padStart(2, '0')}`;
        const endTime = `${endDateTime.getHours().toString().padStart(2, '0')}:${endDateTime.getMinutes().toString().padStart(2, '0')}`;

        return (
          <ListItem key={item.objectID}>
            <Card sx={{ width: '100%' }}>
              <Stack direction="row" spacing={2}>
                {item.ogp_url_hash && (
                  <Box>
                    <img
                      src={`http://localhost:3000/api/ogp/${item.ogp_url_hash}`}
                      alt="公式サイト画像"
                      style={{ maxWidth: '150px', display: 'inline' }}
                      onError={(e) => {
                        e.currentTarget.style.display = 'none';
                      }}
                    />
                  </Box>
                )}
                <Box>
                  <Typography level="title-lg">
                    <Highlight attribute="タイトル" hit={item} />
                  </Typography>
                  <Typography level="body-sm">
                    {item.放送局} {startDate} {item.放送曜日} {startTime}～
                    {endTime} {item.ジャンル.join(', ')}
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
              <Box>
                <Dropdown>
                  <MenuButton>録画/無視</MenuButton>
                  <Menu>
                    <MenuItem
                      onClick={() => props.onAddIndividual(item.program_id)}
                    >
                      🔴 個別録画
                    </MenuItem>
                    <MenuItem
                      onClick={() => props.onIgnoreIndividual(item.program_id)}
                    >
                      ❗ 個別無視
                    </MenuItem>
                  </Menu>
                </Dropdown>
                <Typography level="body-xs" display="inline-block">
                  {item.program_id}
                </Typography>
              </Box>
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
  handleAddAction,
}: {
  inputValue: string;
  setInputValue: (v: string) => void;
  handleAddAction: () => void;
}) {
  const { clear, query, refine } = useSearchBox();
  const status = useInstantSearch();
  const [, cancel] = useDebounce(
    () => {
      refine(inputValue);
    },
    debounceTimeout,
    [inputValue],
  );

  return (
    <Input
      placeholder="検索..."
      startDecorator={<SearchIcon />}
      endDecorator={
        inputValue && (
          <Chip onClick={handleAddAction}>
            <AddIcon />
          </Chip>
        )
      }
      autoFocus={true}
      onChange={(e) => {
        setInputValue(e.target.value);
      }}
      value={inputValue}
    />
  );
}

function CustomRefinementList(props: RefinementListProps) {
  const {
    items,
    hasExhaustiveItems,
    createURL,
    refine,
    sendEvent,
    searchForItems,
    isFromSearch,
    canRefine,
    canToggleShowMore,
    isShowingMore,
    toggleShowMore,
  } = useRefinementList(props);
  return (
    <Box>
      <Typography level="title-md">{props.attribute}</Typography>
      <Input
        placeholder="検索..."
        startDecorator={<SearchIcon />}
        onChange={(e) => searchForItems(e.target.value)}
      />
      <List>
        {items.map((item) => (
          <ListItem key={item.label}>
            <Chip
              component={item.isRefined ? 'span' : 'button'}
              onClick={() => {
                refine(item.value);
                sendEvent('click', 'filter', item.label);
              }}
              variant={item.isRefined ? 'solid' : 'outlined'}
            >
              {item.label} ({item.count})
            </Chip>
          </ListItem>
        ))}
      </List>
      {canToggleShowMore && (
        <Box>
          <Chip onClick={toggleShowMore}>
            {isShowingMore ? 'Show less' : 'Show more'}
          </Chip>
        </Box>
      )}
    </Box>
  );
}

export class EpgSearchQuery {
  private query: string;
  private refinementList: {
    [key: string]: string[];
  };
  constructor(query: string, refinementList: { [key: string]: string[] }) {
    this.query = query;
    this.refinementList = refinementList;
  }
  toQueryObject() {
    const filterString = Object.entries(this.refinementList)
      .map(([key, value]) => {
        return `${key} IN [ ${value.map((v) => `"${v}"`).join(', ')} ]`;
      })
      .map((q) => `( ${q} )`)
      .join(' AND ');
    return {
      query: this.query,
      filter: filterString,
    };
  }
}

type EpgSearchComponentProps = {
  searchClient: InstantMeiliSearchInstance;
  inputValue: string;
  setInputValue: (v: string) => void;
  handleAddAction: (query: EpgSearchQuery) => void; // Add Rule
  indexName: string;
  onAddIndividual: (programId: number) => void;
  onIgnoreIndividual: (programId: number) => void;
};

export default function EpgSearchComponent({
  searchClient,
  inputValue,
  setInputValue,
  handleAddAction,
  indexName,
  onAddIndividual,
  onIgnoreIndividual,
}: EpgSearchComponentProps) {
  const [query, setQuery] = useState<EpgSearchQuery>(
    new EpgSearchQuery('', {}),
  );
  const [updateUiState, setUpdateUiState] = useState<() => void>(() => {});
  const [, cancel] = useDebounce(
    () => {
      if (updateUiState) {
        updateUiState();
      }
    },
    debounceTimeout,
    [updateUiState],
  );

  return (
    <InstantSearchNext
      indexName={indexName}
      searchClient={searchClient}
      onStateChange={(params) => {
        setQuery(
          new EpgSearchQuery(
            params.uiState[indexName].query ?? '',
            params.uiState[indexName].refinementList ?? {},
          ),
        );
        setUpdateUiState(() => params.setUiState(params.uiState));
      }}
    >
      <DefaultLayout
        searchComponent={
          <CustomSearchBox
            inputValue={inputValue}
            setInputValue={setInputValue}
            handleAddAction={() => handleAddAction(query)}
          />
        }
      >
        <Box>
          <CustomRefinementList
            attribute="ジャンル"
            limit={255}
            searchable={true}
          />
        </Box>
        <Box>
          <CustomRefinementList
            attribute="放送局"
            limit={255}
            searchable={true}
          />
        </Box>
        <Box>
          <CustomInfiniteHits
            onAddIndividual={onAddIndividual}
            onIgnoreIndividual={onIgnoreIndividual}
          />
        </Box>
      </DefaultLayout>
    </InstantSearchNext>
  );
}
