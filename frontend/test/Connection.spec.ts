import * as Enzyme from './enzyme';
const { shallow, mount } = Enzyme;

import { Server } from 'mock-socket';

import { Types, FeedMessage, timestamp, VERSION, SortedCollection } from '../../common';

import { Node, Update, State } from '../src/state';
import { Connection } from '../src/Connection';
import { PersistentObject, PersistentSet } from '../src/persist';

const { Actions } = FeedMessage;

describe('Connection.ts', () => {
  const fakeWebSocketURL = 'ws://localhost:8080';
  const mockServer = new Server(fakeWebSocketURL);

  mockServer.on('connection', (socket: any) => {
    console.log('Got connection!');

    // socket.on('message', (message: any) => {
    //   console.log('message received by web socket -> ', message);
    // });
    //
    // socket.send('[0,15,8,["BBQ Birch",6],8,["Pistachios",57]]');
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
      nodes: new SortedCollection(Node.compare),
      settings,
      pins: new Set()
    } as State;

    const pins: PersistentSet<Types.NodeName> = new PersistentSet<Types.NodeName>('key', ((p) => {}));

    update = jest.fn((changes) => {
      // console.log('message passed to the update function -> ', changes);

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
    // connection.handleMessages.restore();
  })

  test('handle Feed Version message state update', async () => {
    connection.handleMessages([
      {
        action: Actions.FeedVersion,
        payload: VERSION
      }
    ] as any as FeedMessage.Message[])

    expect(update).toHaveBeenCalled();
    expect(state.status).toBe('online');
  })

  test('handle Best Block message state update', () => {
    const time = timestamp();

    connection.handleMessages([
      {
        action: Actions.BestBlock,
        payload: [1, time, 0],
      }, {
        action: Actions.BestBlock,
        payload: [1, time, 123456789]
      }
    ] as any as FeedMessage.Message[]);

    expect(update).toHaveBeenCalled();
    expect(state.blockTimestamp).toBe(time);
    expect(state.blockAverage).toBe(123456789);
  })

  describe('Add and Remove a Node', () => {
    const time = timestamp();

    test('handle Added Node message state update', () => {
      /*
        Added Node Message (NodeId, NodeDetails, NodeStats, BlockDetails, Maybe<NodeLocation>)
      */
      connection.handleMessages([
        {
          action: Actions.AddedNode,
          payload: [1, ['Sample Node', 'Sampling', '1.2.3', '0x123456789012345'], [12, 84], [[38], [78], [5]], [1, 'aoaiuhsf9o2ih389r', 777, time, 829]],
        }
      ] as any as FeedMessage.Message[]);

      expect(update).toHaveBeenCalled();
      expect(state.status).toBe('online');
      expect(state.nodes).toBeDefined();

      const firstNode = state.nodes.sorted()[0];

      expect(firstNode.id).toBe(1);
      expect(firstNode.name).toBe('Sample Node');
      expect(firstNode.implementation).toBe('Sampling');
      expect(firstNode.validator).toBe('0x123456789012345');
      expect(firstNode.peers).toBe(12);
      expect(firstNode.txs).toBe(84);
      expect(firstNode.mem).toEqual([38]);
      expect(firstNode.cpu).toEqual([78]);
      expect(firstNode.height).toBe(1);
      expect(firstNode.hash).toBe('aoaiuhsf9o2ih389r');
      expect(firstNode.blockTime).toBe(777);
      expect(firstNode.blockTimestamp).toBe(time);
      expect(firstNode.propagationTime).toBe(829);
    })

    test('handle Located Node message state update', () => {

      /*
        Located Node
        [NodeId, Latitude, Longitude, City]
      */

      connection.handleMessages([
        {
          action: Actions.LocatedNode,
          payload: [1, 30.828, 101.4111, 'Kuala Lumpur']
        }
      ] as any as FeedMessage.Message[]);

      expect(update).toHaveBeenCalled();

      const firstNode = state.nodes.sorted()[0];

      expect(firstNode.lat).toEqual(30.828)
      expect(firstNode.lon).toEqual(101.4111)
      expect(firstNode.city).toEqual('Kuala Lumpur')
    });

    test('handle Time Sync message state update', () => {
      connection.handleMessages([
        {
          action: Actions.TimeSync,
          payload: time + 12345432
        }
      ] as any as FeedMessage.Message[]);

      const firstNode = state.nodes.sorted()[0];

      expect(firstNode.blockTimestamp).toBe(time);
    })

    test('handle Imported Block message state update', () => {
      /*
        ImportedBlockMessage [NodeId, BlockDetails]
        BlockDetails = [BlockNumber, BlockHash, Milliseconds, Timestamp, Maybe<PropagationTime>]
      */
      connection.handleMessages([
        {
          action: Actions.ImportedBlock,
          payload: ["BBQ Birch", [1, 'ABCDEFGH12345678', 123, time, 48292010]],
        }
      ] as any as FeedMessage.Message[]);

      const firstSortedNode = state.nodes.sorted()[0];

      expect(firstSortedNode.pinned).toBeFalsy();
      expect(firstSortedNode).toMatchObject({
          pinned: false,
          id: 1,
          name: 'Sample Node',
          implementation: 'Sampling',
          version: '1.2.3',
          validator: '0x123456789012345',
          peers: 12,
          txs: 84,
          mem: [38],
          cpu: [78],
          height: 1,
          hash: 'aoaiuhsf9o2ih389r',
          blockTime: 777,
          blockTimestamp: time,
          propagationTime: 829,
          lat: 30.828,
          lon: 101.4111,
          city: 'Kuala Lumpur'
        });
      });

    test('handle Removed Node message state update', () => {
      /*
        payload: 1
      */
      connection.handleMessages([
        {
          action: Actions.RemovedNode,
          payload: 1
        }
      ] as any as FeedMessage.Message[]);

      expect(update).toHaveBeenCalled();
      expect(state.nodes.sorted()).toEqual([]);
    })
  })

  describe('Add and Remove a Chain', () => {
    test('handle Added Chain message state update', () => {
      connection.handleMessages([
        {
          action: Actions.AddedChain,
          payload: ["BBQ Birch", 6],
        }, {
          action: Actions.AddedChain,
          payload: ["Krumme Lanke", 57]
        }
      ] as any as FeedMessage.Message[]);

      expect(update).toHaveBeenCalled();

      const chains = [];

      for (const chain of state.chains) {
        chains.push(chain);
      }

      const firstChain = chains[0];
      const secondChain = chains[1];

      expect(chains).toHaveLength(2);
      expect(firstChain).toEqual([ 'BBQ Birch', 6 ]);
      expect(secondChain).toEqual([ 'Krumme Lanke', 57 ]);
    });

    test('handle Node Stats message state update', () => {

    })


    describe('Subscribe and Unsubscribe to Message', () => {
      test('handle Subscribed To message state update', () => {
        connection.handleMessages([
          {
            action: Actions.SubscribedTo,
            payload: 'BBQ Birch'
          }
        ] as any as FeedMessage.Message[]);

        expect(update).toHaveBeenCalled();
        expect(state.subscribed).toBe('BBQ Birch');

        connection.handleMessages([
          {
            action: Actions.SubscribedTo,
            payload: 'Krumme Lanke'
          }
        ] as any as FeedMessage.Message[]);

        expect(update).toHaveBeenCalled();
        expect(state.subscribed).toBe('Krumme Lanke');
      });

      test('handle Unsubscribed From message state update', () => {
        connection.handleMessages([
          {
            action: Actions.UnsubscribedFrom,
            payload: 'Krumme Lanke'
          }
        ] as any as FeedMessage.Message[]);

        expect(update).toHaveBeenCalled();
        expect(state.subscribed).toBeFalsy();
      })
    })

    test('handle Removed Chain message state update', () => {
      connection.handleMessages([
        {
          action: Actions.RemovedChain,
          payload: 'BBQ Birch'
        }, {
          action: Actions.RemovedChain,
          payload: 'Krumme Lanke'
        }
      ] as any as FeedMessage.Message[]);

      expect(update).toHaveBeenCalled();
      expect(state.chains.keys()).toMatchObject({});
    })
  })
});
