// Source code for the Substrate Telemetry Server.
// Copyright (C) 2021 Parity Technologies (UK) Ltd.
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

export * from './helpers';
export * from './id';
export * from './stringify';
export * from './SortedCollection';

import * as Types from './types';
import * as FeedMessage from './feed';

export { Types, FeedMessage };

// Increment this if breaking changes were made to types in `feed.ts`
export const VERSION: Types.FeedVersion = 32 as Types.FeedVersion;
