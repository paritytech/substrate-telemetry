import * as WebSocket from 'ws';

const wss = new WebSocket.Server({ port: 1024 });

wss.on('connection', async (socket: WebSocket) => {
  await Node.fromSocket(socket);
});

type Level = "INFO" | "WARN";

interface SystemConnected {
  msg: "system.connected",
  name: string,
  ts: string,
  chain: string,
  config: string,
  implementation: string,
  version: string,
}

type Message = SystemConnected;

class Node {
  private socket: WebSocket;
  private name: string;
  private config: string;
  private implementation: string;
  private version: string;

  constructor(socket: WebSocket, name: string, config: string, implentation: string, version: string) {
    this.socket = socket;
    this.name = name;
    this.config = config;
    this.implementation = implentation;
    this.version = version;

    console.log(`Started listening to a new node: ${name}`);

    socket.on('message', (message: WebSocket.Data) => {
      console.log('received: %s', message);
    });
  }

  static fromSocket(socket: WebSocket): Promise<Node> {
    return new Promise((resolve, reject) => {
      function handler(msg: WebSocket.Data) {
        let message: Message;

        try {
          message = JSON.parse(msg.toString());
        } catch (err) {
          socket.removeEventListener('message');

          return reject(err);
        }

        if (message.msg === "system.connected") {
          socket.removeEventListener('message');

          const { name, config, implementation, version } = message;

          resolve(new Node(socket, name, config, implementation, version));
        }
      }

      socket.on('message', handler);

      // TODO: timeout
    });
  }
}


// received: {"msg":"block.import","level":"INFO","ts":"2018-06-18T17:30:35.285406538+02:00","best":"3d4fdc7960078ddc9be87dddc48324a6d64afdf1f65fffe89529ce9965cd5f29","height":526}
// received: {"msg":"node.start","level":"INFO","ts":"2018-06-18T17:30:40.038731057+02:00","best":"3d4fdc7960078ddc9be87dddc48324a6d64afdf1f65fffe89529ce9965cd5f29","height":526}
// received: {"msg":"system.connected","level":"INFO","ts":"2018-06-18T17:30:40.038975471+02:00","chain":"dev","config":"","version":"0.2.0","implementation":"parity-polkadot","name":"Majestic Widget"}
