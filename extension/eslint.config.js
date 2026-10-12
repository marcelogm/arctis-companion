const noComments = {
    meta: {
        type: 'problem',
        messages: {comment: 'Comments are not allowed; rename or extract instead.'},
        schema: [],
    },
    create(context) {
        return {
            Program() {
                for (const comment of context.sourceCode.getAllComments())
                    context.report({loc: comment.loc, messageId: 'comment'});
            },
        };
    },
};

const gjsGlobals = {
    ARGV: 'readonly',
    imports: 'readonly',
    log: 'readonly',
    logError: 'readonly',
    print: 'readonly',
    printerr: 'readonly',
};

export default [
    {
        files: ['**/*.js'],
        languageOptions: {
            ecmaVersion: 2024,
            sourceType: 'module',
            globals: gjsGlobals,
        },
        plugins: {local: {rules: {'no-comments': noComments}}},
        rules: {
            'local/no-comments': 'error',
            'complexity': ['error', 6],
            'max-depth': ['error', 3],
            'max-lines': ['warn', 400],
            'no-undef': 'error',
            'no-unused-vars': 'error',
        },
    },
];
