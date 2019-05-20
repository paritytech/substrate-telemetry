import * as React from 'react';

import Measure, {BoundingRect, ContentRect} from 'react-measure';
import { Types, Maybe } from '@dotstats/common';
import Identicon from 'polkadot-identicon';

import { Icon, Tooltip } from '../';
import Jdenticon from './Jdenticon';

import checkIcon from '../../icons/check.svg';
import finalizedIcon from '../../icons/finalized.svg';
import hatchingIcon from '../../icons/hatching.svg';

import './ConsensusBlock.css';

export namespace ConsensusBlock {
  export interface Props {
    authorities: Types.Authority[];
    authoritySetId: Types.AuthoritySetId;
    height: Types.BlockNumber;
    firstInRow: boolean;
    lastInRow: boolean;
    compact: boolean;
    consensusView: Types.ConsensusView;
    changeBlocks: (first: boolean, boundsRect: BoundingRect) => void;
  }
}

export class ConsensusBlock extends React.Component<ConsensusBlock.Props, {}> {

  public render() {
    const finalizedByWhom = this.props.authorities.filter(authority => this.isFinalized(authority));

    const ratio = finalizedByWhom.length + '/' + this.props.authorities.length;
    const tooltip = `${ratio} authorities finalized this block. Authority Set Id: ${this.props.authoritySetId}.`;
    let titleFinal = <span>{ratio}</span>;

    const majorityFinalized = finalizedByWhom.length / this.props.authorities.length >= 2/3;
    if (majorityFinalized && !this.props.compact) {
      titleFinal = <span>FINAL</span>;
    } else if (majorityFinalized && this.props.compact) {
      const hash = this.getFinalizedHash(finalizedByWhom[0]);
      titleFinal =
        <Tooltip text={'Block hash: ' + hash ? String(hash) : ''} copy={true}>
          <Jdenticon hash={hash ? String(hash) : ''} size={this.props.compact ? '14px' : '28px'}/>
        </Tooltip>;
    }

    const handleOnResize = (contentRect: ContentRect) => {
      this.props.changeBlocks(this.props.firstInRow, contentRect.bounds as BoundingRect);
    };

    return (<Measure bounds={true} onResize={handleOnResize}>{({ measureRef }) => (
      <div
        className={`BlockConsensusMatrice ${this.props.firstInRow ? 'firstInRow' : ''} ${this.props.lastInRow ? 'lastInRow' : ''}`}
        key={'block_' + this.props.height}>
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
          {this.props.authorities.map(authority =>
            <th
              className="matrixXLegend"
              key={`${this.props.height}_matrice_x_${authority.Address}`}>
              {this.getAuthorityContent(authority)}
            </th>)}
        </tr>
        </thead>
        <tbody>
          {this.props.authorities.map((authority, row) =>
            this.renderMatriceRow(authority, this.props.authorities, row))}
        </tbody>
        </table>
      </div>)}
    </Measure>);
  }

  private displayBlockNumber(): string {
    const blockNumber = String(this.props.height);
    return blockNumber.length > 2 ?
      '…' + blockNumber.substr(blockNumber.length - 2, blockNumber.length) : blockNumber;
  }

  private isFinalized(authority: Types.Authority): boolean {
    if (!authority || authority.NodeId == null || authority.Address == null) {
      return false;
    }

    const { Address: addr } = authority;
    const consensus = this.props.consensusView;

    return consensus != null && addr in consensus && addr in consensus[addr]
      && consensus[addr][addr].Finalized === true;
  }

  private getFinalizedHash(authority: Types.Authority): Maybe<Types.BlockHash> {
    if (this.isFinalized(authority)) {
      const { Address: addr } = authority;
      return this.props.consensusView[addr][addr].FinalizedHash;
    }
    return null;
  }

  private renderMatriceRow(authority: Types.Authority, authorities: Types.Authority[], row: number): JSX.Element {
    let finalizedInfo = <Tooltip text="No information available yet.">&nbsp;</Tooltip>;
    let finalizedHash;

    if (authority.NodeId != null && this.isFinalized(authority)) {
      const matrice = this.props.consensusView[authority.Address][authority.Address];

      finalizedInfo = matrice.ImplicitFinalized ?
        <Tooltip text={`${authority.Name} finalized this block in ${matrice.ImplicitPointer}`}>
          <Icon className="implicit" src={finalizedIcon} alt="" />
        </Tooltip>
        :
        <Tooltip text={`${authority.Name} finalized this block in this block`}>
          <Icon className="explicit" src={finalizedIcon} alt="" />
        </Tooltip>

      finalizedHash = matrice.FinalizedHash ?
        <Tooltip text={`Block hash: ${matrice.FinalizedHash}`} copy={true}>
          <Jdenticon hash={matrice.FinalizedHash} size="28px"/>
        </Tooltip> : <div className="jdenticonPlaceholder">&nbsp;</div>;
    }

    const name = authority.Name ? <span>{authority.Name}</span> : <em>no name received yet</em>;
    const firstName = this.props.firstInRow ? <td className="nameLegend">{name}</td> : '';

    return <tr className="Row">
      {firstName}
      <td className="legend">{this.getAuthorityContent(authority)}</td>
      <td className="finalizedInfo">{finalizedInfo}{finalizedHash}</td>
      {
        authorities.map((columnNode, column) => {
          const evenOdd = ((row % 2) + column) % 2 === 0 ? 'even' : 'odd';
          return <td key={'matrice_' + authority.Address + '_' + columnNode.Address}
            className={`matrice ${evenOdd}`}>{this.getMatriceContent(authority, columnNode)}</td>
        })
      }
    </tr>;
  }

  private getAuthorityContent(authority: Types.Authority): JSX.Element {
    return <div className="nodeContent">
      <div className="nodeAddress">
        <Tooltip text={authority.Address} copy={true}>
          <Identicon account={authority.Address} size={this.props.compact ? 14 : 28} />
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

  private getMatriceContent(rowAuthority: Types.Authority, columnAuthority: Types.Authority) {
    const consensusInfo = this.props.consensusView &&
      rowAuthority.Address &&
      rowAuthority.Address in this.props.consensusView &&
      columnAuthority.Address in this.props.consensusView[rowAuthority.Address] ?
      this.props.consensusView[rowAuthority.Address][columnAuthority.Address] : null;

    let tooltipText = consensusInfo ?
        rowAuthority.Name + ' has seen this of ' + columnAuthority.Name + ': ' +
        this.format(consensusInfo) : 'No information available yet.';

    if (rowAuthority.Address === columnAuthority.Address) {
      tooltipText = 'Self-referential.';
    }

    const prevote = consensusInfo && consensusInfo.Prevote;
    const implicitPrevote = consensusInfo && consensusInfo.ImplicitPrevote;

    const precommit = consensusInfo && consensusInfo.Precommit;
    const implicitPrecommit = consensusInfo && consensusInfo.ImplicitPrecommit;

    if (rowAuthority.Address !== columnAuthority.Address) {
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
