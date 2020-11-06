import * as React from 'react';
import { Column } from './';
import { Node } from '../../../state';
import { Icon } from '../../';
import icon from '../../../icons/network.svg';
import externalLinkIcon from '../../../icons/link-external.svg';
import { getHashData } from '../../../utils';

const URI_BASE =
  window.location.protocol === 'https:'
    ? `/network_state/`
    : `http://${window.location.hostname}:8000/network_state/`;

export class NetworkStateColumn extends React.Component<Column.Props, {}> {
  public static readonly label = 'Network State';
  public static readonly icon = icon;
  public static readonly width = 16;
  public static readonly setting = 'networkstate';
  public static readonly sortBy = null;

  public shouldComponentUpdate(nextProps: Column.Props) {
    // Network state link changes when the node does
    return this.props.node !== nextProps.node;
  }

  render() {
    const { id } = this.props.node;
    const chainLabel = getHashData().chain;

    if (!chainLabel) {
      return <td className="Row--td">-</td>;
    }

    const uri = `${URI_BASE}${encodeURIComponent(chainLabel)}/${id}/`;

    return (
      <td className="Row--td">
        <a className="Row--a" href={uri} target="_blank">
          <Icon src={externalLinkIcon} />
        </a>
      </td>
    );
  }
}
