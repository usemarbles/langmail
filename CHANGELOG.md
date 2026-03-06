# Changelog

All notable changes to this project will be automatically documented in this file.
## [0.5.1] - 2026-03-06

### Refactor

- Rename langmail-core workspace dependency to langmail([a05b6ec](https://github.com/usemarbles/langmail/commit/a05b6ecc4675dbe945a190a1054ea0cd8471d429))
## [0.5.0] - 2026-03-06

### Features

- **langmail-python**: Add generate-import-lib feature to pyo3([b1f90fc](https://github.com/usemarbles/langmail/commit/b1f90fc407a7ba6f391ea478c87f2e288ef12122))
## [0.4.2] - 2026-03-06

### Bug Fixes

- Specify python versions for maturin build([090670a](https://github.com/usemarbles/langmail/commit/090670ac51ffd94f0d70423644591a9d88f0a4b3))
## [0.4.1] - 2026-03-06

### Bug Fixes

- Correct maturin cross-compilation for aarch64 wheels([b5d685b](https://github.com/usemarbles/langmail/commit/b5d685b4c7875ce57ed908af8d0ed88ebe3333bb))
## [0.4.0] - 2026-03-06

### Features

- Add python bindings via pyo3/maturin([24becc6](https://github.com/usemarbles/langmail/commit/24becc6342b81e2f605dbb0deda33fab5cd56fa5))
## [0.3.2] - 2026-03-06

### Bug Fixes

- **ci**: Align Cargo.toml version with package.json (0.3.2)([865a8b1](https://github.com/usemarbles/langmail/commit/865a8b13d810e690fab1d6f36de55c3d028d6e8e))
- **ci**: Switch code review to manual prompt approach([7482c72](https://github.com/usemarbles/langmail/commit/7482c72a975321c0b6ace33052e6c90033f63bd7))
- **ci**: Use --comment flag in code-review plugin prompt([1d8c6f3](https://github.com/usemarbles/langmail/commit/1d8c6f33fadcbf8bbee2d857d2100f122869cb18))
## [0.3.0] - 2026-02-23

### Bug Fixes

- Commit index.js to repo, remove CI artifact dance([76e4841](https://github.com/usemarbles/langmail/commit/76e484149f5bd6708a0e627125f723527a2afa68))

### Features

- Feat(core): add call-to-action extraction with json-ld and heuristic
scoring([3adc0f9](https://github.com/usemarbles/langmail/commit/3adc0f935909966bbd8d2e0f0bca7278d6503a12))

### Refactor

- Rename message_id to rfc_message_id for clarity([9c18ed8](https://github.com/usemarbles/langmail/commit/9c18ed8edcdd47897694528d37a8b1fad5434c21))
## [0.2.0] - 2026-02-23

### Bug Fixes

- Package name([05cb868](https://github.com/usemarbles/langmail/commit/05cb868a9044e17fd4e1dac93185c43311d38b12))
- Do not strip forwarded messages([3711bc1](https://github.com/usemarbles/langmail/commit/3711bc145218e8231ccb2871897bb195c2d2a7a1))
- Strip tables([07deed1](https://github.com/usemarbles/langmail/commit/07deed198e95a8850c02264a6461aa0db5211b6d))
- Remove leading space([a97a5f2](https://github.com/usemarbles/langmail/commit/a97a5f252b4fae885dc6033f4ff9c5209002a61f))
- Time formatting([a7757c0](https://github.com/usemarbles/langmail/commit/a7757c0b182591d2f6f5a4082533e135298ae0a3))
- Bullet points and numbered lists([206170d](https://github.com/usemarbles/langmail/commit/206170d65460ceb469b56a1444218b11f398ab5c))
- **npm**: Update test script to run node tests directly([41eb55c](https://github.com/usemarbles/langmail/commit/41eb55c6ff9d76e5594d0fb483e9c0eb1240c45a))
- **scripts**: Improve anonymizer regex robustness and QP handling([92fba4c](https://github.com/usemarbles/langmail/commit/92fba4cc0e3343f2d27531eaae7c65738a65442e))
- **scripts**: Handle MIME epilogue and avoid global regex footgun in anonymizer([c3512e2](https://github.com/usemarbles/langmail/commit/c3512e286bbf151c25c9ddec3500d868ccab9806))
- **scripts**: Improve robustness of email anonymization script([f131601](https://github.com/usemarbles/langmail/commit/f131601bfe8bd5e11972f3ac833a008a47bab3db))
- **types**: Add deprecated EmailOutput type alias for backwards compatibility([4510535](https://github.com/usemarbles/langmail/commit/4510535eb06a182412ebcc9660f00c1f3430ce99))

### Features

- Properly format hyperlinks as markdown([4fe9db9](https://github.com/usemarbles/langmail/commit/4fe9db9b289467cd375dff237f991a0afc9c95c6))
- **tests**: Add amie promo email fixture for testing([28fdf84](https://github.com/usemarbles/langmail/commit/28fdf840c37d1c68c6351fbfcc60a720ee86791a))
- **html,core**: Improve html entity decoding and body extraction([959adb5](https://github.com/usemarbles/langmail/commit/959adb520d4a1965a4e534e0112cb76a5c187a21))
- **core**: Improve text cleaning and whitespace handling([b565e98](https://github.com/usemarbles/langmail/commit/b565e98c636ba0de4d805956989758264d274211))
- **html**: Add html entity stripping for zero-width characters([9f56248](https://github.com/usemarbles/langmail/commit/9f56248a5b8a3a8d2d764cb235d9042cd812574a))
- **core**: Add zero-width character stripping([0aa332d](https://github.com/usemarbles/langmail/commit/0aa332db898161244455c3ad508401ab211ac69a))
- **types**: Add `toLlmContext` function to format email for llm prompts([db7f6ae](https://github.com/usemarbles/langmail/commit/db7f6ae8effc9786d226c99e3d3a156ca70b0534))
- **types**: Add `to_llm_context` method for email processing([2d94aa2](https://github.com/usemarbles/langmail/commit/2d94aa2de943e5f7bb2f0ddb4bf6f3a7935565ae))
- **scripts**: Enhance email anonymization for better PII handling([4506ed8](https://github.com/usemarbles/langmail/commit/4506ed84b7a7e37ca587084a90ff77bb11f201d1))
- **core**: Enhance qp text anonymization with preserving line breaks([a9059e9](https://github.com/usemarbles/langmail/commit/a9059e9fa7a654b43a4d58e1b84000a21b62dc60))
- **anonymize**: Enhance pii anonymization in email fixtures([a26f85d](https://github.com/usemarbles/langmail/commit/a26f85d15b728084042cba48dbc1c098acb729a4))
- **scripts**: Add email fixture anonymization script([b31e47c](https://github.com/usemarbles/langmail/commit/b31e47ce5bd06ba935aa1656808138f6f67a67eb))

### Refactor

- Formatting([fd33ed3](https://github.com/usemarbles/langmail/commit/fd33ed377f015c7c467230cac758025e8f41a865))
- Remove unnecessary blank line at end of `lib.rs`([79356be](https://github.com/usemarbles/langmail/commit/79356be9eb6ea8e550cf13044563070b2c2894f7))
- Refactor: improve whitespace and invisible character handling in email
preprocessing([8a6f1fa](https://github.com/usemarbles/langmail/commit/8a6f1fa83b62ca6d271bfcd9c49a22816f06f52c))
- Rename `EmailOutput` to `ProcessedEmail`([7ca0dd3](https://github.com/usemarbles/langmail/commit/7ca0dd346da11fc1c7a1f424217df5b39efcf3d4))

