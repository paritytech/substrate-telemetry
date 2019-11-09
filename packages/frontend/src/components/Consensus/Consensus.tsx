import * as React from 'react';
import { Types, Maybe } from '@dotstats/common';
import { Connection } from '../../Connection';
import Measure, {BoundingRect, ContentRect} from 'react-measure';

import { ConsensusBlock } from './';
import { State as AppState } from '../../state';

import './Consensus.css';

// Maximum number of authorities the visualization is
// allowed of processing.
export const VIS_AUTHORITIES_LIMIT = 10;

export namespace Consensus {
  export interface Props {
    appState: Readonly<AppState>;
    connection: Promise<Connection>;
  }

  export interface State {
    dimensions: BoundingRect;

    largeBlockWithLegend: BoundingRect,
    largeBlock: BoundingRect,
    countBlocksInLargeRow: number,
    largeRowsAddFlexClass: boolean,

    smallBlock: BoundingRect,
    smallBlocksRows: number,
    countBlocksInSmallRow: number,
    smallRowsAddFlexClass: boolean,
    lastConsensusInfo: string,
  }
}

export class Consensus extends React.Component<Consensus.Props, {}> {
  public state = {
    // entire area available for rendering the visualization
    dimensions: { width: -1, height: -1 } as BoundingRect,

    largeBlockWithLegend: { width: -1, height: -1 } as BoundingRect,
    largeBlock: { width: -1, height: -1 } as BoundingRect,
    countBlocksInLargeRow: 2,
    largeRowsAddFlexClass: false,

    smallBlock: { width: -1, height: -1 } as BoundingRect,
    smallBlocksRows: 1,
    countBlocksInSmallRow: 1,
    smallRowsAddFlexClass: false,
    lastConsensusInfo: "",
  };

  public shouldComponentUpdate(nextProps: Consensus.Props, nextState: Consensus.State): boolean {
    if (this.props.appState.authorities.length === 0 && nextProps.appState.authorities.length === 0) {
      return false;
    }

    this.calculateBoxCount(false);

    // size detected, but flex class has not yet been added
    const largeBlocksSizeDetected = this.largeBlocksSizeDetected(nextState) === true &&
      this.state.largeRowsAddFlexClass === false;
    if (largeBlocksSizeDetected) {
      return true;
    }

    const smallBlocksSizeDetected = this.smallBlocksSizeDetected(nextState) === true &&
      this.state.smallRowsAddFlexClass === false;
    if (smallBlocksSizeDetected) {
      return true;
    }

    const windowSizeChanged = JSON.stringify(this.state.dimensions) !==
      JSON.stringify(nextState.dimensions);
    if (windowSizeChanged) {
      return true;
    }

    const newConsensusInfoAvailable = this.state.lastConsensusInfo !==
      JSON.stringify(nextProps.appState.consensusInfo);
    if (newConsensusInfoAvailable) {
      return true;
    }

    const authoritySetIdDidChange = this.props.appState.authoritySetId !==
      nextProps.appState.authoritySetId;
    if (authoritySetIdDidChange) {
      return true;
    }

    const authoritiesDidChange = JSON.stringify(this.props.appState.authorities) !==
      JSON.stringify(nextProps.appState.authorities);
    if (authoritiesDidChange) {
      return true;
    }

    return false;
  }

  public componentDidMount() {
    if (this.props.appState.subscribed != null) {
      const chain = this.props.appState.subscribed;
      this.subscribeConsensus(chain);
    }
  }

  public componentWillUnmount() {
    if (this.props.appState.subscribed != null) {
      const chain = this.props.appState.subscribed;
      this.unsubscribeConsensus(chain);
    }
  }

  public largeBlocksSizeDetected(state: Consensus.State): boolean {
    // we can only state that we detected the two block sizes (with
    // legend and without) if at least two blocks have been added:
    // the first displayed block will always have a legend with the
    // node names attached, the second not.
    if (this.props.appState.consensusInfo.length < 2) {
      return false;
    }

    // if there is more than one block then the size of the first block (with legend)
    // will be different from the succeeding blocks (without legend)
    return state.largeBlockWithLegend.width > -1 && state.largeBlockWithLegend.height > -1 &&
      state.largeBlock.width > -1 && state.largeBlock.height > -1;
  }

  public smallBlocksSizeDetected(state: Consensus.State): boolean {
    return state.smallBlock.width > -1 && state.largeBlockWithLegend.height > -1;
  }

  public calculateBoxCount(wasResized: boolean) {
    // if the css class for flexing has already been added we don't calculate
    // any box measurements then, because the box sizes would be skewed then.
    if ((wasResized || this.state.largeRowsAddFlexClass === false) && this.largeBlocksSizeDetected(this.state)) {
      // we need to add +2 because of the last block which doesn't contain a border.
      let countBlocks = (this.state.dimensions.width - this.state.largeBlockWithLegend.width + 2) /
        (this.state.largeBlock.width + 2);

      // +1 because the firstRect was subtracted above and needs to be counted back in.
      // default count is 2 because we need two blocks to measure properly (one with legend
      // and one without. these measures are necessary to calculate the number of blocks
      // which fit.
      countBlocks = Math.floor(countBlocks + 1) < 1 ? 2 : Math.floor(countBlocks + 1);

      this.setState({largeRowsAddFlexClass: true, countBlocksInLargeRow: countBlocks });
    }

    if ((wasResized || this.state.smallRowsAddFlexClass === false) && this.smallBlocksSizeDetected(this.state)) {
      const howManyRows = 2;

      const heightLeft = this.state.dimensions.height - (this.state.largeBlock.height * howManyRows);

      let smallBlocksRows = heightLeft / this.state.smallBlock.height;
      smallBlocksRows = smallBlocksRows < 1 ? 1 : Math.floor(smallBlocksRows);

      let countBlocksInSmallRow = this.state.dimensions.width / this.state.smallBlock.width;
      countBlocksInSmallRow = countBlocksInSmallRow < 1 ? 1 : Math.floor(countBlocksInSmallRow);

      this.setState({ smallRowsAddFlexClass: true, countBlocksInSmallRow, smallBlocksRows });
    }
  }

  public render() {
    this.state.lastConsensusInfo = JSON.stringify(this.props.appState.consensusInfo);
    const lastBlocks = this.props.appState.consensusInfo;

    if (this.props.appState.authorities.length > VIS_AUTHORITIES_LIMIT) {
      return <div className="Consensus">
        <div className="tooManyAuthorities">
          <p>
            Too many authorities.</p>
          <p>
            Won't display for more than {VIS_AUTHORITIES_LIMIT} authorities
            to protect your browser.
          </p>
        </div>;
      </div>;
    }

    if (this.props.appState.displayConsensusLoadingScreen && lastBlocks.length < 2) {
      return <div className="Consensus">
        <div className="noData">
          {lastBlocks.length === 0 ? "No " : "Not yet enough "}
          GRANDPA data received by the authorities&hellip;
        </div>;
      </div>;
    }

    let from = 0;
    let to = this.state.countBlocksInLargeRow;
    const firstLargeRow = this.getLargeRow(lastBlocks.slice(from, to), 0);

    from = to;
    to = to + this.state.countBlocksInLargeRow;
    const secondLargeRow = this.getLargeRow(lastBlocks.slice(from, to), 1);

    from = to;
    to = to + (this.state.smallBlocksRows * this.state.countBlocksInSmallRow);
    const smallRow = this.getSmallRow(lastBlocks.slice(from, to));

    const get = (measureRef: Maybe<(ref: Element | null) => void>) =>
      <div className="Consensus" ref={measureRef} key="Consensus">
        {firstLargeRow}
        {secondLargeRow}
        {smallRow}
      </div>;

    if (!(this.state.smallRowsAddFlexClass && this.state.largeRowsAddFlexClass)) {
      return (
        <React.Fragment>
          <Measure bounds={true} onResize={this.handleOnResize}>
            {({measureRef}) => get(measureRef)}
          </Measure>
        </React.Fragment>
      );
    } else {
      return (get(null));
    }
  }

  private handleOnResize = (contentRect: ContentRect) => {
    this.setState({ dimensions: contentRect.bounds as BoundingRect });
    this.calculateBoxCount(true);
  };

  private getAuthorities(): Types.Authority[] {
    // find the node for each of these authority addresses
    if (this.props.appState.authorities == null) {
      return [];
    }

    return this.props.appState.authorities.map(address => {
      const node2 = this.props.appState.nodes.sorted().filter(node => node.validator === address)[0];
      if (!node2) {
        return {Address: address, NodeId: null, Name: null} as Types.Authority;
      }
      return {Address: address, NodeId: node2.id, Name: node2.name} as Types.Authority;
    });
  }

  private getLargeRow(blocks: Types.ConsensusInfo, id: number) {
    const largeBlockSizeChanged = (isFirstBlock: boolean, rect: BoundingRect) => {
      if (this.largeBlocksSizeDetected(this.state)) {
        return;
      }
      if (isFirstBlock) {
        this.setState({largeBlockWithLegend: {width: rect.width, height: rect.height} });
      } else {
        this.setState({largeBlock: {width: rect.width, height: rect.height} });
      }
    };

    const stretchLastRowMajor = blocks.length < this.state.countBlocksInLargeRow ?
      'noStretchOnLastRow' : '';
    const flexClass = this.state.largeRowsAddFlexClass ? 'flexContainerLargeRow' : '';

    return <div
        className={`ConsensusList LargeRow ${flexClass} ${stretchLastRowMajor}`}
        key={`consensusList_${id}`}>
        {blocks.map((item, i) => {
           const [height, consensusView] = item;
           return <ConsensusBlock
             changeBlocks={largeBlockSizeChanged}
             firstInRow={i === 0}
             lastInRow={false}
             compact={false}
             key={height}
             height={height}
             measure={!this.state.largeRowsAddFlexClass}
             consensusView={consensusView}
             authorities={this.getAuthorities()}
             authoritySetId={this.props.appState.authoritySetId}
           />;
        })}
      </div>;
  }

  private getSmallRow(blocks: Types.ConsensusInfo) {
    const smallBlockSizeChanged = (isFirstBlock: boolean, rect: BoundingRect) => {
      if (this.smallBlocksSizeDetected(this.state)) {
        return;
      }
      const dimensionsChanged = this.state.smallBlock.height !== rect.height &&
        this.state.smallBlock.width !== rect.width;
      if (dimensionsChanged) {
        this.setState({ smallBlock: {width: rect.width, height: rect.height} });
      }
    };
    const stretchLastRow =
      blocks.length < this.state.countBlocksInSmallRow * this.state.smallBlocksRows ?
        'noStretchOnLastRow' : '';
    const classes = `ConsensusList SmallRow ${this.state.smallRowsAddFlexClass ? 'flexContainerSmallRow' : ''} ${stretchLastRow}`;

    return <div className={classes} key="smallRow">
      {blocks.map((item, i) => {
         const [height, consensusView] = item;
         let lastInRow = (i+1) % this.state.countBlocksInSmallRow === 0 ? true : false;
         if (lastInRow && i === 0) {
           // should not be marked as last one in row if it's the very first in row
           lastInRow = false;
         }

         return <ConsensusBlock
           changeBlocks={smallBlockSizeChanged}
           firstInRow={i === 0}
           lastInRow={lastInRow}
           compact={true}
           key={height}
           height={height}
           measure={!this.state.smallRowsAddFlexClass}
           consensusView={consensusView}
           authorities={this.getAuthorities()}
           authoritySetId={this.props.appState.authoritySetId} />;
         })
      }
      </div>;
  }

  private async subscribeConsensus(chain: Types.ChainLabel) {
    const connection = await this.props.connection;
    connection.subscribeConsensus(chain);
  }

  private async unsubscribeConsensus(chain: Types.ChainLabel) {
    const connection = await this.props.connection;
    connection.unsubscribeConsensus(chain);
  }
}
