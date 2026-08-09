## Summary

<!-- What changed, and why is this the right scope for the pull request? -->

## Related issue

<!-- Use "Closes #123" when applicable. -->

## Validation

<!-- List the commands you ran and summarize the results. -->

## Platforms tested

| Platform | Architecture | Shell | Result |
| --- | --- | --- | --- |
| <!-- Windows/macOS/Linux --> | <!-- x86_64/ARM64 --> | <!-- PowerShell/Bash/etc. --> | <!-- Passed/Not tested --> |

## Checklist

- [ ] My changes are focused, and unrelated changes are excluded.
- [ ] I ran `cargo fmt --all -- --check`.
- [ ] I ran `cargo clippy --workspace --tests -- -D warnings`.
- [ ] I ran `cargo test --workspace`.
- [ ] I tested behavior on every relevant platform, or explained above why a platform was not tested.
- [ ] I added or updated tests for behavior changes, or explained above why tests are not needed.
- [ ] I updated user-facing documentation for command, configuration, or behavior changes.
- [ ] I kept the English and Chinese documentation in sync where both versions cover the changed content.
- [ ] I did not include credentials, tokens, private URLs, or other sensitive information.
- [ ] I updated `CHANGELOG.md` when the change is notable to users.
