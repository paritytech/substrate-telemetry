import * as React from 'react';
import { Connection } from '../Connection';
import { Icon } from './Icon';
import { Types, Maybe } from '@dotstats/common';
// import stable from 'stable';

import { Button, Header, Icon as SUIcon, Image, Menu, Segment, Sidebar } from 'semantic-ui-react';
import 'semantic-ui-css/components/header.min.css';
import 'semantic-ui-css/components/image.min.css';
import 'semantic-ui-css/components/menu.min.css';
import 'semantic-ui-css/components/segment.min.css';
import 'semantic-ui-css/components/sidebar.min.css';

import githubIcon from '../icons/mark-github.svg';
import './Chains.css';

interface ChainData {
  label: any;
  nodeCount: any;
}

// const SidebarExampleVisible = (handleHideClick: any, handleShowClick: any, handleSidebarHide: any, visible: boolean) => (
//   <div>
//     <Button.Group>
//       <Button disabled={visible} onClick={handleShowClick}>
//         Show sidebar
//       </Button>
//       <Button disabled={!visible} onClick={handleHideClick}>
//         Hide sidebar
//       </Button>
//     </Button.Group>

//     <Sidebar.Pushable as={Segment}>
//       <Sidebar
//         as={Menu}
//         animation='overlay'
//         icon='labeled'
//         inverted={true}
//         onHide={handleSidebarHide}
//         vertical={true}
//         visible={visible}
//         width='thin'
//       >
//         <Menu.Item as='a'>
//           <SUIcon name='home' />
//           Home
//         </Menu.Item>
//         <Menu.Item as='a'>
//           <SUIcon name='gamepad' />
//           Games
//         </Menu.Item>
//         <Menu.Item as='a'>
//           <SUIcon name='camera' />
//           Channels
//         </Menu.Item>
//       </Sidebar>

//       <Sidebar.Pusher>
//         <Segment basic={true}>
//           <Header as='h3'>Application Content</Header>
//           <Image src='/images/wireframe/paragraph.png' />
//         </Segment>
//       </Sidebar.Pusher>
//     </Sidebar.Pushable>
//   </div>
// );

export namespace Chains {
  export interface Props {
    chains: any,
    subscribed: Maybe<Types.ChainLabel>,
    connection: Promise<Connection>
  }
}

const abc = [
  {
    nodeCount: 1,
    label: 'AAAAAAAAA'
  }
];
// const abc = [
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   },
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   },
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   },
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   },
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   },
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   },
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   },
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   },
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   },
//   {
//     nodeCount: 1,
//     label: 'AAAAAAAAA'
//   }
// ];

export class Chains extends React.Component<Chains.Props, {}> {
  public state = {
    visible: false
  };

  public handleHideClick() {
    this.setState({ visible: false });
  }

  public handleShowClick() {
    console.log('I MADE IT INTO HANDLE SHOW CLICK');
    this.setState({ visible: true });
  }

  public handleSidebarHide() {
    this.setState({ visible: false });
  }

  public render() {
    return (
      <div className="Chains">
        {
          abc.map((chain) => this.renderChain(chain))
        }
        <a className="Chains-fork-me" href="https://github.com/paritytech/substrate-telemetry" target="_blank">
          <Icon src={githubIcon} alt="Fork Me!" />
        </a>
      </div>
    );
  }

  private renderChain(chain: ChainData): React.ReactNode {
    const { label, nodeCount } = chain;
    const { visible } = this.state;

    const className = label === this.props.subscribed
      ? 'Chains-chain Chains-chain-selected'
      : 'Chains-chain';

    return (
      <div>
        {/* <SidebarExampleVisible
          handleShowClick={this.handleShowClick}
          handleHideClick={this.handleHideClick}
          handleSidebarHide={this.handleSidebarHide}
          visible={this.state.visible}
        /> */}
        <div>
          <Button.Group>
            <Button disabled={visible} onClick={this.handleShowClick}>
              Show sidebar
            </Button>
            <Button disabled={!visible} onClick={this.handleHideClick}>
              Hide sidebar
            </Button>
          </Button.Group>

          <Sidebar.Pushable as={Segment}>
            <Sidebar
              as={Menu}
              animation='overlay'
              icon='labeled'
              inverted={true}
              onHide={this.handleSidebarHide}
              vertical={true}
              visible={visible}
              width='thin'
            >
              <Menu.Item as='a'>
                <SUIcon name='home' />
                Home
              </Menu.Item>
              <Menu.Item as='a'>
                <SUIcon name='gamepad' />
                Games
              </Menu.Item>
              <Menu.Item as='a'>
                <SUIcon name='camera' />
                Channels
              </Menu.Item>
            </Sidebar>

            <Sidebar.Pusher>
              <Segment basic={true}>
                <Header as='h3'>Application Content</Header>
                <Image src='/images/wireframe/paragraph.png' />
              </Segment>
            </Sidebar.Pusher>
          </Sidebar.Pushable>
        </div>
        <a key={label} className={className} onClick={this.subscribe.bind(this, label)}>
          {label} <span className="Chains-node-count" title="Node Count">{nodeCount}</span>
        </a>
      </div>
    )
  }

  // private get chains(): any {
  //   return stable
  //     .inplace(
  //       Array.from((abc as any).entries()),
  //       (a, b) => b[1] - a[1]
  //     )
  //     .map(([label, nodeCount]) => ({ label, nodeCount }));
  // }

  private async subscribe(chain: Types.ChainLabel) {
    const connection = await this.props.connection;

    connection.subscribe(chain);
  }
}
