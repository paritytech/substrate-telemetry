// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

import * as React from 'react';

import './Jdenticon.css';

export interface Props {
  hash: string;
  size: string;
}

class Jdenticon extends React.Component<Props, {}> {
  private element = null;

  public componentDidUpdate() {
    const jdenticon = (window as any).jdenticon;
    if (jdenticon) {
      jdenticon.update(this.element);
    }
  }

  public componentDidMount() {
    const jdenticon = (window as any).jdenticon;
    if (jdenticon) {
      jdenticon.update(this.element);
    }
  }

  public render() {
    const { hash, size } = this.props;
    return (
      <svg
        className="Jdenticon"
        ref={(element) => this.handleRef(element)}
        width={size}
        height={size}
        data-jdenticon-value={hash}
      />
    );
  }

  private handleRef(element: any) {
    this.element = element;
  }
}

export default Jdenticon;
