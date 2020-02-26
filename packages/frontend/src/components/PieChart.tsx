import * as React from 'react';

export namespace PieChart {
  export interface Props {
    radius: number,
    slices: number[],
  }
}

export class PieChart extends React.Component<PieChart.Props, {}> {
  public render() {
    const { radius, slices } = this.props;

    let end = -0.25;
    let ex = 0;
    let ey = -radius;

    const paths = slices.map((percent, index) =>  {
      const sx = ex;
      const sy = ey;
      const large = percent > 0.5 ? 1 : 0;

      end += percent;
      [ex, ey] = this.getPoint(end);

      const path = `M 0 0 L ${sx} ${sy} A ${radius} ${radius} 0 ${large} 1 ${ex} ${ey} L0 0`;

      return (
        <path key={index} d={path} stroke="currentColor" fill="currentColor" strokeWidth="1" fillOpacity="0.25" />
      );
    });

    const size = radius * 2 + 2;
    const offset = -radius -1;
    const viewport = `${offset} ${offset} ${size} ${size}`;

    return (
      <svg xmlns="http://www.w3.org/2000/svg" viewBox={viewport} width={size} height={size}>
        {paths}
      </svg>
    )
  }

  private getPoint(percent: number): [number, number] {
    const { radius } = this.props;

    return [
      radius * Math.cos(Math.PI * 2 * percent),
      radius * Math.sin(Math.PI * 2 * percent),
    ];
  }
}
