import * as React from 'react';
import { Types } from '../common';
import { MultiCounter } from '../utils';
import { TileRaw } from './Tile';
import { PieChart } from './Stats';

export namespace VersionsTile {
  export interface Props {
    nodeVersions: MultiCounter<Types.NodeSemver>;
    stateRef: MultiCounter.StateRef;
  }
}

export class VersionsTile extends React.Component<VersionsTile.Props, {}> {
  public shouldComponentUpdate(nextProps: VersionsTile.Props) {
    return nextProps.stateRef !== this.props.stateRef;
  }

  public render() {
    const list = this.props.nodeVersions.list();
    const count = list.reduce((acc, [_, n]) => acc + n, 0);
    const slices = list.map(([_, n]) => n / count);

    console.log('list', JSON.stringify(list));

    // const { alt, className, onClick, src } = this.props;
    return (
      <TileRaw>
        <PieChart radius={33} slices={slices} stroke={1} />
      </TileRaw>
    );
  }
}
