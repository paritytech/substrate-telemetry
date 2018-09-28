import * as Enzyme from './enzyme';
const { shallow, mount } = Enzyme;

import * as sinon from 'sinon';
import { Server } from 'mock-socket';

import { Types, FeedMessage } from '../../common';

import { Node, Update, State } from '../src/state';
import { Connection } from '../src/Connection';
import { PersistentObject, PersistentSet } from '../src/persist';

describe('Connection.ts', () => {
  const fakeWebSocketURL = 'ws://localhost:8080';
  const mockServer = new Server(fakeWebSocketURL);

  mockServer.on('connection', (socket: any) => {
    console.log('Got connection!');

    socket.on('message', (message: any) => {
      console.log(message);
    });

    socket.send('[0,15,8,["BBQ Birch",6],8,["Krumme Lanke",57]]');
  });

  let state: State;
  let connection: Connection;
  let handleMessages: any;
  let update: any;

  const settings: State.Settings = {
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

  beforeAll(async () => {
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
      settings,
      pins: new Set()
    } as State;

    const pins: PersistentSet<Types.NodeName> = new PersistentSet<Types.NodeName>('key', ((p) => {}));

    update = jest.fn((changes) => {
      console.log(changes);

      state = Object.assign({}, state, changes);

      // stub update function
      return state as Readonly<State>;
    }) as Update;

    connection = await Connection.create(pins, update);

    const before = connection.handleMessages;

    connection.handleMessages = jest.fn(connection.handleMessages);

    // console.log('handle after', connection.handleMessages === handleMessages, connection.handleMessages === before);
  });

  afterEach(() => {
    // clear stubs and fakes after each test case
    // sinon.restore();
    // connection.handleMessages.restore();
  })

  test.only('handle Feed Version message state update', async () => {
    connection.handleMessages([
      {
        action: 0,
        payload: 15
      }, {
        action: 8,
        payload: ["BBQ Birch", 6],
      }, {
        action: 8,
        payload: ["Krumme Lanke", 57]
      }
    ] as any as FeedMessage.Message[])

    expect(update).toHaveBeenCalled();

    console.log(state);
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
