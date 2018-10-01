import * as WebSocket from 'ws';
import * as EventEmitter from 'events';

import { noop, timestamp, Maybe, Types, NumStats } from '@dotstats/common';
import { parseMessage, getBestBlock, Message, BestBlock, SystemInterval } from './message';
import { locate, Location } from './location';
import { getId, refreshId } from './nodeId';
import { MeanList } from './MeanList';

const BLOCK_TIME_HISTORY = 10;
const MEMORY_RECORDS = 20;
const CPU_RECORDS = 20;
const TIMEOUT = (1000 * 60 * 1) as Types.Milliseconds; // 1 minute

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
  public readonly address: Maybe<Types.Address>;
  public readonly authority: boolean;

  public readonly events = new EventEmitter() as EventEmitter & NodeEvents;

  public location: Maybe<Location> = null;
  public lastMessage: Types.Timestamp;
  public config: string;
  public best = '' as Types.BlockHash;
  public height = 0 as Types.BlockNumber;
  public latency = 0 as Types.Milliseconds;
  public blockTime = 0 as Types.Milliseconds;
  public blockTimestamp = 0 as Types.Timestamp;
  public propagationTime: Maybe<Types.PropagationTime> = null;

  private peers = 0 as Types.PeerCount;
  private txcount = 0 as Types.TransactionCount;
  private memory = new MeanList<Types.MemoryUse>();
  private cpu = new MeanList<Types.CPUUse>();
  private chartstamps = new MeanList<Types.Timestamp>();

  private readonly ip: string;
  private readonly socket: WebSocket;
  private blockTimes = new NumStats<Types.Milliseconds>(BLOCK_TIME_HISTORY);
  private lastBlockAt: Maybe<Date> = null;
  private pingStart = 0 as Types.Timestamp;
  private throttle = false;

  constructor(
    ip: string,
    socket: WebSocket,
    name: Types.NodeName,
    chain: Types.ChainLabel,
    config: string,
    implentation: Types.NodeImplementation,
    version: Types.NodeVersion,
    address: Maybe<Types.Address>,
    authority: boolean,
    messages: Array<Message>,
  ) {
    this.ip = ip;
    this.id = getId(address, name);
    this.name = name;
    this.chain = chain;
    this.config = config;
    this.implementation = implentation;
    this.version = version;
    this.address = address;
    this.authority = authority;
    this.lastMessage = timestamp();
    this.socket = socket;

    socket.on('message', (data) => {
      const message = parseMessage(data);

      if (!message) {
        return;
      }

      this.onMessage(message);
    });

    socket.on('close', () => {
      console.log(`${this.name} has disconnected`);

      this.disconnect();
    });

    socket.on('error', (error) => {
      console.error(`${this.name} has errored`, error);

      this.disconnect();
    });

    socket.on('pong', () => {
      this.latency = (timestamp() - this.pingStart) as Types.Milliseconds;
      this.pingStart = 0 as Types.Timestamp;
    });

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

          const { name, chain, config, implementation, version, pubkey, authority } = message;

          resolve(new Node(ip, socket, name, chain, config, implementation, version, pubkey, authority === true, messages));
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
      this.updateLatency(now);
    }
  }

  public nodeDetails(): Types.NodeDetails {
    const authority = this.authority ? this.address : null;

    return [this.name, this.implementation, this.version, authority];
  }

  public nodeStats(): Types.NodeStats {
    return [this.peers, this.txcount];
  }

  public nodeHardware(): Types.NodeHardware {
    return [this.memory.get(), this.cpu.get(), this.chartstamps.get()];
  }

  public blockDetails(): Types.BlockDetails {
    return [this.height, this.best, this.blockTime, this.blockTimestamp, this.propagationTime];
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

  private disconnect() {
    this.socket.removeAllListeners();
    this.socket.close();
    this.socket.terminate();

    refreshId(this.address, this.name, this.id);

    this.events.emit('disconnect');
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
  }

  private onSystemInterval(message: SystemInterval) {
    const { peers, txcount, cpu, memory } = message;

    if (this.peers !== peers || this.txcount !== txcount) {
      this.peers = peers;
      this.txcount = txcount;

      this.events.emit('stats');
    }

    if (cpu != null && memory != null) {
      const cpuChange = this.cpu.push(cpu);
      const memChange = this.memory.push(memory);
      const stampChange = this.chartstamps.push(timestamp());

      if (cpuChange || memChange || stampChange) {
        this.events.emit('hardware');
      }
    }
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

    if (this.best !== best && this.height <= height) {
      const blockTime = this.getBlockTime(time);

      this.best = best;
      this.height = height;
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
}
