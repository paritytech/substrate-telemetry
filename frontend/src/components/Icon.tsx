// Source code for the Substrate Telemetry Server.
// Copyright (C) 2023 Parity Technologies (UK) Ltd.
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
import './Icon.css';
import { getSVGShadowRoot, W3SVG } from '../utils';

interface IconProps {
  src: string;
  className?: string;
  onClick?: () => void;
}

const symbols = new Map<string, string>();

let symbolId = 0;

// Lazily render the icon in the DOM, so that we can referenced
// it by id using shadow DOM.
function renderShadowIcon(src: string): string {
  let symbol = symbols.get(src);

  if (!symbol) {
    symbol = `icon${symbolId}`;
    symbolId += 1;

    symbols.set(src, symbol);

    fetch(src).then(async (response) => {
      const html = await response.text();
      const temp = document.createElement('div');

      temp.innerHTML = html;

      const tempSVG = temp.querySelector('svg') as SVGSVGElement;
      const symEl = document.createElementNS(W3SVG, 'symbol');
      const viewBox = tempSVG.getAttribute('viewBox');

      symEl.setAttribute('id', symbol as string);
      if (viewBox) {
        symEl.setAttribute('viewBox', viewBox);
      }

      for (const child of Array.from(tempSVG.childNodes)) {
        symEl.appendChild(child);
      }

      getSVGShadowRoot().appendChild(symEl);
    });
  }

  return symbol;
}

export class Icon extends React.Component<IconProps> {
  public props: IconProps;

  public shouldComponentUpdate(nextProps: IconProps) {
    return (
      this.props.src !== nextProps.src ||
      this.props.className !== nextProps.className
    );
  }

  public render() {
    const { className, onClick, src } = this.props;
    const symbol = renderShadowIcon(src);

    // Use `href` for a shadow DOM reference to the rendered icon
    return (
      <svg className={`Icon ${className || ''}`} onClick={onClick}>
        <use href={`#${symbol}`} />
      </svg>
    );
  }
}
