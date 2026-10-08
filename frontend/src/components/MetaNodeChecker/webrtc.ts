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

// libp2p WebRTC-direct requires the same ICE ufrag (and password) on both
// sides, so the node can recover the credentials from the STUN USERNAME.
const UFRAG_PREFIX = 'libp2p+webrtc+v1/';

// The STUN exchange happens before DTLS, which is the only step that checks
// the certificate fingerprint, so any well-formed value works for our check.
const PLACEHOLDER_FINGERPRINT =
  'sha-256 ' + Array.from({ length: 32 }, () => '00').join(':');

const DEFAULT_TIMEOUT_MS = 5000;

export interface WebRtcAddr {
  ip: string;
  port: number;
}

// Extracts the IPv4 address and UDP port from a multiaddr such as
// `/ip4/1.2.3.4/udp/30333/webrtc-direct/certhash/.../p2p/...`.
export function parseWebRtcMultiaddr(addr: string): WebRtcAddr | null {
  const match = /^\/ip4\/([^/]+)\/udp\/(\d+)\/webrtc-direct(\/|$)/.exec(addr);
  return match ? { ip: match[1], port: Number(match[2]) } : null;
}

export interface StunCheckResult {
  // Round trip time of the STUN binding request, if the browser reports it.
  rttMs: number | null;
}

// Checks that a libp2p WebRTC-direct listener answers STUN.
//
// Browsers can't send raw UDP, and the node ignores STUN binding requests
// without ICE credentials. So we start a WebRTC connection to it: the browser
// then sends STUN binding requests (ICE connectivity checks) carrying our
// ufrag, and we resolve as soon as one of them gets a response.
export async function stunCheck(
  addr: WebRtcAddr,
  timeoutMs: number = DEFAULT_TIMEOUT_MS
): Promise<StunCheckResult> {
  if (typeof RTCPeerConnection === 'undefined') {
    throw new Error('This browser does not support WebRTC');
  }

  const conn = new RTCPeerConnection();
  try {
    conn.createDataChannel('');
    const offer = await conn.createOffer();
    // The node only answers when both sides of the STUN USERNAME are the same
    // ufrag. Chromium lets us pick our own; if the browser doesn't, we give
    // the node the browser's ufrag instead.
    const ufrag = randomUfrag();
    const munged = (offer.sdp ?? '')
      .replace(/^a=ice-ufrag:.*$/gm, `a=ice-ufrag:${ufrag}`)
      .replace(/^a=ice-pwd:.*$/gm, `a=ice-pwd:${ufrag}`);
    try {
      await conn.setLocalDescription({ type: 'offer', sdp: munged });
    } catch (err) {
      await conn.setLocalDescription(offer);
    }

    const localSdp = conn.localDescription?.sdp ?? '';
    const localUfrag = /^a=ice-ufrag:(\S+)/m.exec(localSdp)?.[1];
    const mid = /^a=mid:(\S+)/m.exec(localSdp)?.[1] ?? '0';
    if (!localUfrag) {
      throw new Error('Could not determine the local ICE credentials');
    }

    await conn.setRemoteDescription({
      type: 'answer',
      sdp: answerSdp(addr, localUfrag, mid),
    });

    return await waitForStunResponse(conn, timeoutMs);
  } finally {
    conn.close();
  }
}

function randomUfrag(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  return (
    UFRAG_PREFIX +
    Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
  );
}

// The SDP answer the node would give us: an ICE-lite peer with a single host
// candidate at the given IP and port, using our ufrag as credentials.
function answerSdp({ ip, port }: WebRtcAddr, ufrag: string, mid: string) {
  return [
    'v=0',
    `o=- 0 0 IN IP4 ${ip}`,
    's=-',
    `c=IN IP4 ${ip}`,
    't=0 0',
    'a=ice-lite',
    `m=application ${port} UDP/DTLS/SCTP webrtc-datachannel`,
    `a=mid:${mid}`,
    'a=setup:passive',
    `a=ice-ufrag:${ufrag}`,
    `a=ice-pwd:${ufrag}`,
    `a=fingerprint:${PLACEHOLDER_FINGERPRINT}`,
    'a=sctp-port:5000',
    'a=max-message-size:16384',
    `a=candidate:1467250027 1 UDP 1467250027 ${ip} ${port} typ host`,
    'a=end-of-candidates',
    '',
  ].join('\r\n');
}

function waitForStunResponse(
  pc: RTCPeerConnection,
  timeoutMs: number
): Promise<StunCheckResult> {
  return new Promise((resolve, reject) => {
    const started = Date.now();

    const poll = async () => {
      if (pc.iceConnectionState === 'failed') {
        reject(new Error('no STUN response (ICE connectivity checks failed)'));
        return;
      }

      const stats = await pc.getStats();
      for (const report of stats.values()) {
        if (report.type === 'candidate-pair' && report.responsesReceived > 0) {
          const rtt = report.currentRoundTripTime ?? report.totalRoundTripTime;
          resolve({ rttMs: typeof rtt === 'number' ? rtt * 1000 : null });
          return;
        }
      }

      if (Date.now() - started > timeoutMs) {
        reject(new Error(`no STUN response after ${timeoutMs / 1000}s`));
        return;
      }
      setTimeout(poll, 200);
    };

    poll().catch(reject);
  });
}
