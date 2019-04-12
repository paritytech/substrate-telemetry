import * as React from 'react';
import { Types } from '@dotstats/common';
import Measure, {BoundingRect, ContentRect} from 'react-measure';

import { ConsensusBlock } from './';
import { State as AppState } from '../../state';

import './Consensus.css';

export namespace Consensus {
  export interface Props {
    appState: Readonly<AppState>;
  }

  export interface State {
    dimensions: BoundingRect;
  }
}

export class Consensus extends React.Component<Consensus.Props, {}> {
  public state = {
    // entire area available for rendering the visualization
    dimensions: { width: -1, height: -1 } as BoundingRect,

    largeBlockWithLegend: { width: -1, height: -1 },
    largeBlock: { width: -1, height: -1 },
    countBlocksInLargeRow: 2,
    largeRowsAddFlexClass: false,

    smallBlock: { width: -1, height: -1 },
    smallBlocksRows: 1,
    countBlocksInSmallRow: 1,
    smallRowsAddFlexClass: false,
  };

  public largeBlocksSizeDetected(): boolean {
    return this.state.largeBlockWithLegend.width > -1 && this.state.largeBlockWithLegend.height > -1 &&
      this.state.largeBlock.width > -1 && this.state.largeBlock.height > -1;
  }

  public smallBlocksSizeDetected(): boolean {
    return this.state.smallBlock.width > -1 && this.state.largeBlockWithLegend.height > -1;
  }

  public calculateBoxCount(wasResized: boolean) {
    // if the css class for flexing has already been added we don't calculate
    // any box measurements then, because the box sizes would be skewed then.
    if ((wasResized || this.state.largeRowsAddFlexClass === false) && this.largeBlocksSizeDetected()) {
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

    if ((wasResized || this.state.smallRowsAddFlexClass === false) && this.smallBlocksSizeDetected()) {
      const howManyRows = 2;

      const heightLeft = this.state.dimensions.height - (this.state.largeBlock.height * howManyRows);

      let smallBlocksRows = heightLeft / this.state.smallBlock.height;
      smallBlocksRows = smallBlocksRows < 1 ? 1 : Math.floor(smallBlocksRows);

      let countBlocksInSmallRow = this.state.dimensions.width / this.state.smallBlock.width;
      countBlocksInSmallRow = countBlocksInSmallRow < 1 ? 1 : Math.floor(countBlocksInSmallRow);

      this.setState({ smallRowsAddFlexClass: true, countBlocksInSmallRow, smallBlocksRows });
    }
  }

  public shouldComponentUpdate(nextProps: Consensus.Props, nextState: Consensus.State): boolean {
    if (this.props.appState.authorities.length === 0 && nextProps.appState.authorities.length === 0) {
      return false;
    }
    if (this.props.appState.nodes.sorted().length === 0 && nextProps.appState.nodes.sorted().length === 0) {
      return false;
    }

    const authoritiesDidChange = JSON.stringify(this.props.appState.authorities) !==
      JSON.stringify(nextProps.appState.authorities);
    const authoritySetIdDidChange = this.props.appState.authoritySetId !==
      nextProps.appState.authoritySetId;

    const newConsensusInfoAvailable = JSON.stringify(this.props.appState.consensusInfo) !==
      JSON.stringify(nextProps.appState.consensusInfo);

    const windowSizeChanged = JSON.stringify(this.state.dimensions) !==
      JSON.stringify(nextState.dimensions);

    return authoritiesDidChange || authoritySetIdDidChange || newConsensusInfoAvailable ||
      windowSizeChanged;
  }

  public render() {
    this.calculateBoxCount(false);

    const lastBlocks = Object.keys(this.props.appState.consensusInfo).reverse();

    let from = 0;
    let to = this.state.countBlocksInLargeRow;
    const firstLargeRow = this.getLargeRow(lastBlocks.slice(from, to), 0);

    from = to;
    to = to + this.state.countBlocksInLargeRow;
    const secondLargeRow = this.getLargeRow(lastBlocks.slice(from, to), 1);

    from = to;
    to = to + (this.state.smallBlocksRows * this.state.countBlocksInSmallRow);
    const smallRow = this.getSmallRow(lastBlocks.slice(from, to));

    return (
      <React.Fragment>
        <Measure bounds={true} onResize={this.handleOnResize}>
          {({ measureRef }) => (
            <div className="allRows" ref={measureRef}>
              {firstLargeRow}
              {secondLargeRow}
              {smallRow}
            </div>
          )}
        </Measure>
      </React.Fragment>
    );
  }

  private handleOnResize = (contentRect: ContentRect) => {
    this.setState({ dimensions: contentRect.bounds as BoundingRect });
    this.calculateBoxCount(true);
  };

  private getAuthorities(): Types.Authority[] {
    // find the node for each of these authority addresses
    return this.props.appState.authorities.map(address => {
      const node2 = this.props.appState.nodes.sorted().filter(node => node.address === address)[0];
      if (!node2) {
        return {Address: address, NodeId: null, Name: null} as Types.Authority;
      }
      return {Address: address, NodeId: node2.id, Name: node2.name} as Types.Authority;
    });
  }

  private getLargeRow(blocks: string[], id: number) {
    const largeBlockSizeChanged = (isFirstBlock: boolean, rect: BoundingRect) => {
      if (this.largeBlocksSizeDetected()) {
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
        {blocks.map((height, i) =>
           <ConsensusBlock
             changeBlocks={largeBlockSizeChanged}
             firstInRow={i === 0}
             lastInRow={false}
             compact={false}
             key={height}
             height={parseInt(height, 10) as Types.BlockNumber}
             consensusView={this.props.appState.consensusInfo[height]}
             authorities={this.getAuthorities()}
             authoritySetId={this.props.appState.authoritySetId}
             authoritySetBlockNumber={this.props.appState.authoritySetBlockNumber}
           />)}
      </div>;
  }

  private getSmallRow(blocks: string[]) {
    const smallBlockSizeChanged = (isFirstBlock: boolean, rect: BoundingRect) => {
      if (this.smallBlocksSizeDetected()) {
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
      {blocks.map((height, i) => {
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
           height={parseInt(height, 10) as Types.BlockNumber}
           consensusView={this.props.appState.consensusInfo[height]}
           authorities={this.getAuthorities()}
           authoritySetBlockNumber={this.props.appState.authoritySetBlockNumber}
           authoritySetId={this.props.appState.authoritySetId} />;
         })
      }
      </div>;
  }
}
