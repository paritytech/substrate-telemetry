import { Types } from '@dotstats/common';
import { State, UpdateBound } from './state';

// Number of blocks which are kept in memory
const BLOCKS_LIMIT = 50;

export class AfgHandling {
  private updateState: UpdateBound;
  private getState: () => Readonly<State>;

  constructor(
    updateState: UpdateBound,
    getState: () => Readonly<State>,
  ) {
    this.updateState = updateState;
    this.getState = getState;
  }

  public receivedAuthoritySet(
    authoritySetId: Types.AuthoritySetId,
    authorities: Types.Authorities,
  ) {
    if (this.getState().authoritySetId != null && authoritySetId !== this.getState().authoritySetId) {
      // the visualization is restarted when we receive a new auhority set
      this.updateState({
        authoritySetId,
        authorities,
        consensusInfo: [],
        displayConsensusLoadingScreen: false,
      });
    } else if (this.getState().authoritySetId == null) {
      // initial display
      this.updateState({
        authoritySetId,
        authorities,
        consensusInfo: [],
        displayConsensusLoadingScreen: true,
      });
    }
    return null;
  }

  public receivedFinalized(
    addr: Types.Address,
    finalizedNumber: Types.BlockNumber,
    finalizedHash: Types.BlockHash,
  ) {
    const state = this.getState();
    if (finalizedNumber < state.best - BLOCKS_LIMIT) {
      return;
    };

    const data = {
      Finalized: true,
      FinalizedHash: finalizedHash,
      FinalizedHeight: finalizedNumber,

      // this is extrapolated. if this app was just started up we
      // might not yet have received prevotes/precommits. but
      // those are a necessary precondition for finalization, so
      // we can set them and display them in the ui.
      Prevote: true,
      Precommit: true,
    } as Types.ConsensusDetail;
    this.initialiseConsensusView(state.consensusInfo, finalizedNumber, addr, addr);

    this.updateConsensusInfo(state.consensusInfo, finalizedNumber, addr, addr, data as Partial<Types.ConsensusDetail>);

    // Finalizing a block implicitly includes finalizing all
    // preceding blocks. This function marks the preceding
    // blocks as implicitly finalized on and stores a pointer
    // to the block which contains the explicit finalization.
    const op = (i: Types.BlockNumber, index: number) : boolean => {
      const consensusDetail = state.consensusInfo[index][1][addr][addr];
      if (consensusDetail.Finalized || consensusDetail.ImplicitFinalized) {
        return false;
      }

      state.consensusInfo[index][1][addr][addr] = {
        Finalized: true,
        FinalizedHeight: i,
        ImplicitFinalized: true,
        ImplicitPointer: finalizedNumber,

        // this is extrapolated. if this app was just started up we
        // might not yet have received prevotes/precommits. but
        // those are a necessary precondition for finalization, so
        // we can set them and display them in the ui.
        Prevote: true,
        Precommit: true,
        ImplicitPrevote: true,
        ImplicitPrecommit: true,
      };
      return true;
    };
    this.backfill(state.consensusInfo, finalizedNumber, op, addr, addr);

    this.pruneBlocks(state.consensusInfo);
    this.updateState({consensusInfo: state.consensusInfo});
  }

  public receivedPre(
    addr: Types.Address,
    height: Types.BlockNumber,
    hash: Types.BlockHash,
    voter: Types.Address,
    what: string,
  ) {
    const state = this.getState();
    if (height < state.best - BLOCKS_LIMIT) {
      return;
    };

    const data = what === "prevote" ? { Prevote: true } : { Precommit: true };
    this.initialiseConsensusView(state.consensusInfo, height, addr, voter);
    this.updateConsensusInfo(state.consensusInfo, height, addr, voter, data as Partial<Types.ConsensusDetail>);

    // A Prevote or Precommit on a block implicitly includes
    // a vote on all preceding blocks. This function marks
    // the preceding blocks as implicitly voted on and stores
    // a pointer to the block which contains the explicit vote.
    const op = (i: Types.BlockNumber, index: number) : boolean => {
      const consensusDetail = state.consensusInfo[index][1][addr][voter];
      if (what === "prevote" && (consensusDetail.Prevote || consensusDetail.ImplicitPrevote)) {
        return false;
      }
      if (what === "precommit" && (consensusDetail.Precommit || consensusDetail.ImplicitPrecommit)

          // because of extrapolation a prevote needs to be set as well.
          // if it is not we continue backfilling (and set it during that process).
          && (consensusDetail.Prevote || consensusDetail.ImplicitPrevote)) {
        return false;
      }

      if (what === "prevote") {
        consensusInfo[index][1][addr][voter].ImplicitPrevote = true;
      } else if (what === "precommit") {
        consensusInfo[index][1][addr][voter].ImplicitPrecommit = true;

        // Extrapolate. Precommit implies Prevote.
        consensusInfo[index][1][addr][voter].ImplicitPrevote = true;
      }
      consensusInfo[index][1][addr][voter].ImplicitPointer = height;
      return true;
    };
    const consensusInfo = this.getState().consensusInfo;
    this.backfill(consensusInfo, height, op, addr, voter);

    this.pruneBlocks(consensusInfo);
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
      this.initialiseConsensusViewByRef(consensusView, own, other);
    } else {
      consensusView = {} as Types.ConsensusView;

      this.initialiseConsensusViewByRef(consensusView, own, other);

      const item: Types.ConsensusItem = [height, consensusView];
      const insertPos = consensusInfo.findIndex(([elHeight, elView]) => elHeight < height);
      if (insertPos >= 0) {
        consensusInfo.splice(insertPos, 0, item);
      } else {
        consensusInfo.push(item);
      }
    }
  }

  // Initializes the `ConsensusView` with empty objects.
  private initialiseConsensusViewByRef(
    consensusView: Types.ConsensusView,
    own: Types.Address,
    other: Types.Address,
  ) {
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
    start: Types.BlockNumber,
    f: (i: Types.BlockNumber, index: number) => boolean,
    own: Types.Address,
    other: Types.Address,
  ) {
    // if this is the first block then we don't fill latter blocks
    // if there is only one block, then it also doesn't make
    // sense to backfill, because we could potentially backfill
    // until 0 (which could be unfortunate if the first received
    // block is e.g. 28317.
    if (consensusInfo.length < 2) {
      return;
    }

    let firstBlockNumber = consensusInfo[consensusInfo.length - 1][0];
    const limit = this.getState().best - BLOCKS_LIMIT;
    if (firstBlockNumber < limit) {
      firstBlockNumber = limit as Types.BlockNumber;
    }

    if (start - 1 < firstBlockNumber) {
      // if the first block which would be backfilled is already
      // less than the first block number we can abort.
      //
      // this can happen if e.g. one authority is hanging behind,
      // most of them could e.g. be at 3000 and one is hanging behind
      // and sending info for 2000. then we can't start backfilling
      // from 2000.
      return;
    }

    let counter = 0;
    while (start-- > 0) {
      counter++;
      if (counter >= BLOCKS_LIMIT) {
        break;
      }

      const startBlockNumber = start as Types.BlockNumber;
      this.initialiseConsensusView(consensusInfo, startBlockNumber, own, other);
      const index =
        consensusInfo.findIndex(([blockNumber,]) => blockNumber === start);
      const cont = f(start, index);
      if (!cont) {
        break;
      }

      // we don't want to fill into nirvana
      const firstBlockReached = startBlockNumber <= firstBlockNumber;
      if (firstBlockReached) {
        break;
      }
    }
  }

  private updateConsensusInfo(
    consensusInfo: Types.ConsensusInfo,
    height: Types.BlockNumber,
    addr: Types.Address,
    voter: Types.Address,
    data: Partial<Types.ConsensusDetail>,
  ) {
    const found =
      consensusInfo.findIndex(([blockNumber,]) => blockNumber === height);
    if (found < 0) {
      return;
    }

    for (const k in data) {
      if (data.hasOwnProperty(k)) {
        consensusInfo[found][1][addr][voter][k] = data[k];
      }
    }
  }

  private pruneBlocks(consensusInfo: Types.ConsensusInfo) {
    if (consensusInfo.length >= BLOCKS_LIMIT) {
      consensusInfo.length = BLOCKS_LIMIT;
    }
  }
}
