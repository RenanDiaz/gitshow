import type { Commit } from './graphTypes';

// Static test data: 6 commits, 2 branches (main + feature), 1 merge
// Topological order (newest first):
//
//  * e5f6 (HEAD -> main) Merge branch 'feature/login'
//  |\
//  | * d4e5 (feature/login) Add login validation
//  | * c3d4 Add login form UI
//  * | b2c3 Fix README typos
//  |/
//  * a1b2 Add project setup
//  * 0001 Initial commit

type RawCommit = Omit<Commit, 'column' | 'row'>;

export const SAMPLE_COMMITS: RawCommit[] = [
  {
    hash: 'e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4',
    parents: [
      'b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1',
      'd4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3',
    ],
    author: 'Renan Diaz',
    email: 'renan@example.com',
    timestamp: 1711929600,
    refs: [
      { name: 'main', type: 'local' },
      { name: 'HEAD', type: 'head' },
    ],
    subject: "Merge branch 'feature/login'",
  },
  {
    hash: 'b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1',
    parents: ['a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0'],
    author: 'Renan Diaz',
    email: 'renan@example.com',
    timestamp: 1711843200,
    refs: [],
    subject: 'Fix README typos',
  },
  {
    hash: 'd4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3',
    parents: ['c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2'],
    author: 'Renan Diaz',
    email: 'renan@example.com',
    timestamp: 1711756800,
    refs: [{ name: 'feature/login', type: 'local' }],
    subject: 'Add login validation',
  },
  {
    hash: 'c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2',
    parents: ['a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0'],
    author: 'Renan Diaz',
    email: 'renan@example.com',
    timestamp: 1711670400,
    refs: [],
    subject: 'Add login form UI',
  },
  {
    hash: 'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0',
    parents: ['0001a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9'],
    author: 'Renan Diaz',
    email: 'renan@example.com',
    timestamp: 1711584000,
    refs: [],
    subject: 'Add project setup',
  },
  {
    hash: '0001a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9',
    parents: [],
    author: 'Renan Diaz',
    email: 'renan@example.com',
    timestamp: 1711497600,
    refs: [{ name: 'origin/main', type: 'remote' }],
    subject: 'Initial commit',
  },
];
