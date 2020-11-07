import * as React from 'react';
import { Chain } from './';
import { Icon } from '../';
import { setHashData } from '../../utils';

import './Tab.css';

export namespace Tab {
  export interface Props {
    label: string;
    icon: string;
    display: Chain.Display;
    current: string;
    tab: string;
    setDisplay: (display: Chain.Display) => void;
  }
}

export class Tab extends React.Component<Tab.Props, {}> {
  public render() {
    const { label, icon, display, current } = this.props;
    const highlight = display === current;
    const className = highlight ? 'Chain-Tab-on Chain-Tab' : 'Chain-Tab';

    return (
      <div className={className} onClick={this.onClick} title={label}>
        <Icon src={icon} />
      </div>
    );
  }

  private onClick = () => {
    const { tab, display, setDisplay } = this.props;
    setHashData({ tab });
    setDisplay(display);
  };
}
