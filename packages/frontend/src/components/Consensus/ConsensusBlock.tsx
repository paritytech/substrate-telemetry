import * as React from 'react';

import Measure, {BoundingRect, ContentRect} from 'react-measure';
import { Types, Maybe } from '@dotstats/common';

import { Icon, Tooltip, PolkadotIcon } from '../';
import Jdenticon from './Jdenticon';

import checkIcon from '../../icons/check.svg';
import finalizedIcon from '../../icons/finalized.svg';
import hatchingIcon from '../../icons/hatching.svg';

import './ConsensusBlock.css';

export namespace ConsensusBlock {
  export interface Props {
    authorities: Types.Authority[];
    authoritySetId: Maybe<Types.AuthoritySetId>;
    height: Types.BlockNumber;
    firstInRow: boolean;
    lastInRow: boolean;
    compact: boolean;
    measure: boolean;
    consensusView: Types.ConsensusView;
    changeBlocks: (first: boolean, boundsRect: BoundingRect) => void;
  }
}

export class ConsensusBlock extends React.Component<ConsensusBlock.Props, {}> {
  public state = {
    lastConsensusView: "",
  };

  public shouldComponentUpdate(nextProps: ConsensusBlock.Props): boolean {
    if (this.props.authorities.length === 0 && nextProps.authorities.length === 0) {
      return false;
    }

    const positionInfoChanged = this.props.firstInRow !== nextProps.firstInRow ||
      this.props.lastInRow !== nextProps.lastInRow;
    if (positionInfoChanged) {
      return true;
    }

    const newConsensusInfo =
      JSON.stringify(nextProps.consensusView) !== this.state.lastConsensusView;
    if (newConsensusInfo) {
      return true;
    }

    return false;
  }

  public render() {
    this.state.lastConsensusView = JSON.stringify(this.props.consensusView);
    const finalizedByWhom = this.props.authorities.filter(authority => this.isFinalized(authority));

    const ratio = finalizedByWhom.length + '/' + this.props.authorities.length;
    let titleFinal = <span>{ratio}</span>;

    const majorityFinalized = finalizedByWhom.length / this.props.authorities.length >= 2/3;
    if (majorityFinalized && !this.props.compact) {
      titleFinal = <span>FINAL</span>;
    } else if (majorityFinalized && this.props.compact) {
      const hash = this.getFinalizedHash(finalizedByWhom[0]);
      titleFinal = <Jdenticon hash={hash ? String(hash) : ''} size={this.props.compact ? '14px' : '28px'}/>
    }

    const handleOnResize = (contentRect: ContentRect) => {
      this.props.changeBlocks(this.props.firstInRow, contentRect.bounds as BoundingRect);
    };

    const get = (measureRef: Maybe<(ref: Element | null) => void>) => {
      return <div
        className={
          `BlockConsensusMatrice
          ${this.props.firstInRow ? 'firstInRow' : ''} ${this.props.lastInRow ? 'lastInRow' : ''}`
        }
        key={'block_' + this.props.height}>
        <table ref={measureRef} key={'block_table_' + this.props.height}>
          <thead key={'block_thead_' + this.props.height}>
          <tr className="Row" key={'block_row_' + this.props.height}>
            {this.props.firstInRow && !this.props.compact ?
              <th className="emptylegend" key={'block_row_' + this.props.height + '_empty'}>&nbsp;</th> : null}
            <th className="legend" key={'block_row_' + this.props.height + '_legend'}>
              <Tooltip text={`Block number: ${this.props.height}`}>
                {this.displayBlockNumber()}
              </Tooltip>
            </th>
            <th className='finalizedInfo' key={'block_row_' + this.props.height + '_finalized_info'}>
              {titleFinal}
            </th>
            {this.props.authorities.map(authority =>
              <th
                className="matrixXLegend"
                key={`${this.props.height}_matrice_x_${authority.Address}`}>
                {this.getAuthorityContent(authority)}
              </th>)}
          </tr>
          </thead>
          <tbody key={'block_row_' + this.props.height + '_tbody'}>
          {this.props.authorities.map((authority, row) =>
            this.renderMatriceRow(authority, this.props.authorities, row))}
          </tbody>
        </table>
      </div>
    };

    if (this.props.measure) {
      return (
      <Measure bounds={true} onResize={handleOnResize}>{({measureRef}) => (
        get(measureRef)
      )}
      </Measure>
      );
    } else {
      return (get(null));
    }
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
    let finalizedInfo = <span>&nbsp;</span>;
    let finalizedHash;

    if (authority.NodeId != null && this.isFinalized(authority)) {
      const matrice = this.props.consensusView[authority.Address][authority.Address];

      finalizedInfo = matrice.ImplicitFinalized ?
        <Icon className="implicit" src={finalizedIcon} alt="" /> :
        <Icon className="explicit" src={finalizedIcon} alt="" />;

      finalizedHash = matrice.FinalizedHash ?
        <Jdenticon hash={matrice.FinalizedHash} size="28px"/> :
        <div className="jdenticonPlaceholder">&nbsp;</div>;
    }

    const name = authority.Name ?
      <span>{authority.Name}</span> : <em>no data received from node</em>;
    const firstName = this.props.firstInRow ?
      <td key={"name_" + name} className="nameLegend">{name}</td> : '';

    return <tr className="Row" key={'block_row_' + this.props.height + '_' + row}>
      {firstName}
      <td className="legend" key={'block_row_' + this.props.height + '_' + row + '_legend'}>{this.getAuthorityContent(authority)}</td>
      <td className="finalizedInfo" key={'block_row_' + this.props.height + '_' + row + '_finalizedInfo'}>{finalizedInfo}{finalizedHash}</td>
      {
        authorities.map((columnNode, column) => {
          const evenOdd = ((row % 2) + column) % 2 === 0 ? 'even' : 'odd';
          return <td key={'matrice_' + this.props.height + '_' + row + '_' + authority.Address + '_' + columnNode.Address}
            className={`matrice ${evenOdd}`}>{this.getCellContent(authority, columnNode)}</td>
        })
      }
    </tr>;
  }

  private getAuthorityContent(authority: Types.Authority): JSX.Element {
    return <div className="nodeContent" key={'authority_' + this.props.height + '_' + authority.Address}>
      <div className="nodeAddress" key={'authority_' + authority.Address}>
        <PolkadotIcon account={authority.Address} size={this.props.compact ? 14 : 28} />
      </div>
    </div>;
  }

  private getCellContent(rowAuthority: Types.Authority, columnAuthority: Types.Authority) {
    const consensusInfo = this.props.consensusView &&
      rowAuthority.Address &&
      rowAuthority.Address in this.props.consensusView &&
      columnAuthority.Address in this.props.consensusView[rowAuthority.Address] ?
        this.props.consensusView[rowAuthority.Address][columnAuthority.Address] : null;

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

      return <span key={"icons_pre"}>{statPrevote}{statPrecommit}</span>;
    } else {
      return <Icon src={hatchingIcon} className="hatching" alt=""/>;
    }
  }

}
