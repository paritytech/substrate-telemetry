import { timestamp, Maybe, Types, idGenerator } from '@dotstats/common';

const CACHE_LIFETIME = (24 * 3600 * 1000) as Types.Milliseconds; // 24h
const CACHE_INTERVAL = (3600 * 1000) as Types.Milliseconds; // 1h

interface NodeIdCache {
  id: Types.NodeId;
  ts: Types.Timestamp;
}

const nextId = idGenerator<Types.NodeId>();
const idCache = new Map<Types.NodePubKey, NodeIdCache>();

function clearCache() {
  const now = timestamp();

  for (const [pubkey, { ts }] of idCache.entries()) {
    if ((now - ts) > CACHE_LIFETIME) {
      idCache.delete(pubkey);
    }
  }

  setTimeout(clearCache, CACHE_INTERVAL);
}

clearCache();

export function getId(pubkey: Maybe<Types.NodePubKey>): Types.NodeId {
  if (!pubkey) {
    return nextId();
  }

  const cached = idCache.get(pubkey);

  if (cached) {
    return cached.id;
  }

  const id = nextId();
  const ts = timestamp();

  idCache.set(pubkey, { id, ts });

  return id;
}

export function refreshId(pubkey: Maybe<Types.NodePubKey>, id: Types.NodeId) {
  if (!pubkey) {
    return;
  }

  const ts = timestamp();

  idCache.set(pubkey, { id, ts });
}
