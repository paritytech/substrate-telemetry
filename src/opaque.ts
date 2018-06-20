// Hack for Opaque Types
export type Opaque<T, Label> = T & {__TYPE__: Label};
