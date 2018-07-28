import { Types, Maybe } from '@dotstats/common';

export interface Props {
  id: Types.NodeId;
  nodeDetails: Types.NodeDetails;
  nodeStats: Types.NodeStats;
  blockDetails: Types.BlockDetails;
  location: Maybe<Types.NodeLocation>;
}
