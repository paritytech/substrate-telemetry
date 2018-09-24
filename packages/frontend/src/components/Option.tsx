import * as React from 'react';
import { Icon } from './';

import './Option.css';

export namespace Option {
  export interface Props {
    icon: string;
    label: string;
    checked: boolean;
    onClick: () => void;
  }
}

export function Option(props: Option.Props): React.ReactElement<any> {
  const className = props.checked ? "Option Option-on" : "Option";

  return (
    <p className={className} onClick={props.onClick}>
      <Icon src={props.icon} alt={props.label} />
      {props.label}
      <span className="Option-switch">
        <span className="Option-knob" />
      </span>
    </p>
  );
}
