import { Types, Maybe } from '@dotstats/common';
import { State } from './state';

export class AfgHandling {
  private updateState: (state: any) => void;
  private getState: () => State;

  constructor(
    updateState: (state: any) => void,
    getState: () => State,
  ) {
    this.updateState = updateState;
    this.getState = getState;
  }

  public receivedAuthoritySet(
    authoritySetId: Types.AuthoritySetId,
    authorities: Types.Authorities,
  ) {
    if (authoritySetId !== this.getState().authoritySetId) {
      // the visualization is restarted when we receive a new auhority set
      this.updateState({authoritySetId, authorities, consensusInfo: []});
    }
    return null;
  }

  public receivedFinalized(
    addr: Types.Address,
    finalizedNumber: Types.BlockNumber,
    finalizedHash: Types.BlockHash,
  ) {
    const consensusInfo = this.getState().consensusInfo;
    this.markFinalized(addr, finalizedNumber, finalizedHash);

    const op = (i: Types.BlockNumber, view: Types.ConsensusView) => {
      const consensusDetail = view[addr][addr];
      if (consensusDetail.Finalized || consensusDetail.ImplicitFinalized) {
        return false;
      }

      this.markImplicitlyFinalized(i, addr, finalizedNumber, addr);
      return true;
    };
    this.backfill(consensusInfo, finalizedNumber, op, addr, addr);
  }

  public receivedPre(
    addr: Types.Address,
    height: Types.BlockNumber,
    hash: Types.BlockHash,
    voter: Types.Address,
    what: string,
  ) {
    const data = what === "prevote" ? { Prevote: true } : { Precommit: true };
    this.updateConsensusInfo(height, addr, voter, data as Partial<Types.ConsensusDetail>);

    const op = (i: Types.BlockNumber, view: Types.ConsensusView) => {
      const consensusDetail = view[addr][voter];
      if (consensusDetail.Prevote || consensusDetail.ImplicitPrevote) {
        return false;
      }

      this.markImplicitlyPre(i, addr, height, what, voter);
      return true;
    };
    this.backfill(this.getState().consensusInfo, height, op, addr, voter);
  }

  private markFinalized(
    addr: Types.Address,
    finalizedHeight: Types.BlockNumber,
    finalizedHash: Types.BlockHash,
  ) {
    const data = {
      Finalized: true,
      FinalizedHash: finalizedHash,
      FinalizedHeight: finalizedHeight,

      // this is extrapolated. if this app was just started up we
      // might not yet have received prevotes/precommits. but
      // those are a necessary precondition for finalization, so
      // we can set them and display them in the ui.
      Prevote: true,
      Precommit: true,
    } as Types.ConsensusDetail;
    this.updateConsensusInfo(finalizedHeight, addr, addr, data);
  }

  // A Prevote or Precommit on a block implicitly includes
  // a vote on all preceding blocks. This function marks
  // the preceding blocks as implicitly voted on and stores
  // a pointer to the block which contains the explicit vote.
  private markImplicitlyPre(
    height: Types.BlockNumber,
    addr: Types.Address,
    where: Types.BlockNumber,
    what: string,
    voter: Types.Address,
  ) {
    const consensusInfo = this.getState().consensusInfo;
    this.initialiseConsensusView(consensusInfo, height, addr, voter);

    const [consensusView, index] = this.getConsensusView(consensusInfo, height);

    if (what === "prevote") {
      consensusView[addr][voter].ImplicitPrevote = true;
    } else if (what === "precommit") {
      consensusView[addr][voter].ImplicitPrecommit = true;
    }
    consensusView[addr][voter].ImplicitPointer = where;

    consensusInfo[index] = [height, consensusView];
    this.updateState({consensusInfo});
  }

  // Finalizing a block implicitly includes finalizing all
  // preceding blocks. This function marks the preceding
  // blocks as implicitly finalized on and stores a pointer
  // to the block which contains the explicit finalization.
  private markImplicitlyFinalized(
    height: Types.BlockNumber,
    addr: Types.Address,
    to: Types.BlockNumber,
    voter: Types.Address,
  ) {
    const consensusInfo = this.getState().consensusInfo;
    this.initialiseConsensusView(consensusInfo, height, addr, voter);

    const [consensusView, index] = this.getConsensusView(consensusInfo, height);

    const consensusDetail = {
      Finalized: true,
      FinalizedHeight: height,
      ImplicitFinalized: true,
      ImplicitPointer: to,

      // this is extrapolated. if this app was just started up we
      // might not yet have received prevotes/precommits. but
      // those are a necessary precondition for finalization, so
      // we can set them and display them in the ui.
      Prevote: true,
      Precommit: true,
      ImplicitPrevote: true,
      ImplicitPrecommit: true,
    };
    consensusView[addr][voter] = consensusDetail;

    if (index !== -1) {
      consensusInfo[index] = [height, consensusView];
    } else {
      consensusInfo.unshift([height, consensusView]);
    }

    this.updateState({consensusInfo});
  }

  // Initializes the `ConsensusView` with empty objects.
  private initialiseConsensusView(
    consensusInfo: Types.ConsensusInfo,
    height: Types.BlockNumber,
    own: Types.Address,
    other: Types.Address,
  ) {
    const found =
      consensusInfo.find(([blockNumber,]) => blockNumber === height);

    let consensusView;
    if (found) {
      [, consensusView] = found;
    } else {
      consensusView = {} as Types.ConsensusView;
      consensusInfo.unshift([height, consensusView]);
    }

    if (!consensusView[own]) {
      consensusView[own] = {} as Types.ConsensusState;
    }

    if (!consensusView[own][other]) {
      consensusView[own][other] = {} as Types.ConsensusDetail;
    }
  }

  // Fill the block cache back from the `to` number to the last block.
  // The function `f` is used to evaluate if we should continue backfilling.
  // `f` returns false when backfilling the cache should be stopped, true to continue.
  //
  // Returns block number until which we backfilled.
  private backfill(
    consensusInfo: Types.ConsensusInfo,
    to: Types.BlockNumber,
    f: Maybe<(i: Types.BlockNumber, consensusView: Types.ConsensusView) => boolean>,
    own: Types.Address,
    other: Types.Address,
  ): Types.BlockNumber {
    for (const [height, consensusView] of consensusInfo) {
      if (height >= to) {
        continue;
      }

      this.initialiseConsensusView(consensusInfo, height, own, other);

      const cont = f ? f(height, consensusView) : true;
      if (!cont) {
        break;
      }
    }
    return to;
  }

  private getConsensusView(
    consensusInfo: Types.ConsensusInfo,
    height: Types.BlockNumber,
  ): [Types.ConsensusView, number] {
    const index =
      consensusInfo.findIndex(([blockNumber,]) => blockNumber === height);
    const [, consensusView] = consensusInfo[index];
    return [consensusView, index];
  }

  private updateConsensusInfo(
    height: Types.BlockNumber,
    addr: Types.Address,
    voter: Types.Address,
    data: Partial<Types.ConsensusDetail>,
  ) {
    const consensusInfo = this.getState().consensusInfo;
    this.initialiseConsensusView(consensusInfo, height, addr, voter);

    const index = consensusInfo.findIndex(([blockNumber,]) => blockNumber === height);
    const [, consensusView] = consensusInfo[index];

    for (const o in data) {
      if (data.hasOwnProperty(o)) {
        consensusView[addr][voter][o] = data[o];
      }
    }

    if (index !== -1) {
      consensusInfo[index] = [height, consensusView];
    } else {
      // append at the beginning
      consensusInfo.unshift([height, consensusView]);
    }

    this.updateState({consensusInfo});
  }
}
