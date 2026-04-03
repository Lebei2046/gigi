// ESLint v9 configuration
import react from 'eslint-plugin-react'
import typescript from '@typescript-eslint/eslint-plugin'
import parser from '@typescript-eslint/parser'

export default [
  {
    ignores: [
      'node_modules/**',
      'dist/**',
      'build/**',
      '.git/**',
      '**/*.md',
      'src-tauri/**',
    ],
  },
  {
    files: ['**/*.{ts,tsx,js,jsx}'],
    languageOptions: {
      parser: parser,
      parserOptions: {
        ecmaFeatures: {
          jsx: true,
        },
        ecmaVersion: 'latest',
        sourceType: 'module',
      },
      globals: {
        browser: true,
        node: true,
        console: true,
        window: true,
        document: true,
        navigator: true,
        localStorage: true,
        sessionStorage: true,
        indexedDB: true,
        IDBDatabase: true,
        IDBOpenDBRequest: true,
        IDBTransactionMode: true,
        IDBObjectStore: true,
        IDBKeyRange: true,
        IDBRequest: true,
        Blob: true,
        File: true,
        URL: true,
        btoa: true,
        atob: true,
        setTimeout: true,
        clearTimeout: true,
        setInterval: true,
        clearInterval: true,
        CustomEvent: true,
        crypto: true,
        describe: true,
        it: true,
        test: true,
        expect: true,
        beforeEach: true,
        afterEach: true,
        KeyboardEvent: true,
        alert: true,
        PermissionState: true,
        PermissionName: true,
        HTMLDivElement: true,
        React: true,
        HTMLInputElement: true,
        NodeJS: true,
        Event: true,
        EventListener: true,
      },
    },
    plugins: {
      react: react,
      '@typescript-eslint': typescript,
    },
    rules: {
      // React rules
      'react/react-in-jsx-scope': 'off',
      'react/prop-types': 'off',

      // TypeScript rules
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_' },
      ],
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/no-explicit-any': 'warn',

      // General rules
      'prefer-const': 'error',
      'no-var': 'error',
      'no-unused-vars': 'off', // Handled by TypeScript
      'no-undef': 'error',
    },
    settings: {
      react: {
        version: 'detect',
      },
    },
  },
]
