# Custom DS5Dongle Notes

This fork tracks upstream `awalol/DS5Dongle` and keeps one local firmware behavior change:

- Base upstream tag: see `custom/upstream-base.txt`
- Custom patch: allow `haptics_gain` down to `0.25` instead of upstream `1.0`
- UF2 files are intentionally not committed or uploaded by the upstream update workflow.

When upstream publishes a newer tag, `.github/workflows/upstream-update.yml` can open a pull request that merges the new upstream tag and reapplies the custom haptics-gain patch.
