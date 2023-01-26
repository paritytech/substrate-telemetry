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
import { Maybe } from '../common';

import './Tooltip.css';

interface TooltipProps {
  text: string;
  copy?: (cb: TooltipCopyCallback) => void;
  position?: 'left' | 'right' | 'center';
  onInit?: (update: TooltipUpdateCallback) => void;
}

export type TooltipUpdateCallback = (text: string) => void;
export type TooltipCopyCallback = Maybe<() => void>;

function copyToClipboard(text: string) {
  const el = document.createElement('textarea');
  el.value = text;
  document.body.appendChild(el);
  el.select();
  document.execCommand('copy');
  document.body.removeChild(el);
}

export function Tooltip({
  text,
  position,
  copy,
  onInit,
}: TooltipProps): JSX.Element {
  const [copied, setCopied] = React.useState<boolean>(false);
  const [timer, setTimer] = React.useState<NodeJS.Timer | null>(null);
  const el = React.useRef<HTMLDivElement>(null);

  const update = React.useCallback(
    (newText: string) => {
      if (el.current) {
        el.current.textContent = newText;
      }
    },
    [el]
  );

  function restore() {
    setCopied(false);
    setTimer(null);
  }

  function onClick() {
    copyToClipboard(text);

    if (timer) {
      clearTimeout(timer);
    }

    setCopied(true);

    setTimer(setTimeout(restore, 2000));
  }

  React.useEffect(() => {
    if (onInit) {
      onInit(update);
    }

    if (copy) {
      copy(onClick);
    }

    return () => {
      if (timer) {
        clearTimeout(timer);
      }

      if (copy) {
        copy(null);
      }
    };
  }, []);

  let tooltipClass = 'Tooltip';

  if (position && position !== 'center') {
    tooltipClass += ` Tooltip-${position}`;
  }

  if (copied) {
    tooltipClass += ' Tooltip-copied';
  }

  return (
    <div className={tooltipClass} ref={el}>
      {copied ? 'Copied to clipboard!' : text}
    </div>
  );
}
