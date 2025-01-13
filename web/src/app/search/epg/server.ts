'use server';
import { getKvsValue, putKvsValue, updateKvsValue } from '@/lib/nats';
import type { MeilisearchRule } from '@/types/kurec_interface';

// TODO: prefixを設定から取る
const ruleBucketName = 'kurec-meilisearch-rules';
const ruleKeyName = 'rules';

export async function addRule(rule: MeilisearchRule) {
  // 同時書き込みで壊れないようupdateでrevision指定したatomicな書き込みを行う
  for (let idx = 0; idx < 5; idx++) {
    try {
      const rulesEntry = await getKvsValue(ruleBucketName, ruleKeyName);
      if (!rulesEntry) {
        await putKvsValue(
          ruleBucketName,
          ruleKeyName,
          new TextEncoder().encode(JSON.stringify([rule])),
        );
        return;
      }
      const rules = JSON.parse(new TextDecoder().decode(rulesEntry.value));
      rules.push(rule);

      await updateKvsValue(
        ruleBucketName,
        ruleKeyName,
        rulesEntry.revision,
        new TextEncoder().encode(JSON.stringify(rules)),
      );
    } catch (e) {
      console.error(`exception occured ${idx}:`, e);
      continue;
    }
    break;
  }
}
