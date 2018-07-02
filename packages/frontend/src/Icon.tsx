import * as React from 'react';
import ReactSVG from 'react-svg';
import './Icon.css';

export interface Props {
    src: string,
    alt: string,
    className?: string,
};

export function Icon(props: Props) {
    return <ReactSVG title={props.alt} className={`Icon ${ props.className || '' }`} path={props.src} />;
}
