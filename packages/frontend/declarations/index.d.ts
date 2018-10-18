declare module '@fnando/sparkline' {
  namespace sparkline {
    export interface Options {
      minScale?: number;
      spotRadius?: number;
      cursorWidth?: number;
      interactive?: boolean;
      onmousemove?: (event: MouseEvent, datapoint: { x: number, y: number, index: number, value: number });
      onmouseout?: () => void;
    }
  }

  function sparkline(svg: SVGSVGElement, values: number[], options?: sparkline.Options): void;

  export = sparkline;
}
