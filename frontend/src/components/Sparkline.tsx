import * as React from 'react';
import { Types, Maybe } from '../common';
import { Tooltip } from './';

import './Sparkline.css';

export namespace Sparkline {
  export interface Props {
    stroke: number;
    width: number;
    height: number;
    values: number[];
    stamps?: Types.Timestamp[];
    minScale?: number;
    format?: (value: number, stamp: Maybe<Types.Timestamp>) => string;
  }
}

export class Sparkline extends React.Component<Sparkline.Props, {}> {
  private cursor: SVGPathElement;
  private update: Tooltip.UpdateCallback;

  public shouldComponentUpdate(nextProps: Sparkline.Props): boolean {
    const { stroke, width, height, minScale, format, values } = this.props;

    return (
      values !== nextProps.values ||
      stroke !== nextProps.stroke ||
      width !== nextProps.width ||
      height !== nextProps.height ||
      format !== nextProps.format
    );
  }

  public render() {
    const { stroke, width, height, minScale, values } = this.props;
    const padding = stroke / 2;
    const paddedHeight = height - padding;
    const paddedWidth = width - 2;

    const max = Math.max(minScale || 0, ...values);
    const offset = paddedWidth / (values.length - 1);

    let path = '';

    values.forEach((value, index) => {
      const x = 1 + index * offset;
      const y = padding + (1 - value / max) * paddedHeight;

      if (path) {
        path += ` L ${x} ${y}`;
      } else {
        path = `${x} ${y}`;
      }
    });

    return (
      <>
        <Tooltip text="-" onInit={this.onTooltipInit} />
        <svg
          className="Sparkline"
          width={width}
          height={height}
          strokeWidth={stroke}
          onMouseMove={this.onMouseMove}
          onMouseLeave={this.onMouseLeave}
        >
          <path d={`M 0 ${height} L ${path} V ${height} Z`} stroke="none" />
          <path d={`M ${path}`} fill="none" />
          <path className="Sparkline-cursor" strokeWidth="2" ref={this.onRef} />
        </svg>
      </>
    );
  }

  private onRef = (cursor: SVGPathElement) => {
    this.cursor = cursor;
  };

  private onTooltipInit = (update: Tooltip.UpdateCallback) => {
    this.update = update;
  };

  private onMouseMove = (
    event: React.MouseEvent<SVGSVGElement, MouseEvent>
  ) => {
    const { width, height, values, format, stamps } = this.props;
    const offset = (width - 2) / (values.length - 1);
    const cur =
      Math.round((event.nativeEvent.offsetX - 1 - offset / 2) / offset) | 0;

    this.cursor.setAttribute('d', `M ${1 + offset * cur} 0 V ${height}`);

    const str = format
      ? format(values[cur], stamps ? stamps[cur] : null)
      : `${values[cur]}`;
    this.update(str);
  };

  private onMouseLeave = () => {
    this.cursor.removeAttribute('d');
  };
}
