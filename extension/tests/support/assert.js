export function assertEqual(actual, expected) {
    const [got, wanted] = [canonical(actual), canonical(expected)];
    if (got !== wanted)
        throw new Error(`expected ${wanted}, got ${got}`);
}

export function assertTrue(condition, message) {
    if (!condition)
        throw new Error(message);
}

export async function assertRejects(promise, pattern) {
    try {
        await promise;
    } catch (error) {
        assertTrue(pattern.test(error.message), `rejected with "${error.message}", expected ${pattern}`);
        return;
    }
    throw new Error(`expected a rejection matching ${pattern}`);
}

function canonical(value) {
    return JSON.stringify(value, sortedKeys);
}

function sortedKeys(key, value) {
    if (value === null || typeof value !== 'object' || Array.isArray(value))
        return value;
    return Object.fromEntries(Object.entries(value).sort(([left], [right]) => left.localeCompare(right)));
}
