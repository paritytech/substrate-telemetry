import * as React from 'react';
import './Icon.css';

export interface Props {
  src: string;
  alt?: string;
  className?: string;
  onClick?: () => void;
}

const W3SVG = 'http://www.w3.org/2000/svg';
const symbols = new Map<string, string>();

let root: null | SVGSVGElement = null;
let symbolId = 0;

// Get a root node where all the icons are stored within the DOM
function getRoot(): SVGSVGElement {
  if (!root) {
    root = document.createElementNS(W3SVG, 'svg');
    root.setAttribute('class', 'Icon-symbol-root');
    root.setAttribute('style', 'display: none;');

    document.body.appendChild(root);
  }

  return root;
}

// Get the DOM id of the node matching the icon
function getSVGSymbol(src: string): string {
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

      if (viewBox) {
        symEl.setAttribute('viewBox', viewBox);
      }
      symEl.setAttribute('id', symbol as string);

      for (const child of Array.from(tempSVG.childNodes)) {
        symEl.appendChild(child);
      }

      getRoot().appendChild(symEl);
    });
  }

  return symbol;
}

export class Icon extends React.Component<{}, Props> {
  public props: Props;

  public shouldComponentUpdate(nextProps: Props) {
    return (
      this.props.src !== nextProps.src ||
      this.props.alt !== nextProps.alt ||
      this.props.className !== nextProps.className
    );
  }

  public render() {
    const { alt, className, onClick, src } = this.props;
    const symbol = getSVGSymbol(src);

    // Use xlink:href for a shadow DOM reference to the rendered icon
    return (
      <div
        key={src}
        className={`Icon ${className || ''}`}
        title={alt}
        onClick={onClick}
      >
        <svg xmlns={W3SVG} xmlnsXlink="http://www.w3.org/1999/xlink">
          <use xlinkHref={`#${symbol}`} />
        </svg>
      </div>
    );
  }
}
