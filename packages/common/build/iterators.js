"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
function* map(iter, fn) {
    for (const item of iter) {
        yield fn(item);
    }
}
exports.map = map;
function* chain(a, b) {
    yield* a;
    yield* b;
}
exports.chain = chain;
function* zip(a, b) {
    let itemA = a.next();
    let itemB = b.next();
    while (!itemA.done && !itemB.done) {
        yield [itemA.value, itemB.value];
        itemA = a.next();
        itemB = b.next();
    }
}
exports.zip = zip;
function* take(iter, n) {
    for (const item of iter) {
        if (n-- === 0) {
            return;
        }
        yield item;
    }
}
exports.take = take;
function skip(iter, n) {
    while (n-- !== 0 && !iter.next().done) { }
    return iter;
}
exports.skip = skip;
function reduce(iter, fn, accumulator) {
    for (const item of iter) {
        accumulator = fn(accumulator, item);
    }
    return accumulator;
}
exports.reduce = reduce;
function join(iter, glue) {
    const first = iter.next();
    if (first.done) {
        return '';
    }
    let result = first.value.toString();
    for (const item of iter) {
        result += glue + item;
    }
    return result;
}
exports.join = join;
//# sourceMappingURL=iterators.js.map