// @ts-check

import petal from '@flowr/eslint-config';

export default petal({
	typescript: true,
	solid: true,
	gitignore: true,
	toml: {
		overrides: {
			'toml/padding-line-between-pairs': 'off',
		},
	},
	ignores: [
		'**/target',
		'**/dist',
		'**/types',
		'**/cache',
		'**/dist',
		'**/.temp',
		'**/*.svg',
		'*.rs',
		'pnpm-lock.yaml',
		'./src-tauri/prisma/migrations/migration_lock.toml',
	],
}, { rules: { 'petal/consistent-list-newline': 'off' } });