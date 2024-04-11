window.BENCHMARK_DATA = {
  "lastUpdate": 1712858198473,
  "repoUrl": "https://github.com/metaplex-foundation/mpl-core",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "90809591+blockiosaurus@users.noreply.github.com",
            "name": "blockiosaurus",
            "username": "blockiosaurus"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8eded814fec6b29a032ac0ba77de432ce224e984",
          "message": "Merge pull request #62 from blockiosaurus/main\n\nAdding benchmarking suite",
          "timestamp": "2024-04-11T13:52:21-04:00",
          "tree_id": "898fb163e89935d26c04391a6c12b42a3ad74a8e",
          "url": "https://github.com/metaplex-foundation/mpl-core/commit/8eded814fec6b29a032ac0ba77de432ce224e984"
        },
        "date": 1712858198021,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "CU: create a new, empty asset with empty collection",
            "value": 35167,
            "unit": "Compute Units"
          },
          {
            "name": "Space: create a new, empty asset with empty collection",
            "value": 91,
            "unit": "Bytes"
          },
          {
            "name": "CU: create a new asset with plugins and empty collection",
            "value": 41967,
            "unit": "Compute Units"
          },
          {
            "name": "Space: create a new asset with plugins and empty collection",
            "value": 194,
            "unit": "Bytes"
          },
          {
            "name": "CU: create a new asset with plugins",
            "value": 36126,
            "unit": "Compute Units"
          },
          {
            "name": "Space: create a new asset with plugins",
            "value": 194,
            "unit": "Bytes"
          },
          {
            "name": "CU: create a new, empty asset",
            "value": 7404,
            "unit": "Compute Units"
          },
          {
            "name": "Space: create a new, empty asset",
            "value": 91,
            "unit": "Bytes"
          },
          {
            "name": "CU: transfer an empty asset with empty collection",
            "value": 10995,
            "unit": "Compute Units"
          },
          {
            "name": "CU: transfer an asset with plugins and empty collection",
            "value": 69889,
            "unit": "Compute Units"
          },
          {
            "name": "CU: transfer an asset with plugins",
            "value": 68783,
            "unit": "Compute Units"
          },
          {
            "name": "CU: transfer an empty asset",
            "value": 8704,
            "unit": "Compute Units"
          }
        ]
      }
    ]
  }
}