import { jetstream } from '@nats-io/jetstream';
import { KvEntry, Kvm } from '@nats-io/kv';
import { connect } from '@nats-io/transport-node';
import { getConfig } from './config';

async function getNatsConnection() {
  const config = getConfig();
  const nc = await connect({ servers: config.nats.url });
  return nc;
}

async function getJetstreamConnection() {
  const nc = await getNatsConnection();
  const js = jetstream(nc);
  return js;
}

export async function getKvsValue(bucket: string, key: string) {
  const js = await getJetstreamConnection();
  const kvm = new Kvm(js);
  const kv = await kvm.open(bucket);
  return await kv.get(key);
}

export async function putKvsValue(
  bucket: string,
  key: string,
  value: Uint8Array,
) {
  const js = await getJetstreamConnection();
  const kvm = new Kvm(js);
  const kv = await kvm.open(bucket);
  return await kv.put(key, value);
}

export async function updateKvsValue(
  bucket: string,
  key: string,
  version: number,
  value: Uint8Array,
) {
  const js = await getJetstreamConnection();
  const kvm = new Kvm(js);
  const kv = await kvm.open(bucket);
  return await kv.update(key, value, version);
}
