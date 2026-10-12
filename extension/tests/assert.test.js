import {test} from './support/harness.js';
import {assertEqual, assertRejects, assertTrue} from './support/assert.js';

test('assertEqual ignores key order', () => assertEqual({b: 1, a: [2]}, {a: [2], b: 1}));

test('assertEqual reports a difference', () => {
    let message = '';
    try {
        assertEqual({a: 1}, {a: 2});
    } catch (error) {
        message = error.message;
    }
    assertEqual(message, 'expected {"a":2}, got {"a":1}');
});

test('assertRejects fails on a resolved promise', async () => {
    let failed = false;
    await assertRejects(Promise.resolve(), /never/).catch(() => {
        failed = true;
    });
    assertTrue(failed, 'a resolved promise must fail assertRejects');
});
