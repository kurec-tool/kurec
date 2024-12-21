'use client';
import EpgSearchComponent from '@/component/Search/Meilisearch/EpgSearch';
import {
  type InstantMeiliSearchInstance,
  instantMeiliSearch,
} from '@meilisearch/instant-meilisearch';
import { useCallback, useEffect, useState } from 'react';

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

  const handleAddAction = useCallback(() => {
    console.log(`Add action inputValue: ${inputValue}`);
  }, [inputValue]);

  return (
    <>
      {searchClient && (
        <EpgSearchComponent
          searchClient={searchClient}
          inputValue={inputValue}
          setInputValue={setInputValue}
          handleAddAction={handleAddAction}
        />
      )}
    </>
  );
}
