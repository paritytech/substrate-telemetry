import * as React from 'react';
import { CSSTransitionGroup } from 'react-transition-group';

import Measure, {BoundingRect, ContentRect} from 'react-measure';
import { Types } from '@dotstats/common';
import Identicon from 'polkadot-identicon';

import { Node } from '../../state';
import { Icon, Tooltip } from '../';
import Jdenticon from './Jdenticon';

import checkIcon from '../../icons/check.svg';
import finalizedIcon from '../../icons/finalized.svg';
import hatchingIcon from '../../icons/hatching.svg';

import './ConsensusBlock.css';

export namespace ConsensusBlock {
  export interface Props {
    authorities: Node[];
    authoritySetId: Types.AuthoritySetId;
    authoritySetBlockNumber: Types.BlockNumber;
    height: Types.BlockNumber;
    firstInRow: boolean;
    lastInRow: boolean;
    animateOnAppearing: boolean;
    compact: boolean;
    consensusView: Types.ConsensusView;
    changeBlocks: (first: boolean, boundsRect: BoundingRect) => void;
  }
}

export class ConsensusBlock extends React.Component<ConsensusBlock.Props, {}> {

  public shouldComponentUpdate(nextProps: ConsensusBlock.Props): boolean {
    if (this.props.authorities.length === 0 && nextProps.authorities.length === 0) {
      return false;
    }

    const newConsensusInfo =
      JSON.stringify(nextProps.consensusView) !== JSON.stringify(this.props.consensusView);
    const positionInfoChanged = this.props.firstInRow !== nextProps.firstInRow ||
      this.props.lastInRow !== nextProps.lastInRow;

    return newConsensusInfo || positionInfoChanged;
  }

  public render() {
    const finalizedByWhom = this.props.authorities.filter(node => this.isFinalized(node));
    const ratio = finalizedByWhom.length + '/' + this.props.authorities.length;
    const tooltip = `${ratio} authorities finalized this block. Authority Set Id: ${this.props.authoritySetId}.`;
    let titleFinal = <span>{ratio}</span>;

    const majorityFinalized = finalizedByWhom.length / this.props.authorities.length >= 2/3;
    if (majorityFinalized && !this.props.compact) {
      titleFinal = <span>FINAL</span>;
    } else if (majorityFinalized && this.props.compact) {
      const hash = this.getFinalizedHash(finalizedByWhom[0]);
      titleFinal =
        <Tooltip text={'Block hash: ' + hash} copy={true}>
          <Jdenticon hash={hash} size={this.props.compact ? '14px' : '28px'}/>
        </Tooltip>;
    }

    const handleOnResize = (contentRect: ContentRect) => {
      this.props.changeBlocks(this.props.firstInRow, contentRect.bounds as BoundingRect);
    };

    return (<Measure bounds={true} onResize={handleOnResize}>{({ measureRef }) => (
      <div
        className={`BlockConsensusMatrice ${this.props.firstInRow ? 'firstInRow' : ''} ${this.props.lastInRow ? 'lastInRow' : ''}`}
        key={'block_' + this.props.height}>
        <CSSTransitionGroup
          key={'animate_' + this.props.height}
          transitionName="blockTransition"
          transitionAppear={this.props.animateOnAppearing}
          transitionAppearTimeout={3000}
          transitionEnter={false}
          transitionLeave={false}>
          <table ref={measureRef}>
          <thead>
          <tr className="Row">
            {this.props.firstInRow && !this.props.compact ?
              <th className="emptylegend">&nbsp;</th> : ''}
            <th className="legend">
              <Tooltip text={`Block number: ${this.props.height}`}>
                {this.displayBlockNumber()}
              </Tooltip>
            </th>
            <th className='finalizedInfo'>
              <Tooltip text={tooltip}>{titleFinal}</Tooltip>
            </th>
            {this.props.authorities.map(node =>
              <th
                className="matrixXLegend"
                key={`${this.props.height}_matrice_x_${node.address}`}>
                {this.getNodeContent(node, false)}
              </th>)}
          </tr>
          </thead>
          <tbody>
            {this.props.authorities.map((node, row) =>
              this.renderMatriceRow(node, this.props.authorities, row))}
          </tbody>
          </table>
        </CSSTransitionGroup>
      </div>)}
    </Measure>);
  }

  private displayBlockNumber(): string {
    const blockNumber = String(this.props.height);
    return blockNumber.length > 2 ?
      '…' + blockNumber.substr(blockNumber.length - 2, blockNumber.length) : blockNumber;
  }

  private isFinalized(node: Node): boolean {
    const { address } = node;
    const consensus = this.props.consensusView;

    return consensus !== undefined &&
      address in consensus &&
      address in consensus[address] &&
      consensus[address][address].Finalized === true;
  }

  private getFinalizedHash(node: Node): Types.BlockHash {
    const { address } = node;
    const consensus = this.props.consensusView;

    if (consensus !== undefined &&
      address in consensus &&
      address in consensus[address] &&
      consensus[address][address].Finalized === true) {
      return consensus[address][address].FinalizedHash;
    }
    return '' as Types.BlockHash;
  }

  private renderMatriceRow(node: Node, authorities: Node[], row: number): JSX.Element {
    let finalizedInfo = <Tooltip text="No information available yet.">&nbsp;</Tooltip>;
    let finalizedHash;

    if (this.isFinalized(node)) {
      const matrice = this.props.consensusView[node.address][node.address];
      finalizedInfo = matrice.ImplicitFinalized ?
        <Tooltip text={`${node.name} finalized this block in ${matrice.ImplicitPointer}`}>
          <Icon className="implicit" src={finalizedIcon} alt="" />
        </Tooltip>
        :
        <Tooltip text={`${node.name} finalized this block in this block`}>
          <Icon className="explicit" src={finalizedIcon} alt="" />
        </Tooltip>

      finalizedHash = matrice.FinalizedHash ?
        <Tooltip text={`Block hash: ${matrice.FinalizedHash}`} copy={true}>
          <Jdenticon hash={matrice.FinalizedHash} size="28px"/>
        </Tooltip> : <div className="jdenticonPlaceholder">&nbsp;</div>;
    }

    const firstName = this.props.firstInRow ? <td className="nameLegend">{node.name}</td> : '';

    return <tr className="Row">
      {firstName}
      <td className="legend">{this.getNodeContent(node, true)}</td>
      <td className="finalizedInfo">{finalizedInfo}{finalizedHash}</td>
      {
        authorities.map((columnNode, column) => {
          const evenOdd = ((row % 2) + column) % 2 === 0 ? 'even' : 'odd';
          return <td key={'matrice_' + node.address + '_' + columnNode.address}
            className={`matrice ${evenOdd}`}>{this.getMatriceContent(node, columnNode)}</td>
        })
      }
    </tr>;
  }

  private getNodeContent(node: Node, nodeName: boolean): JSX.Element {
    return <div className="nodeContent">
      <div className="nodeAddress">
        <Tooltip text={node.address} copy={true}>
          <Identicon account={node.address} size={this.props.compact ? 14 : 28} />
        </Tooltip>
      </div>
    </div>;
  }

  private format(consensusDetail: Types.ConsensusDetail): string {
    const txt = [];
    if (consensusDetail.Prevote) {
      txt.push('Prevote on this chain in this block');
    } else if (consensusDetail.ImplicitPrevote) {
      txt.push('Prevote on this chain in block ' + consensusDetail.ImplicitPointer);
    }
    if (consensusDetail.Precommit) {
      txt.push('Precommit on this chain in this block');
    } else if (consensusDetail.ImplicitPrecommit) {
      txt.push('Precommit on this chain in block ' + consensusDetail.ImplicitPointer);
    }
    if (consensusDetail.Finalized && consensusDetail.ImplicitFinalized) {
      txt.push('Finalized this chain in block ' + consensusDetail.ImplicitPointer);
    } else if (consensusDetail.Finalized && !consensusDetail.ImplicitFinalized) {
      txt.push('Finalized this chain in this block');
    }
    return txt.join(', '); // + JSON.stringify((consensusDetail));
  }

  private getMatriceContent(rowNode: Node, columnNode: Node) {
    const consensusInfo = this.props.consensusView &&
      rowNode.address in this.props.consensusView &&
      columnNode.address in this.props.consensusView[rowNode.address] ?
      this.props.consensusView[rowNode.address][columnNode.address] : null;

    let tooltipText = consensusInfo ?
        rowNode.name + ' has seen this of ' + columnNode.name + ': ' +
        this.format(consensusInfo) : 'No information available yet.';

    if (rowNode.address === columnNode.address) {
      tooltipText = 'Self-referential.';
    }

    const prevote = consensusInfo && consensusInfo.Prevote;
    const implicitPrevote = consensusInfo && consensusInfo.ImplicitPrevote;

    const precommit = consensusInfo && consensusInfo.Precommit;
    const implicitPrecommit = consensusInfo && consensusInfo.ImplicitPrecommit;

    if (rowNode.address !== columnNode.address) {
      let statPrevote;
      let statPrecommit;

      if (implicitPrevote) {
        statPrevote = <Icon src={checkIcon} className="implicit" alt="Implicit Prevote"/>;
      }
      if (implicitPrecommit) {
        statPrecommit = <Icon src={checkIcon} className="implicit" alt="Implicit Precommit"/>;
      }

      if (prevote) {
        statPrevote = <Icon src={checkIcon} className="explicit" alt="Prevote"/>;
      }
      if (precommit) {
        statPrecommit = <Icon src={checkIcon} className="explicit" alt="Precommit"/>;
      }

      const stat = [statPrevote, statPrecommit];
      return <Tooltip text={tooltipText}>{stat}</Tooltip>
    } else {
      return <Tooltip text={tooltipText}>
        <Icon src={hatchingIcon} className="hatching" alt=""/>
      </Tooltip>
    }
  }

}
