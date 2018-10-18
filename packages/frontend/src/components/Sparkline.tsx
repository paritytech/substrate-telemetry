import * as React from 'react';
import { Types, Maybe } from "@dotstats/common";
import sparkline from "@fnando/sparkline";
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
  private el: SVGSVGElement;
  private update: Tooltip.UpdateCallback;

  public componentDidMount() {
    sparkline(this.el, this.props.values, {
      spotRadius: 0.1,
      minScale: this.props.minScale,
      interactive: true,
      onmousemove: this.onMouseMove,
    });
  }

  public shouldComponentUpdate(nextProps: Sparkline.Props): boolean {
    const { stroke, width, height, minScale, format } = this.props;

    if (stroke !== nextProps.stroke || width !== nextProps.width || height !== nextProps.height || format !== nextProps.format) {
      return true;
    }

    if (this.props.values !== nextProps.values) {
      sparkline(this.el, nextProps.values, {
        spotRadius: 0.1,
        minScale,
        interactive: true,
        onmousemove: this.onMouseMove,
      });
    }

    return false;
  }

  public render() {
    const { stroke, width, height } = this.props;

    return (
      <Tooltip text="-" onInit={this.onTooltipInit}>
        <svg className="Sparkline" ref={this.onRef} width={width} height={height} strokeWidth={stroke} />
      </Tooltip>
    );
  }

  private onRef = (el: SVGSVGElement) => {
    this.el = el;
  }

  private onTooltipInit = (update: Tooltip.UpdateCallback) => {
    this.update = update;
  }

  private onMouseMove = (event: MouseEvent, data: { value: number, index: number }) => {
    const { format, stamps } = this.props;
    const str = format ? format(data.value, stamps ? stamps[data.index] : null) : `${data.value}`;
    this.update(str);
  }
}
