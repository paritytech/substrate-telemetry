import * as React from 'react';
import { Types } from '@dotstats/common';
import { MultiCounter } from '../../utils';
import { PieChart } from './';

// import './Settings.css';

export namespace Stats {
  export interface Props {
    nodeVersions: Readonly<MultiCounter<Types.NodeSemver>>;
  }

  // export interface State {
    // display: Display;
    // filter: Maybe<string>;
  // }
}

export class Stats extends React.Component<Stats.Props, {}> {
	private ref: MultiCounter.StateRef;

	constructor(props: Stats.Props) {
		super(props);

		this.ref = props.nodeVersions.ref();
	}

	public shouldComponentUpdate(nextProps: Stats.Props): boolean {
		return nextProps.nodeVersions.hasChangedSince(this.ref);
	}

	public componentDidUpdate() {
		this.ref = this.props.nodeVersions.ref();
	}

  public render() {
  	const { nodeVersions } = this.props;
  	const list = nodeVersions.list();
  	const totalCount = list.reduce((count, entry) => count + entry[1], 0);
  	const slices = list.map(([_, count]) => count / totalCount);

    return (
    	<div>
        <PieChart slices={slices} radius={60} stroke={2} strokeColor="#2C2B2B" />
	    	{
	    		list.map(([version, count]) => {
	    			const percent = count / totalCount;

		    		return (
		    			<div key={version}>{version}: {count} ({Math.floor(percent * 10000) / 100}%)</div>
		    		);
		    	})
	    	}
	    	<div>
	    		Total: {totalCount}
    		</div>
    	</div>
    );
  }
}
