const registered = [];

export function test(name, body) {
    registered.push({name, body});
}

export async function runAll() {
    let failures = 0;
    for (const {name, body} of registered)
        failures += await runOne(name, body);
    print(`${registered.length - failures} passed, ${failures} failed`);
    return failures;
}

async function runOne(name, body) {
    try {
        await body();
        print(`ok - ${name}`);
        return 0;
    } catch (error) {
        print(`not ok - ${name}\n  ${error.message}\n${error.stack}`);
        return 1;
    }
}
