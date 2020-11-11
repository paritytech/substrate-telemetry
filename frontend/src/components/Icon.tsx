import * as React from 'react';
import './Icon.css';
import { getSVGShadowRoot, W3SVG } from '../utils';

export namespace Icon {
  export interface Props {
    src: string;
    className?: string;
    onClick?: () => void;
  }
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

export class Icon extends React.Component<Icon.Props, {}> {
  public props: Icon.Props;

  public shouldComponentUpdate(nextProps: Icon.Props) {
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
