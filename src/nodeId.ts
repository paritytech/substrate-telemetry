import { Opaque } from './opaque';

let currentId = 0;

export type NodeId = Opaque<number, "NodeId">;

export function getId(): NodeId {
    return currentId++ as NodeId;
}
