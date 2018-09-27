import * as React from 'react';
import { Chain } from './';
import { Icon } from '../';

import './Tab.css';

export namespace Tab {
  export interface Props {
    label: string;
    icon: string;
    display: Chain.Display;
    current: string;
    hash: string;
    setDisplay: (display: Chain.Display) => void;
  }
}

export class Tab extends React.Component<Tab.Props, {}> {
  public render() {
    const { label, icon, display, current } = this.props;
    const highlight = display === current;
    const className = highlight ? 'Chain-Tab-on Chain-Tab' : 'Chain-Tab';

    return (
      <div className={className} onClick={this.onClick}>
        <Icon src={icon} alt={label} />
      </div>
    );
  }

  private onClick = () => {
    const { hash, display, setDisplay } = this.props;
    window.location.hash = hash;
    setDisplay(display);
  }
}
