import * as React from 'react';

import './Jdenticon.css';

export interface Props {
  hash: string,
  size: string
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
    return <svg
      className="Jdenticon"
      ref={element => this.handleRef(element)}
      width={size}
      height={size}
      data-jdenticon-value={hash}
      />;
  }

  private handleRef(element: any) {
    this.element = element;
  }
}

export default Jdenticon;
