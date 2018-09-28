import * as React from 'react';
import sparkline from "@fnando/sparkline";
import { Tooltip } from './';

import './Sparkline.css';

export namespace Sparkline {
  export interface Props {
    stroke: number;
    width: number;
    height: number;
    values: number[];
    format?: (value: number) => string;
  }
}

export class Sparkline extends React.Component<Sparkline.Props, {}> {
  private el: SVGSVGElement;
  private update: Tooltip.UpdateCallback;

  public componentDidMount() {
    sparkline(this.el, this.props.values, {
      interactive: true,
      onmousemove: this.onMouseMove,
    });
  }

  public render() {
    const { stroke, width, height } = this.props;

    return (
      <Tooltip text="-" onInit={this.onTooltipInit}>
        <svg className="Sparkline" ref={this.onRef} width={width} height={height} stroke-width={stroke} />
      </Tooltip>
    );
  }

  private onRef = (el: SVGSVGElement) => {
    this.el = el;
  }

  private onTooltipInit = (update: Tooltip.UpdateCallback) => {
    this.update = update;
  }

  private onMouseMove = (event: MouseEvent, data: { value: number }) => {
    const { format } = this.props;
    const str = format ? format(data.value) : `${data.value}`;
    this.update(str);
  }
}
