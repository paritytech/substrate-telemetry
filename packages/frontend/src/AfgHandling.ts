import { Types, Maybe } from '@dotstats/common';
import { State } from './state';

export function afgAuthoritySet(
  authoritySetId: Types.AuthoritySetId,
  authorities: Types.Authorities,
  updateState: (state: any) => void,
  getState: () => State,
) {
  if (authoritySetId !== getState().authoritySetId) {
    // the visualization is restarted when we receive a new auhority set
    updateState({ authoritySetId, authorities, consensusInfo: [] });
  }
  return null;
}

export function afgFinalized(
  addr: Types.Address,
  finalizedNumber: Types.BlockNumber,
  finalizedHash: Types.BlockHash,
  updateState: (state: any) => void,
  getState: () => State,
) {
  const consensusInfo = getState().consensusInfo;
  markFinalized(addr, finalizedNumber, finalizedHash, updateState, getState);

  const op = (i: Types.BlockNumber, view: Types.ConsensusView) => {
    const consensusDetail = view[addr][addr];
    if (consensusDetail.Finalized || consensusDetail.ImplicitFinalized) {
      return false;
    }

    markImplicitlyFinalized(i, addr, finalizedNumber, addr, updateState, getState);
    return true;
  };
  backfill(consensusInfo, finalizedNumber, op, addr, addr);
}

export function afgMarkPre(
  addr: Types.Address,
  height: Types.BlockNumber,
  hash: Types.BlockHash,
  voter: Types.Address,
  what: string,
  updateState: (state: any) => void,
  getState: () => State,
) {
  const consensusInfo = getState().consensusInfo;
  initialiseConsensusView(consensusInfo, height, addr, voter);

  const index = consensusInfo.findIndex(([blockNumber,]) => blockNumber === height);
  const [, consensusView] = consensusInfo[index];

  if (what === "prevote") {
    consensusView[addr][voter].Prevote = true;
  } else if (what === "precommit") {
    consensusView[addr][voter].Precommit = true;
  }

  if (index !== -1) {
    consensusInfo[index] = [height, consensusView];
  } else {
    // append at the beginning
    consensusInfo.unshift([height, consensusView]);
  }

  updateState({consensusInfo});

  const op = (i: Types.BlockNumber, view: Types.ConsensusView) => {
    const consensusDetail = view[addr][voter];
    if (consensusDetail.Prevote || consensusDetail.ImplicitPrevote) {
      return false;
    }

    markImplicitlyPre(i, addr, height, what, voter, updateState, getState);
    return true;
  };
  backfill(consensusInfo, height, op, addr, voter);
}

function markFinalized(
  addr: Types.Address,
  finalizedHeight: Types.BlockNumber,
  finalizedHash: Types.BlockHash,
  updateState: (state: any) => void,
  getState: () => State,
) {
  const consensusInfo = getState().consensusInfo;

  initialiseConsensusView(consensusInfo, finalizedHeight, addr, addr);

  const index = consensusInfo
    .findIndex(([blockNumber,]) => blockNumber === finalizedHeight);
  if (index === -1) {
    return;
  }
  const [, consensusView] = consensusInfo[index];

  consensusView[addr][addr].Finalized = true;
  consensusView[addr][addr].FinalizedHash = finalizedHash;
  consensusView[addr][addr].FinalizedHeight = finalizedHeight;

  // this is extrapolated. if this app was just started up we
  // might not yet have received prevotes/precommits. but
  // those are a necessary precondition for finalization, so
  // we can set them and display them in the ui.
  consensusView[addr][addr].Prevote = true;
  consensusView[addr][addr].Precommit = true;

  consensusInfo[index] = [finalizedHeight, consensusView];

  if (index !== -1) {
    consensusInfo[index] = [finalizedHeight, consensusView];
  } else {
    consensusInfo.unshift([finalizedHeight, consensusView]);
  }

  updateState({consensusInfo});
}

// A Prevote or Precommit on a block implicitly includes
// a vote on all preceding blocks. This function marks
// the preceding blocks as implicitly voted on and stores
// a pointer to the block which contains the explicit vote.
function markImplicitlyPre(
  height: Types.BlockNumber,
  addr: Types.Address,
  where: Types.BlockNumber,
  what: string,
  voter: Types.Address,
  updateState: (state: any) => void,
  getState: () => State,
) {
  const consensusInfo = getState().consensusInfo;
  initialiseConsensusView(consensusInfo, height, addr, voter);

  const [consensusView, index] = getConsensusView(consensusInfo, height);

  if (what === "prevote") {
    consensusView[addr][voter].ImplicitPrevote = true;
  } else if (what === "precommit") {
    consensusView[addr][voter].ImplicitPrecommit = true;
  }
  consensusView[addr][voter].ImplicitPointer = where;

  consensusInfo[index] = [height, consensusView];
  updateState({consensusInfo});
}

// Finalizing a block implicitly includes finalizing all
// preceding blocks. This function marks the preceding
// blocks as implicitly finalized on and stores a pointer
// to the block which contains the explicit finalization.
function markImplicitlyFinalized(
  height: Types.BlockNumber,
  addr: Types.Address,
  to: Types.BlockNumber,
  voter: Types.Address,
  updateState: (state: any) => void,
  getState: () => State,
) {
  const consensusInfo = getState().consensusInfo;
  initialiseConsensusView(consensusInfo, height, addr, voter);

  const [consensusView, index] = getConsensusView(consensusInfo, height);

  consensusView[addr][voter].Finalized = true;
  consensusView[addr][voter].FinalizedHeight = height;
  consensusView[addr][voter].ImplicitFinalized = true;
  consensusView[addr][voter].ImplicitPointer = to;

  // this is extrapolated. if this app was just started up we
  // might not yet have received prevotes/precommits. but
  // those are a necessary precondition for finalization, so
  // we can set them and display them in the ui.
  consensusView[addr][voter].Prevote = true;
  consensusView[addr][voter].Precommit = true;
  consensusView[addr][voter].ImplicitPrevote = true;
  consensusView[addr][voter].ImplicitPrecommit = true;

  if (index !== -1) {
    consensusInfo[index] = [height, consensusView];
  } else {
    consensusInfo.unshift([height, consensusView]);
  }

  updateState({consensusInfo});
}

// Initializes the `ConsensusView` with empty objects.
function initialiseConsensusView(
  consensusInfo: Types.ConsensusInfo,
  height: Types.BlockNumber,
  own: Types.Address,
  other: Types.Address,
) {
  const index = consensusInfo.findIndex(item => {
    const [blockNumber,] = item;
    return blockNumber === height;
  });

  let consensusView;
  if (index === -1) {
    consensusView = {} as Types.ConsensusView;
  } else {
    const found = consensusInfo[index];
    const [, foundView] = found;
    consensusView = foundView;
  }

  if (!consensusView[own]) {
    consensusView[own] = {} as Types.ConsensusState;
  }

  if (!consensusView[own][other]) {
    consensusView[own][other] = {} as Types.ConsensusDetail;
  }

  if (index === -1) {
    // append at the beginning
    consensusInfo.unshift([height, consensusView]);
  } else {
    consensusInfo[index] = [height, consensusView];
  }
}

// Fill the block cache back from the `to` number to the last block.
// The function `f` is used to evaluate if we should continue backfilling.
// `f` returns false when backfilling the cache should be stopped, true to continue.
//
// Returns block number until which we backfilled.
function backfill(
  consensusInfo: Types.ConsensusInfo,
  to: Types.BlockNumber,
  f: Maybe<(i: Types.BlockNumber, consensusView: Types.ConsensusView) => boolean>,
  own: Types.Address,
  other: Types.Address,
): Types.BlockNumber {
  let cont = true;
  for (const i of consensusInfo) {
    const [height,] = i;
    if (height >= to) {
      continue;
    }

    initialiseConsensusView(consensusInfo, height, own, other);

    const index = consensusInfo.findIndex(item => {
      const [blockNumber, ] = item;
      return blockNumber === height;
    });

    if (index === -1) {
      cont = false;
      break;
    }

    const [, consensusView] = consensusInfo[index];

    cont = f ? f(height, consensusView) : true;
    if (!cont) {
      break;
    }
  }
  return to;
}

function getConsensusView(
  consensusInfo: Types.ConsensusInfo,
  height: Types.BlockNumber,
): [Types.ConsensusView, number] {
  const index =
    consensusInfo.findIndex(([blockNumber,]) => blockNumber === height);
  const [, consensusView] = consensusInfo[index];
  return [consensusView, index];
}
