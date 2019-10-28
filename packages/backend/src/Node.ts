import * as WebSocket from 'ws';
import * as EventEmitter from 'events';

import { noop, timestamp, idGenerator, Maybe, Types, NumStats } from '@dotstats/common';
import { BlockHash, BlockNumber, ConsensusView } from "@dotstats/common/build/types";
import {
  parseMessage,
  getBestBlock,
  Message,
  BestBlock,
  SystemInterval,
  SystemNetworkState,
  AfgFinalized,
  AfgReceivedPrecommit,
  AfgReceivedPrevote,
  AfgAuthoritySet,
} from './message';
import { locate, Location } from './location';
import MeanList from './MeanList';
import Block from './Block';

const BLOCK_TIME_HISTORY = 10;
const MEMORY_RECORDS = 20;
const CPU_RECORDS = 20;
const TIMEOUT = (1000 * 60 * 1) as Types.Milliseconds; // 1 minute
const NO_BLOCK_TIMEOUT = (1000 * 60 * 1) as Types.Milliseconds; // 1 minute

const nextId = idGenerator<Types.NodeId>();

export interface NodeEvents {
  on(event: 'location', fn: (location: Location) => void): void;
  emit(event: 'location', location: Location): void;
}

export default class Node {
  public readonly id: Types.NodeId;
  public readonly name: Types.NodeName;
  public readonly chain: Types.ChainLabel;
  public readonly implementation: Types.NodeImplementation;
  public readonly version: Types.NodeVersion;
  public readonly networkId: Maybe<Types.NetworkId>;
  public readonly authority: boolean;

  public readonly events = new EventEmitter() as EventEmitter & NodeEvents;

  public address: Maybe<Types.Address> = null;
  public networkState: Maybe<Types.NetworkState> = null;
  public location: Maybe<Location> = null;
  public lastMessage: Types.Timestamp;
  public config: string;
  public best = Block.ZERO;
  public finalized = Block.ZERO;
  public latency = 0 as Types.Milliseconds;
  public blockTime = 0 as Types.Milliseconds;
  public blockTimestamp = 0 as Types.Timestamp;
  public propagationTime: Maybe<Types.PropagationTime> = null;
  public isStale = false;

  private peers = 0 as Types.PeerCount;
  private txcount = 0 as Types.TransactionCount;
  private memory = new MeanList<Types.MemoryUse>();
  private cpu = new MeanList<Types.CPUUse>();
  private upload = new MeanList<Types.BytesPerSecond>();
  private download = new MeanList<Types.BytesPerSecond>();
  private chartstamps = new MeanList<Types.Timestamp>();

  private readonly ip: string;
  private readonly socket: WebSocket;
  private blockTimes = new NumStats<Types.Milliseconds>(BLOCK_TIME_HISTORY);
  private lastBlockAt: Maybe<Date> = null;
  private pingStart = 0 as Types.Timestamp;
  private throttle = false;

  private authorities: Types.Authorities = [] as Types.Authorities;
  private authoritySetId: Types.AuthoritySetId = 0 as Types.AuthoritySetId;

  constructor(
    ip: string,
    socket: WebSocket,
    name: Types.NodeName,
    chain: Types.ChainLabel,
    config: string,
    implentation: Types.NodeImplementation,
    version: Types.NodeVersion,
    networkId: Maybe<Types.NetworkId>,
    authority: boolean,
    messages: Array<Message>,
  ) {
    this.ip = ip;
    this.id = nextId();
    this.name = name;
    this.chain = chain;
    this.config = config;
    this.implementation = implentation;
    this.version = version;
    this.authority = authority;
    this.networkId = networkId;
    this.lastMessage = timestamp();
    this.socket = socket;

    socket.on('message', this.onMessageData);
    socket.on('close', this.disconnect);
    socket.on('error', this.disconnect);
    socket.on('pong', this.onPong);

    process.nextTick(() => {
      // Handle cached messages
      for (const message of messages) {
        this.onMessage(message);
      }
    });

    locate(ip).then((location) => {
      if (!location) {
        return;
      }

      this.location = location;

      this.events.emit('location', location);
    });
  }

  public static fromSocket(socket: WebSocket, ip: string): Promise<Node> {
    return new Promise((resolve, reject) => {
      function cleanup() {
        clearTimeout(timeout);
        socket.removeAllListeners('message');
      }

      const messages: Array<Message> = [];

      function handler(data: WebSocket.Data) {
        const message = parseMessage(data);

        if (!message || !message.msg) {
          return;
        }

        if (message.msg === "system.connected") {
          cleanup();

          const { name, chain, config, implementation, version, authority, network_id: networkId } = message;

          resolve(new Node(ip, socket, name, chain, config, implementation, version, networkId, authority === true, messages));
        } else {
          if (messages.length === 10) {
            messages.shift();
          }

          messages.push(message);
        }
      }

      socket.on('message', handler);

      const timeout = setTimeout(() => {
        cleanup();

        socket.close();
        socket.terminate();

        return reject(new Error('Timeout on waiting for system.connected message'));
      }, 5000);
    });
  }

  public timeoutCheck(now: Types.Timestamp) {
    if (this.lastMessage + TIMEOUT < now) {
      this.disconnect();
    } else {
      if (!this.isStale && this.blockTimestamp + NO_BLOCK_TIMEOUT < now) {
        this.events.emit('stale');
      }

      this.updateLatency(now);
    }
  }

  public nodeDetails(): Types.NodeDetails {
    const authority = this.authority ? this.address : null;
    const addr = this.address ? this.address : '' as Types.Address;

    return [this.name, this.implementation, this.version, authority, this.networkId, addr];
  }

  public nodeStats(): Types.NodeStats {
    return [this.peers, this.txcount];
  }

  public nodeHardware(): Types.NodeHardware {
    return [this.memory.get(), this.cpu.get(), this.upload.get(), this.download.get(), this.chartstamps.get()];
  }

  public blockDetails(): Types.BlockDetails {
    return [this.best.number, this.best.hash, this.blockTime, this.blockTimestamp, this.propagationTime];
  }

  public nodeLocation(): Maybe<Types.NodeLocation> {
    const { location } = this;

    return location ? [location.lat, location.lon, location.city] : null;
  }

  public get average(): Types.Milliseconds {
    return this.blockTimes.average();
  }

  public get localBlockAt(): Types.Milliseconds {
    if (!this.lastBlockAt) {
      return 0 as Types.Milliseconds;
    }

    return +(this.lastBlockAt || 0) as Types.Milliseconds;
  }

  private disconnect = () => {
    console.log(`${this.name} has disconnected`);

    this.socket.removeListener('message', this.onMessageData);
    this.socket.removeListener('close', this.disconnect);
    this.socket.removeListener('error', this.disconnect);
    this.socket.removeListener('pong', this.onPong);
    this.socket.close();
    this.socket.terminate();

    this.events.emit('disconnect');
  }

  private onMessageData = (data: WebSocket.Data) => {
    const message = parseMessage(data);

    if (!message) {
      return;
    }

    this.onMessage(message);
  }

  private onMessage(message: Message) {
    this.lastMessage = timestamp();

    const update = getBestBlock(message);

    if (update) {
      this.updateBestBlock(update);
    }

    if (message.msg === 'system.interval') {
      this.onSystemInterval(message);
    }

    if (message.msg === 'system.network_state') {
      this.onSystemNetworkState(message);
    }

    if (message.msg === 'afg.finalized') {
      this.onAfgFinalized(message);
    }
    if (message.msg === 'afg.received_precommit') {
      this.onAfgReceivedPrecommit(message);
    }
    if (message.msg === 'afg.received_prevote') {
      this.onAfgReceivedPrevote(message);
    }
    if (message.msg === 'afg.authority_set') {
      this.onAfgAuthoritySet(message);
    }
  }

  private onSystemInterval(message: SystemInterval) {
    const {
      network_state,
      peers,
      txcount,
      cpu,
      memory,
      bandwidth_download: download,
      bandwidth_upload: upload,
      finalized_height: finalized,
      finalized_hash: finalizedHash
    } = message;

    if (this.networkState !== network_state && network_state) {
      this.networkState = network_state;
    };

    if (this.peers !== peers || this.txcount !== txcount) {
      this.peers = peers;
      this.txcount = txcount;

      this.events.emit('stats');
    }

    if (finalized != null && finalizedHash != null && finalized > this.finalized.number) {
      this.finalized = new Block(finalized, finalizedHash);

      this.events.emit('finalized');
    }

    if (cpu != null && memory != null) {
      const cpuChange = this.cpu.push(cpu);
      const memChange = this.memory.push(memory);

      const uploadChange = this.upload.push(upload);
      const downloadChange = this.download.push(download);

      const stampChange = this.chartstamps.push(timestamp());

      if (cpuChange || memChange || uploadChange || downloadChange || stampChange) {
        this.events.emit('hardware');
      }
    }
  }

  private onSystemNetworkState(message: SystemNetworkState) {
    this.networkState = message.state;
  }

  public isAuthority(): boolean {
    return this.authority;
  }

  private onAfgReceivedPrecommit(message: AfgReceivedPrecommit) {
    const {
      target_number: targetNumber,
      target_hash: targetHash,
    } = message;
    const voter = this.extractVoter(message.voter);
    const number = parseInt(String(targetNumber), 10) as Types.BlockNumber;
    this.events.emit('afg-received-precommit', number, targetHash, voter);
  }

  private onAfgReceivedPrevote(message: AfgReceivedPrevote) {
    const {
      target_number: targetNumber,
      target_hash: targetHash,
    } = message;
    const voter = this.extractVoter(message.voter);
    const number = parseInt(String(targetNumber), 10) as Types.BlockNumber;
    this.events.emit('afg-received-prevote', number, targetHash, voter);
  }

  private onAfgAuthoritySet(message: AfgAuthoritySet) {
    const {
      authority_id: authorityId,
      authority_set_id: authoritySetId,
      hash,
      number,
    } = message;

    // we manually parse the authorities message, because the array was formatted as a
    // string by substrate before sending it.
    const authorities = JSON.parse(String(message.authorities)) as Types.Authorities;

    this.address = authorityId;

    if (JSON.stringify(this.authorities) !== String(message.authorities) ||
        this.authoritySetId !== authoritySetId) {
      const no = parseInt(String(number), 10) as Types.BlockNumber;
      this.events.emit('authority-set-changed', authorities, authoritySetId, no, hash);
    }
  }

  private onAfgFinalized(message: AfgFinalized) {
    const {
      finalized_number: finalizedNumber,
      finalized_hash: finalizedHash,
    } = message;
    const number = parseInt(String(finalizedNumber), 10) as Types.BlockNumber;
    this.events.emit('afg-finalized', number, finalizedHash);
  }

  private extractVoter(message_voter: String): Types.Address {
    return String(message_voter.replace(/"/g, '')) as Types.Address;
  }

  private updateLatency(now: Types.Timestamp) {
    // if (this.pingStart) {
    //   console.error(`${this.name} timed out on ping message.`);
    //   this.disconnect();
    //   return;
    // }

    this.pingStart = now;

    try {
      this.socket.ping(noop);
    } catch (err) {
      console.error('Failed to send ping to Node', err);

      this.disconnect();
    }
  }

  private updateBestBlock(update: BestBlock) {
    const { height, ts: time, best } = update;

    if (this.best.hash !== best && this.best.number <= height) {
      const blockTime = this.getBlockTime(time);

      this.best = new Block(height, best);
      this.blockTimestamp = timestamp();
      this.lastBlockAt = time;
      this.blockTimes.push(blockTime);
      this.blockTime = blockTime;

      if (blockTime > 100) {
        this.events.emit('block');
      } else if (!this.throttle) {
        this.throttle = true;

        setTimeout(() => {
          this.events.emit('block');
          this.throttle = false;
        }, 1000);
      }
    }
  }

  private getBlockTime(time: Date): Types.Milliseconds {
    if (!this.lastBlockAt) {
      return 0 as Types.Milliseconds;
    }

    return (+time - +this.lastBlockAt) as Types.Milliseconds;
  }

  private onPong = () => {
    this.latency = (timestamp() - this.pingStart) as Types.Milliseconds;
    this.pingStart = 0 as Types.Timestamp;
  }
}
