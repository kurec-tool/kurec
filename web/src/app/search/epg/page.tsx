'use client';
import EpgSearchComponent, {
  type EpgSearchQuery,
} from '@/component/Search/Meilisearch/EpgSearch';
import {
  type InstantMeiliSearchInstance,
  instantMeiliSearch,
} from '@meilisearch/instant-meilisearch';
import { useCallback, useEffect, useState } from 'react';
import { addRule } from './server';

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

  const handleAddAction = useCallback(
    async (query: EpgSearchQuery) => {
      console.log(
        `Add action inputValue: ${inputValue} query: ${JSON.stringify(query.toQueryObject())}`,
      );
      await addRule(query.toQueryObject());
    },
    [inputValue],
  );

  return (
    <>
      {searchClient && (
        <EpgSearchComponent
          searchClient={searchClient}
          indexName="kurec-epg"
          inputValue={inputValue}
          setInputValue={setInputValue}
          handleAddAction={handleAddAction}
        />
      )}
    </>
  );
}
