import { timestamp, Maybe, Types, idGenerator, Opaque } from '@dotstats/common';

const CACHE_LIFETIME = (24 * 3600 * 1000) as Types.Milliseconds; // 24h
const CACHE_INTERVAL = (3600 * 1000) as Types.Milliseconds; // 1h

interface NodeIdCache {
  id: Types.NodeId;
  ts: Types.Timestamp;
}

type SaltedName = Opaque<string, 'SaltedName'>;

const nextId = idGenerator<Types.NodeId>();
const idCache = new Map<Types.Address | SaltedName, NodeIdCache>();

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

export function getId(pubkey: Maybe<Types.Address>, name: Types.NodeName): Types.NodeId {
  let cachekey: Types.Address | SaltedName;

  if (pubkey) {
    const cached = idCache.get(pubkey);

    if (cached) {
      return cached.id;
    }

    cachekey = pubkey;
  } else {
    cachekey = `name:${name}` as SaltedName;
  }

  const id = nextId();
  const ts = timestamp();

  idCache.set(cachekey, { id, ts });

  return id;
}

export function refreshId(pubkey: Maybe<Types.Address>, name: Types.NodeName, id: Types.NodeId) {
  const cachekey = pubkey ? pubkey : `name:${name}` as SaltedName;
  const ts = timestamp();

  idCache.set(cachekey, { id, ts });
}
