import * as Enzyme from './enzyme';
const { shallow, mount } = Enzyme;

import * as sinon from 'sinon';
import { WebSocket, Server } from 'mock-socket';

import { Types, FeedMessage } from '../../common';

import { Node, Update, State } from '../src/state';
import { Connection } from '../src/Connection';
import { PersistentObject, PersistentSet } from '../src/persist';

describe('Connection.ts', () => {
  const fakeWebSocketURL = 'ws://localhost:8080';
  const mockServer = new Server(fakeWebSocketURL);

  let state: State;
  let connection: Promise<Connection>;

  const settings: Settings = {
    location: false,
    validator: false,
    implementation: false,
    peers: false,
    txs: false,
    cpu: false,
    mem: false,
    blocknumber: false,
    blockhash: false,
    blocktime: false,
    blockpropagation: false,
    blocklasttime: false
  }

  beforeAll(() => {
    state = {
      status: 'offline',
      best: 0 as Types.BlockNumber,
      blockTimestamp: 0 as Types.Timestamp,
      blockAverage: null,
      timeDiff: 0 as Types.Milliseconds,
      subscribed: null,
      chains: new Map(),
      nodes: new Map(),
      sortedNodes: [],
      settings: { settings },
      pins: []
    } as State;

    const pins: PersistentSet<Types.NodeName> = new PersistentSet<Types.NodeName>('key', (p) => {
      // stub pins function
      // const { nodes, sortedNodes } = this.state;
      //
      // for (const node of nodes.values()) {
      //   node.pinned = pins.has(node.name);
      // }

      // this.setState({ nodes, pins, sortedNodes: sortedNodes.sort(Node.compare) });
    });

    const update: Update = () => {
      // stub update function
      return state as Readonly<State>;
    };

    connection = sinon.stub(Connection, 'create')
                      .returns(new Connection(mockServer, update, pins))

    handleMessages = sinon.spy(connection, 'handleMessages');
  });

  afterEach(() => {
    // clear stubs and fakes after each test case
    sinon.restore();
  })

  test.only('handle Feed Version message state update', () => {
    // { action: 'FeedVersion', payload: null }
    expect(connection.handleMessages).toHaveBeenCalled();
  })


  test('handle Best Block message state update', () => {

  })

  test('handle Added Node message state update', () => {

  })


  test('handle Removed Node message state update', () => {

  })

  test('handle Located Node message state update', () => {

  })


  test('handle Imported Block message state update', () => {

  })

  test('handle Node Stats message state update', () => {

  })

  test('handle Time Sync message state update', () => {

  })

  test('handle Added Chain message state update', () => {

  })

  test('handle Removed Chain message state update', () => {

  })

  test('handle Subscribed To message state update', () => {

  })

  test('handle Unsubscribed From message state update', () => {

  })

  test('handle message update', () => {

  })
});
