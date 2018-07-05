import * as React from 'react';
import './Tile.css';
import { Icon } from './Icon';

export namespace Tile {
    export interface Props {
        title: string,
        icon: string,
        children?: React.ReactNode,
    }
}

export function Tile(props: Tile.Props) {
    return (
        <div className="Tile">
            <Icon src={props.icon} alt={props.title} /> {props.children}
        </div>
    );
}
