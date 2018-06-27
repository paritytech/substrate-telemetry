"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/**
 * Higher order function producing new auto-incremented `Id`s.
 */
function idGenerator() {
    let current = 0;
    return () => current++;
}
exports.idGenerator = idGenerator;
class IdSet {
    constructor() {
        this.map = new Map();
    }
    add(item) {
        this.map.set(item.id, item);
    }
    remove(item) {
        this.map.delete(item.id);
    }
    get entries() {
        return this.map.values();
    }
}
exports.IdSet = IdSet;
//# sourceMappingURL=id.js.map