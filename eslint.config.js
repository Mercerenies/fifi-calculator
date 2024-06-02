
// @ts-check

import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import globals from 'globals';

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.strict,
  ...tseslint.configs.stylistic,
  {
    languageOptions: {
      globals: { ...globals.browser },
    },
    rules: {
      "@typescript-eslint/no-inferrable-types": ["error", {
        "ignoreParameters": true,
        "ignoreProperties": true,
      }],
    },
  },
);
