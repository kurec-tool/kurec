import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { load } from 'js-yaml';

interface AppConfig {
  prefix?: string;
  nats: {
    url: string;
  };
}

export function getConfig(): AppConfig {
  const configPath =
    process.env.KUREC_CONFIG_PATH ?? join(process.cwd(), 'kurec.yml'); // YAMLファイルのパス
  const fileContents = readFileSync(configPath, 'utf8'); // ファイルを読み込む
  const config = load(fileContents) as AppConfig; // YAMLをパース

  // TODO: デフォルト値を設定する方法をもっとかっこよく
  if (config.prefix === undefined) {
    config.prefix = 'kurec';
  }
  return config;
}
