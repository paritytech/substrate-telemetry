import * as React from 'react';
import ReactSVG from 'react-svg';
import './Icon.css';

export interface Props {
  src: string;
  alt?: string;
  className?: string;
  onClick?: () => void;
};

export class Icon extends React.Component<{}, Props> {
  public props: Props;

  public shouldComponentUpdate(nextProps: Props) {
    return this.props.src !== nextProps.src
        || this.props.alt !== nextProps.alt
        || this.props.className !== nextProps.className;
  }

  public render() {
    const { alt, className, onClick, src } = this.props;

    return <ReactSVG key={this.props.src} title={alt} className={`Icon ${ className || '' }`} path={src} onClick={onClick} />;
  }
}
