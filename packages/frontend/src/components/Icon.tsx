import * as React from 'react';
import ReactSVG from 'react-svg';
import './Icon.css';

export interface Props {
  src: string,
  alt: string,
  className?: string,
};

export class Icon extends React.Component<{}, Props> {
  public props: Props;

  public shouldComponentUpdate() {
    return false;
  }

  public render() {
    const { alt, className, src } = this.props;

    return <ReactSVG title={alt} className={`Icon ${ className || '' }`} path={src} />;
  }
}
