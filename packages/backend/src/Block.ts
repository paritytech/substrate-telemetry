import { Types } from '@dotstats/common';

export default class Block {
	public static readonly ZERO = new Block(0 as Types.BlockNumber, '' as Types.BlockHash);

	public readonly number: Types.BlockNumber;
	public readonly hash: Types.BlockHash;

	constructor(number: Types.BlockNumber, hash: Types.BlockHash) {
		this.number = number;
		this.hash = hash;
	}

	gt(other: Block): boolean {
		return this.number > other.number;
	}

	eq(other: Block): boolean {
		return this.number === other.number && this.hash === other.hash;
	}
}
